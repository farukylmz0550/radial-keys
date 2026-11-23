mod backend;

use anyhow::Context;
use backend::{config::ConfigStore, dbus::Backend};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let store = ConfigStore::default_path()?;
    let config = store.load_or_default()?;
    let shared = Arc::new(RwLock::new(config));
    let portal = backend::portal::probe().await;
    let backend = Backend::new(shared.clone(), store, portal.clone())?;
    if let Err(error) = backend.start_configured_activation().await {
        tracing::warn!(%error, "could not restore configured global shortcut registration");
    }
    let macro_processes = backend.macro_process_manager();
    let connection = zbus::connection::Builder::session()?
        .name("io.github.radialkeys.Service")?
        .serve_at("/io/github/radialkeys/Service", backend)?
        .build()
        .await
        .context("failed to acquire the radial-keys session bus name")?;

    if let Err(error) = crate::backend::kwin::install_fullscreen_monitor(&connection).await {
        tracing::warn!(%error, "could not enable fullscreen activation blocking");
    }

    let unique_name = connection
        .unique_name()
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown".into());
    tracing::info!(portal = %portal, unique_name, "Radial Keys backend ready");

    let mut terminate_signal =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => result?,
        _ = terminate_signal.recv() => {},
    }
    crate::backend::kwin::unload_fullscreen_monitor(&connection).await;
    macro_processes.terminate_all().await;
    Ok(())
}
