use super::{
    config::{ConfigError, ConfigStore, RadialConfig},
    portal::{self, PortalSession},
    shortcuts::{ActivationMode, ActivationState, InputEvent, Transition},
};
use serde::Serialize;
use std::path::Path;
use std::{sync::Arc, time::Instant};
use tokio::{
    process::Command,
    sync::{Mutex, RwLock},
};
use zbus::{fdo, interface};

pub struct Backend {
    config: Arc<RwLock<RadialConfig>>,
    store: ConfigStore,
    activation: Arc<Mutex<ActivationState>>,
    portal_session: Mutex<Option<PortalSession>>,
    mouse_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    mouse_status: Arc<RwLock<String>>,
    portal_operation: Mutex<()>,
    renderer_last_seen: Mutex<Option<Instant>>,
    cursor_position: Mutex<Option<(i32, i32)>>,
    portal_report: String,
}

#[derive(Serialize)]
struct Status<'a> {
    api_version: u32,
    backend_available: bool,
    renderer_available: bool,
    menu_open: bool,
    shortcut: String,
    mode: ActivationMode,
    activation_type: super::config::ActivationType,
    mouse_button: Option<String>,
    global_shortcuts: &'a str,
    global_activation_enabled: bool,
    global_shortcut_registered: bool,
    mouse_helper_status: String,
    cursor_x: Option<i32>,
    cursor_y: Option<i32>,
}

impl Backend {
    pub fn new(
        config: Arc<RwLock<RadialConfig>>,
        store: ConfigStore,
        portal_report: String,
    ) -> Self {
        let mode = futures_free_mode(&config);
        Self {
            config,
            store,
            activation: Arc::new(Mutex::new(ActivationState::new(mode))),
            portal_session: Mutex::new(None),
            mouse_task: Mutex::new(None),
            mouse_status: Arc::new(RwLock::new("Disabled".into())),
            portal_operation: Mutex::new(()),
            renderer_last_seen: Mutex::new(None),
            cursor_position: Mutex::new(None),
            portal_report,
        }
    }

    pub async fn start_configured_activation(&self) -> fdo::Result<()> {
        if self.config.read().await.activation.enabled {
            self.set_global_activation_enabled(true).await?;
        }
        Ok(())
    }

