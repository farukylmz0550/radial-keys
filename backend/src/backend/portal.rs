use super::shortcuts::{ActivationState, InputEvent};
use anyhow::{Context, anyhow};
use futures_util::StreamExt;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, oneshot},
    task::JoinHandle,
    time::timeout,
};
use uuid::Uuid;
use zbus::{
    Connection, Proxy,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

const PORTAL_NAME: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const GLOBAL_SHORTCUTS: &str = "org.freedesktop.portal.GlobalShortcuts";

/// An active user-approved portal binding. Dropping the backend disconnects it;
/// `close` also asks the portal to close the session explicitly.
pub struct PortalSession {
    connection: Connection,
    session_handle: OwnedObjectPath,
    listener: JoinHandle<()>,
}

impl PortalSession {
    pub fn is_running(&self) -> bool {
        !self.listener.is_finished()
    }

    pub async fn close(self) {
        self.listener.abort();
        if let Ok(proxy) = Proxy::new(
            &self.connection,
            PORTAL_NAME,
            self.session_handle.as_str(),
            "org.freedesktop.portal.Session",
        )
        .await
            && let Err(error) = proxy.call_method("Close", &()).await
        {
            tracing::debug!(%error, "portal session close failed");
        }
    }
}

/// Checks whether the selected desktop portal exposes GlobalShortcuts.
pub async fn probe() -> String {
    let connection = match Connection::session().await {
        Ok(connection) => connection,
        Err(error) => return format!("unavailable: cannot connect to session bus ({error})"),
    };
    let proxy = match Proxy::new(&connection, PORTAL_NAME, PORTAL_PATH, GLOBAL_SHORTCUTS).await {
        Ok(proxy) => proxy,
        Err(error) => return format!("unavailable: GlobalShortcuts interface not found ({error})"),
    };
    match proxy.get_property::<u32>("version").await {
        Ok(version) => format!("available (portal interface version {version})"),
        Err(error) => format!("interface found but version unavailable ({error})"),
    }
}

/// Registers one shortcut through the user-mediated XDG portal.
/// The call waits for the portal request to complete; the desktop may show a
/// configuration or conflict dialog. The caller should invoke this only after
/// the user explicitly enables global activation.
pub async fn register_shortcut(
    shortcut: &str,
    activation: Arc<Mutex<ActivationState>>,
) -> anyhow::Result<PortalSession> {
    let connection = Connection::session()
        .await
        .context("cannot connect to the session bus for shortcut registration")?;
    let shortcut = shortcut.to_owned();
    let connection_for_task = connection.clone();
    let (ready_sender, ready_receiver) = oneshot::channel();
    let listener = tokio::spawn(async move {
        let mut ready_sender = Some(ready_sender);
        if let Err(error) =
            run_registration(connection_for_task, shortcut, activation, &mut ready_sender).await
        {
            if let Some(ready_sender) = ready_sender {
                let _ = ready_sender.send(Err(error.to_string()));
            }
            tracing::warn!(%error, "global shortcut portal session ended");
        }
    });

    let session_handle = match timeout(Duration::from_secs(300), ready_receiver).await {
        Ok(Ok(Ok(session))) => session,
        Ok(Ok(Err(error))) => {
            listener.abort();
            return Err(anyhow!(error));
        }
        Ok(Err(_)) => {
            listener.abort();
            return Err(anyhow!(
                "global shortcut registration task stopped unexpectedly"
            ));
        }
        Err(_) => {
            listener.abort();
            return Err(anyhow!(
                "timed out waiting for portal shortcut confirmation"
            ));
        }
    };

    Ok(PortalSession {
        connection,
        session_handle,
        listener,
    })
}

async fn run_registration(
    connection: Connection,
    shortcut: String,
    activation: Arc<Mutex<ActivationState>>,
    ready: &mut Option<oneshot::Sender<Result<OwnedObjectPath, String>>>,
) -> anyhow::Result<()> {
    let portal = Proxy::new(&connection, PORTAL_NAME, PORTAL_PATH, GLOBAL_SHORTCUTS).await?;
    let trigger = super::shortcuts::Shortcut::parse(&shortcut)?
        .portal_trigger()
        .context("the portal shortcut format cannot represent this modifier combination")?;

    let session_token = format!("rk_session_{}", Uuid::new_v4().simple());
    let create_token = format!("rk_create_{}", Uuid::new_v4().simple());
    let (create_request_path, mut create_responses) =
        request_stream(&connection, &create_token).await?;
    let mut create_options = HashMap::new();
    create_options.insert("handle_token", Value::new(create_token.as_str()));
    create_options.insert("session_handle_token", Value::new(session_token.as_str()));
    let create_reply = portal
        .call_method("CreateSession", &create_options)
        .await
        .context("portal CreateSession call failed")?;
    let returned_create_path: OwnedObjectPath = create_reply.body().deserialize()?;
    if returned_create_path != create_request_path {
        return Err(anyhow!(
            "portal returned an unexpected CreateSession request handle"
        ));
    }
    let create_response = await_response(&mut create_responses).await?;
    let (response_code, create_results): (u32, HashMap<String, OwnedValue>) =
        create_response.body().deserialize()?;
    if response_code != 0 {
        return Err(anyhow!(
            "portal cancelled shortcut session creation (response {response_code})"
        ));
    }
    // The portal protocol unfortunately returns this object path as a string.
    let session_value = create_results
        .get("session_handle")
        .ok_or_else(|| anyhow!("portal response did not include session_handle"))?;
    let session_string = String::try_from(session_value.try_clone()?)?;
    let session_handle = OwnedObjectPath::try_from(session_string)?;

    let (activated, deactivated) = match bind_shortcut(&connection, &session_handle, trigger).await
    {
        Ok(streams) => streams,
        Err(error) => {
            close_portal_session(&connection, &session_handle).await;
            return Err(error);
        }
    };

    if let Some(ready) = ready.take()
        && ready.send(Ok(session_handle.clone())).is_err()
    {
        close_portal_session(&connection, &session_handle).await;
        return Ok(());
    }

    let mut activated = activated;
    let mut deactivated = deactivated;
    loop {
        let (is_activation, message) = tokio::select! {
            message = activated.next() => (true, message),
            message = deactivated.next() => (false, message),
        };
        let Some(message) = message else { break };
        let (event_session, shortcut_id, _timestamp, _options): (
            OwnedObjectPath,
            String,
            u64,
            HashMap<String, OwnedValue>,
        ) = match message.body().deserialize() {
            Ok(signal) => signal,
            Err(error) => {
                tracing::warn!(%error, "could not decode GlobalShortcuts signal");
                continue;
            }
        };
        if event_session != session_handle || shortcut_id != "activate-menu" {
            continue;
        }
        let event = if is_activation {
            InputEvent::Activated
        } else {
            InputEvent::Deactivated
        };
        tracing::info!(
            shortcut_id,
            event = if is_activation {
                "activated"
            } else {
                "deactivated"
            },
            "received desktop shortcut event"
        );
        let mut state = activation.lock().await;
        let transition = state.handle(event, &[]);
        tracing::debug!(?transition, "shortcut event updated activation state");
    }

    close_portal_session(&connection, &session_handle).await;
    Ok(())
}

async fn bind_shortcut<'a>(
    connection: &'a Connection,
    session_handle: &OwnedObjectPath,
    trigger: String,
) -> anyhow::Result<(zbus::proxy::SignalStream<'a>, zbus::proxy::SignalStream<'a>)> {
    let portal = Proxy::new(connection, PORTAL_NAME, PORTAL_PATH, GLOBAL_SHORTCUTS).await?;
    // Subscribe without an arg0 match: the session handle is a D-Bus object
    // path, and portal implementations may not match it as a string. The
    // listener validates the typed handle after decoding each signal.
    let activated = portal.receive_signal("Activated").await?;
    let deactivated = portal.receive_signal("Deactivated").await?;

    let bind_token = format!("rk_bind_{}", Uuid::new_v4().simple());
    let (bind_request_path, mut bind_responses) = request_stream(connection, &bind_token).await?;
    let mut properties = HashMap::new();
    properties.insert(
        "description",
        Value::new("Open or close the Radial Keys menu"),
    );
    properties.insert("preferred_trigger", Value::new(trigger));
    let shortcuts = vec![("activate-menu", properties)];
    let mut bind_options = HashMap::new();
    bind_options.insert("handle_token", Value::new(bind_token.as_str()));
    let bind_args = (session_handle, shortcuts, "", bind_options);
    let bind_reply = portal
        .call_method("BindShortcuts", &bind_args)
        .await
        .context("portal BindShortcuts call failed")?;
    let returned_bind_path: OwnedObjectPath = bind_reply.body().deserialize()?;
    if returned_bind_path != bind_request_path {
        return Err(anyhow!(
            "portal returned an unexpected BindShortcuts request handle"
        ));
    }
    let bind_response = await_response(&mut bind_responses).await?;
    let (response_code, bind_results): (u32, HashMap<String, OwnedValue>) =
        bind_response.body().deserialize()?;
    if response_code != 0 {
        return Err(anyhow!(
            "portal did not register the shortcut (response {response_code})"
        ));
    }
    tracing::info!(
        ?bind_results,
        "desktop portal accepted the shortcut binding"
    );
    Ok((activated, deactivated))
}

