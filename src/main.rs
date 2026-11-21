mod config;
mod dbus;
mod portal;
mod shortcuts;

use anyhow::Context;
use config::{ConfigStore, RadialConfig};
use dbus::Backend;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = ConfigStore::default_path()?;
    let config = store.load_or_default()?;
    let shared = Arc::new(RwLock::new(config));
    let backend = Backend::new(shared.clone(), store);
    let connection = zbus::connection::Builder::session()?
        .name("org.radialkeys.Backend1")?
        .serve_at("/org/radialkeys/Backend1", backend)?
        .build()
        .await
        .context("failed to acquire the radial-keys session bus name")?;

    let portal = portal::probe().await;
    eprintln!("radial-keys: {portal}");
    eprintln!("radial-keys: D-Bus service ready on {}", connection.unique_name().unwrap_or_default());

    tokio::signal::ctrl_c().await?;
    Ok(())
}

#[allow(dead_code)]
fn default_config() -> RadialConfig {
    RadialConfig::default()
}