    async fn set_global_activation_enabled(&self, enabled: bool) -> fdo::Result<()> {
        let _operation = self.portal_operation.lock().await;
        let current = self.config.read().await.clone();
        if current.activation.input_type == super::config::ActivationType::Mouse {
            if enabled {
                self.stop_mouse_helper().await;
                let button = current
                    .activation
                    .mouse_button
                    .as_deref()
                    .ok_or_else(|| to_dbus_error("select a mouse button first"))?;
                let runner = match super::input::mouse::MouseRunner::connect(button) {
                    Ok(runner) => runner,
                    Err(error) => {
                        let message = error.to_string();
                        *self.mouse_status.write().await = message.clone();
                        let mut changed = current;
                        changed.activation.enabled = false;
                        self.save_update(changed).await?;
                        return Err(to_dbus_error(message));
                    }
                };
                let device_name = runner.device_name().to_owned();
                let button = button.to_owned();
                let mut changed = current;
                changed.activation.enabled = true;
                self.save_update(changed).await?;

                *self.mouse_status.write().await = format!("Active: {device_name}");
                let activation = self.activation.clone();
                let status = self.mouse_status.clone();
                *self.mouse_task.lock().await = Some(tokio::spawn(async move {
                    let mut runner = Some(runner);
                    loop {
                        let current_runner = match runner.take() {
                            Some(runner) => runner,
                            None => match super::input::mouse::MouseRunner::connect(&button) {
                                Ok(runner) => {
                                    *status.write().await =
                                        format!("Active: {}", runner.device_name());
                                    runner
                                }
                                Err(error) => {
                                    *status.write().await = error.to_string();
                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    continue;
                                }
                            },
                        };
                        if let Err(error) = current_runner.run(activation.clone()).await {
                            tracing::warn!(%error, "mouse activation helper stopped; waiting for a usable mouse");
                            *status.write().await = error.to_string();
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }));
                return Ok(());
            }

            let mut changed = current;
            changed.activation.enabled = false;
            self.save_update(changed).await?;
            self.stop_mouse_helper().await;
            return Ok(());
        }

        if enabled {
            let expired = {
                let mut active_session = self.portal_session.lock().await;
                if active_session
                    .as_ref()
                    .is_some_and(PortalSession::is_running)
                {
                    return Ok(());
                }
                active_session.take()
            };
            if let Some(expired) = expired {
                expired.close().await;
            }
            let session =
                portal::register_shortcut(&current.activation.shortcut(), self.activation.clone())
                    .await
                    .map_err(to_dbus_error)?;
            let mut changed = current;
            changed.activation.enabled = true;
            if let Err(error) = self.save_update(changed).await {
                session.close().await;
                return Err(error);
            }
            *self.portal_session.lock().await = Some(session);
            Ok(())
        } else {
            let mut changed = current;
            changed.activation.enabled = false;
            self.save_update(changed).await?;
            let session = self.portal_session.lock().await.take();
            if let Some(session) = session {
                session.close().await;
            }
            Ok(())
        }
    }

    async fn stop_mouse_helper(&self) {
        if let Some(task) = self.mouse_task.lock().await.take() {
            task.abort();
            let _ = task.await;
        }
        *self.mouse_status.write().await = "Disabled".into();
    }

    async fn save_update(&self, changed: RadialConfig) -> fdo::Result<()> {
        changed.validate().map_err(to_dbus_error)?;
        self.store.save(&changed).map_err(to_dbus_error)?;
        *self.config.write().await = changed;
        Ok(())
    }

    async fn run_python_action(&self, index: usize) -> fdo::Result<String> {
        let (label, script) = {
            let config = self.config.read().await;
            let slot = config
                .slots
                .get(index)
                .ok_or_else(|| to_dbus_error("slot index must be between 0 and 7"))?;
            (slot.label.clone(), slot.action.clone())
        };
        if script.is_empty() {
            return Ok(label);
        }
        let path = Path::new(&script);
        if !path.is_absolute() || path.extension().is_none_or(|extension| extension != "py") {
            return Err(to_dbus_error(
                "macro path must be an absolute .py file path",
            ));
        }
        if !path.is_file() {
            return Err(to_dbus_error(format!(
                "Python macro does not exist: {script}"
            )));
        }

        let mut child = Command::new("python3")
            .arg(path)
            .spawn()
            .map_err(|error| to_dbus_error(format!("could not start Python macro: {error}")))?;
        let script_for_log = script.clone();
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) if status.success() => {
                    tracing::info!(script = %script_for_log, "Python macro completed");
                }
                Ok(status) => {
                    tracing::warn!(script = %script_for_log, %status, "Python macro exited with an error");
                }
                Err(error) => {
                    tracing::warn!(script = %script_for_log, %error, "could not wait for Python macro");
                }
            }
        });
        tracing::info!(slot = index + 1, script, "started Python macro");
        Ok(label)
    }

    async fn update_slot(
        &self,
        index: u32,
        label: String,
        script: String,
        icon: Option<String>,
    ) -> fdo::Result<()> {
        let index = index as usize;
        if index >= 8 {
            return Err(to_dbus_error("slot index must be between 0 and 7"));
        }
        if label.trim().is_empty() || label.chars().count() > 32 {
            return Err(to_dbus_error("slot label must contain 1 to 32 characters"));
        }
        if script.len() > 4096 {
            return Err(to_dbus_error("Python macro path is too long"));
        }
        if !script.is_empty() {
            let path = Path::new(&script);
            if !path.is_absolute() || path.extension().is_none_or(|extension| extension != "py") {
                return Err(to_dbus_error("choose an absolute path to a .py file"));
            }
            if !path.is_file() {
                return Err(to_dbus_error("selected Python macro file does not exist"));
            }
        }
        if let Some(icon_path) = icon.as_deref().filter(|path| !path.is_empty()) {
            let path = Path::new(icon_path);
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !path.is_absolute() || !matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "svg")
            {
                return Err(to_dbus_error(
                    "choose an absolute PNG, JPEG, or SVG icon path",
                ));
            }
            if !path.is_file() {
                return Err(to_dbus_error("selected icon file does not exist"));
            }
        }

        let mut changed = self.config.read().await.clone();
        let slot = changed
            .slots
            .get_mut(index)
            .ok_or_else(|| to_dbus_error("slot index must be between 0 and 7"))?;
        slot.label = label.trim().to_owned();
        slot.action = script;
        if let Some(icon_path) = icon {
            slot.icon = icon_path;
        }
        self.save_update(changed).await
    }
}