async fn request_stream<'a>(
    connection: &'a Connection,
    token: &str,
) -> anyhow::Result<(OwnedObjectPath, zbus::proxy::SignalStream<'a>)> {
    let sender = connection
        .unique_name()
        .ok_or_else(|| anyhow!("session bus did not assign a unique name"))?;
    let sender_component = sender.as_str().trim_start_matches(':').replace('.', "_");
    let request_path =
        OwnedObjectPath::try_from(format!("{PORTAL_PATH}/request/{sender_component}/{token}"))?;
    let request_proxy = Proxy::new(
        connection,
        PORTAL_NAME,
        request_path.as_str(),
        "org.freedesktop.portal.Request",
    )
    .await?;
    let stream = request_proxy.receive_signal("Response").await?;
    Ok((request_path, stream))
}

async fn await_response(
    stream: &mut zbus::proxy::SignalStream<'_>,
) -> anyhow::Result<zbus::Message> {
    timeout(Duration::from_secs(300), stream.next())
        .await
        .context("timed out waiting for portal response")?
        .ok_or_else(|| anyhow!("portal request ended without a response"))
}

async fn close_portal_session(connection: &Connection, session_handle: &OwnedObjectPath) {
    if let Ok(proxy) = Proxy::new(
        connection,
        PORTAL_NAME,
        session_handle.as_str(),
        "org.freedesktop.portal.Session",
    )
    .await
        && let Err(error) = proxy.call_method("Close", &()).await
    {
        tracing::debug!(%error, "could not close failed portal session");
    }
}
