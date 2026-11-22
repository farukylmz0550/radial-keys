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

    let unique_name = connection
        .unique_name()
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown".into());
    tracing::info!(portal = %portal, unique_name, "Radial Keys backend ready");

    tokio::signal::ctrl_c().await?;
    macro_processes.terminate_all().await;
    Ok(())
}