fn futures_free_mode(config: &Arc<RwLock<RadialConfig>>) -> ActivationMode {
    config
        .try_read()
        .map(|config| config.activation.mode)
        .unwrap_or(ActivationMode::Toggle)
}

fn to_dbus_error(error: impl std::fmt::Display) -> fdo::Error {
    fdo::Error::Failed(error.to_string())
}

impl From<ConfigError> for fdo::Error {
    fn from(value: ConfigError) -> Self {
        to_dbus_error(value)
    }
}

#[interface(name = "io.github.radialkeys.Service1")]
impl Backend {
    #[zbus(name = "GetStatus")]
    async fn get_status(&self) -> fdo::Result<String> {
        let config = self.config.read().await;
        let portal_activation_enabled = config.activation.enabled;
        let shortcut = config.activation.shortcut();
        let mode = config.activation.mode;
        let activation_type = config.activation.input_type;
        let mouse_button = config.activation.mouse_button.clone();
        drop(config);
        let state = self.activation.lock().await;
        let renderer_available = self
            .renderer_last_seen
            .lock()
            .await
            .is_some_and(|last_seen| last_seen.elapsed().as_secs() < 3);
        let global_shortcut_registered = self
            .portal_session
            .lock()
            .await
            .as_ref()
            .is_some_and(PortalSession::is_running);
        let mouse_helper_status = self.mouse_status.read().await.clone();
        let cursor_position = *self.cursor_position.lock().await;
        serde_json::to_string(&Status {
            api_version: 1,
            backend_available: true,
            renderer_available,
            menu_open: state.is_open(),
            shortcut,
            mode,
            activation_type,
            mouse_button,
            global_shortcuts: &self.portal_report,
            global_activation_enabled: portal_activation_enabled,
            global_shortcut_registered,
            mouse_helper_status,
            cursor_x: cursor_position.map(|position| position.0),
            cursor_y: cursor_position.map(|position| position.1),
        })
        .map_err(to_dbus_error)
    }

    #[zbus(name = "Ping")]
    async fn ping(&self) -> String {
        "pong".into()
    }

    #[zbus(name = "GetVersion")]
    async fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    #[zbus(name = "RendererHeartbeat")]
    async fn renderer_heartbeat(&self) {
        *self.renderer_last_seen.lock().await = Some(Instant::now());
    }

    #[zbus(name = "RendererDisconnected")]
    async fn renderer_disconnected(&self) {
        *self.renderer_last_seen.lock().await = None;
    }

    #[zbus(name = "SetCursorPosition")]
    async fn set_cursor_position(&self, x: i32, y: i32) {
        *self.cursor_position.lock().await = Some((x, y));
    }

    #[zbus(name = "GetConfiguration")]
    async fn get_configuration(&self) -> fdo::Result<String> {
        toml::to_string_pretty(&*self.config.read().await).map_err(to_dbus_error)
    }

    #[zbus(name = "GetSlots")]
    async fn get_slots(&self) -> fdo::Result<String> {
        serde_json::to_string(&self.config.read().await.slots).map_err(to_dbus_error)
    }

    #[zbus(name = "SetSlot")]
    async fn set_slot(&self, index: u32, label: String, script: String) -> fdo::Result<()> {
        self.update_slot(index, label, script, None).await
    }

    #[zbus(name = "SetSlotDetails")]
    async fn set_slot_details(
        &self,
        index: u32,
        label: String,
        script: String,
        icon: String,
    ) -> fdo::Result<()> {
        self.update_slot(index, label, script, Some(icon)).await
    }

    #[zbus(name = "GetActivationShortcut")]
    async fn get_activation_shortcut(&self) -> String {
        self.config.read().await.activation.shortcut()
    }

    #[zbus(name = "SetActivationShortcut")]
    async fn set_activation_shortcut(&self, shortcut: String) -> fdo::Result<()> {
        let _operation = self.portal_operation.lock().await;
        let mut changed = self.config.read().await.clone();
        let previous_input_type = changed.activation.input_type;
        changed
            .activation
            .set_shortcut(&shortcut)
            .map_err(to_dbus_error)?;
        let previous_shortcut = self.config.read().await.activation.shortcut();
        let replacement = if changed.activation.enabled
            && changed.activation.shortcut() != previous_shortcut
        {
            Some(
                portal::register_shortcut(&changed.activation.shortcut(), self.activation.clone())
                    .await
                    .map_err(to_dbus_error)?,
            )
        } else {
            None
        };
        if let Err(error) = self.save_update(changed).await {
            if let Some(session) = replacement {
                session.close().await;
            }
            return Err(error);
        }
        if let Some(replacement) = replacement {
            let previous = self.portal_session.lock().await.replace(replacement);
            if let Some(previous) = previous {
                previous.close().await;
            }
        }
        if previous_input_type == super::config::ActivationType::Mouse {
            self.stop_mouse_helper().await;
        }
        let mode = self.config.read().await.activation.mode;
        *self.activation.lock().await = ActivationState::new(mode);
        Ok(())
    }

    #[zbus(name = "SetGlobalActivationEnabled")]
    async fn set_global_activation_enabled_dbus(&self, enabled: bool) -> fdo::Result<()> {
        self.set_global_activation_enabled(enabled).await
    }

    #[zbus(name = "SetMode")]
    async fn set_mode(&self, mode: String) -> fdo::Result<()> {
        let mode = match mode.to_ascii_lowercase().as_str() {
            "hold" => ActivationMode::Hold,
            "toggle" => ActivationMode::Toggle,
            _ => return Err(to_dbus_error("mode must be 'hold' or 'toggle'")),
        };
        let mut changed = self.config.read().await.clone();
        changed.activation.mode = mode;
        self.save_update(changed).await?;
        *self.activation.lock().await = ActivationState::new(mode);
        Ok(())
    }

    #[zbus(name = "SetActivationDevice")]
    async fn set_activation_device(&self, device: String, button: String) -> fdo::Result<()> {
        let device = match device.to_ascii_lowercase().as_str() {
            "keyboard" => super::config::ActivationType::Keyboard,
            "mouse" => super::config::ActivationType::Mouse,
            _ => return Err(to_dbus_error("device must be 'keyboard' or 'mouse'")),
        };
        let button = match button.to_ascii_lowercase().as_str() {
            "left" | "right" | "middle" | "back" | "forward" => button.to_ascii_lowercase(),
            _ => {
                return Err(to_dbus_error(
                    "mouse button must be left, right, middle, back, or forward",
                ));
            }
        };

        let _operation = self.portal_operation.lock().await;
        let mut changed = self.config.read().await.clone();
        changed.activation.input_type = device;
        changed.activation.mouse_button = if device == super::config::ActivationType::Mouse {
            Some(button)
        } else {
            None
        };
        let was_enabled = changed.activation.enabled;
        changed.activation.enabled = false;
        self.save_update(changed).await?;
        self.stop_mouse_helper().await;
        if was_enabled && let Some(session) = self.portal_session.lock().await.take() {
            session.close().await;
        }
        Ok(())
    }

    #[zbus(name = "ShowMenu")]
    async fn show_menu(&self) -> fdo::Result<()> {
        self.activation.lock().await.handle(InputEvent::Show, &[]);
        Ok(())
    }

    #[zbus(name = "HideMenu")]
    async fn hide_menu(&self) -> fdo::Result<()> {
        self.activation.lock().await.handle(InputEvent::Hide, &[]);
        Ok(())
    }

    #[zbus(name = "CancelMenu")]
    async fn cancel_menu(&self) -> fdo::Result<()> {
        self.activation.lock().await.handle(InputEvent::Escape, &[]);
        Ok(())
    }

    #[zbus(name = "SelectSlot")]
    async fn select_slot(&self, index: u32) -> fdo::Result<String> {
        let config = self.config.read().await;
        let actions = config
            .slots
            .iter()
            .map(|slot| slot.action.clone())
            .collect::<Vec<_>>();
        let mut state = self.activation.lock().await;
        match state.handle(InputEvent::Select(index as usize), &actions) {
            Transition::Dispatch(action) => Ok(action),
            Transition::Closed => Ok(String::new()),
            _ => Err(to_dbus_error(
                "menu is closed, slot is invalid, or action was already dispatched",
            )),
        }
    }

    #[zbus(name = "ActivateSlot")]
    async fn activate_slot(&self, index: u32) -> fdo::Result<String> {
        let label = self.run_python_action(index as usize).await?;
        let mut state = self.activation.lock().await;
        state.handle(InputEvent::Hide, &[]);
        Ok(label)
    }
}
