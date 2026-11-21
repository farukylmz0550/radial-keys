use crate::{config::{ConfigError, ConfigStore, RadialConfig}, shortcuts::{ActivationMode, ActivationState, InputEvent, Transition}};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use zbus::{fdo, interface};

pub struct Backend {
    config: Arc<RwLock<RadialConfig>>,
    store: ConfigStore,
    activation: Mutex<ActivationState>,
    portal_report: String,
}

#[derive(Serialize)]
struct Status<'a> {
    api_version: u32,
    backend_available: bool,
    menu_available: bool,
    menu_open: bool,
    shortcut: String,
    mode: ActivationMode,
    global_shortcuts: &'a str,
}

impl Backend {
    pub fn new(config: Arc<RwLock<RadialConfig>>, store: ConfigStore, portal_report: String) -> Self {
        let mode = futures_free_mode(&config);
        Self { config, store, activation: Mutex::new(ActivationState::new(mode)), portal_report }
    }

    async fn save_update(&self, changed: RadialConfig) -> fdo::Result<()> {
        changed.validate().map_err(to_dbus_error)?;
        self.store.save(&changed).map_err(to_dbus_error)?;
        *self.config.write().await = changed;
        Ok(())
    }
}

fn futures_free_mode(config: &Arc<RwLock<RadialConfig>>) -> ActivationMode {
    config.try_read().map(|config| config.mode).unwrap_or(ActivationMode::Hold)
}

fn to_dbus_error(error: impl std::fmt::Display) -> fdo::Error {
    fdo::Error::Failed(error.to_string())
}

impl From<ConfigError> for fdo::Error {
    fn from(value: ConfigError) -> Self { to_dbus_error(value) }
}

#[interface(name = "org.radialkeys.Backend1")]
impl Backend {
    #[zbus(name = "GetStatus")]
    async fn get_status(&self) -> fdo::Result<String> {
        let config = self.config.read().await;
        let state = self.activation.lock().await;
        serde_json::to_string(&Status {
            api_version: 1,
            backend_available: true,
            menu_available: true,
            menu_open: state.is_open(),
            shortcut: config.shortcut.clone(),
            mode: config.mode,
            global_shortcuts: &self.portal_report,
        }).map_err(to_dbus_error)
    }

    #[zbus(name = "GetConfiguration")]
    async fn get_configuration(&self) -> fdo::Result<String> {
        toml::to_string_pretty(&*self.config.read().await).map_err(to_dbus_error)
    }

    #[zbus(name = "GetShortcut")]
    async fn get_shortcut(&self) -> String { self.config.read().await.shortcut.clone() }

    #[zbus(name = "SetShortcut")]
    async fn set_shortcut(&self, shortcut: String) -> fdo::Result<()> {
        let mut changed = self.config.read().await.clone();
        changed.shortcut = crate::shortcuts::Shortcut::parse(&shortcut)
            .map_err(to_dbus_error)?.canonical();
        self.activation.lock().await.handle(InputEvent::Escape, &[]);
        self.save_update(changed).await
    }

    #[zbus(name = "SetMode")]
    async fn set_mode(&self, mode: String) -> fdo::Result<()> {
        let mode = match mode.to_ascii_lowercase().as_str() {
            "hold" => ActivationMode::Hold,
            "toggle" => ActivationMode::Toggle,
            _ => return Err(to_dbus_error("mode must be 'hold' or 'toggle'")),
        };
        let mut changed = self.config.read().await.clone();
        changed.mode = mode;
        self.save_update(changed).await?;
        *self.activation.lock().await = ActivationState::new(mode);
        Ok(())
    }

    #[zbus(name = "ShowMenu")]
    async fn show_menu(&self) -> fdo::Result<()> {
        let mut state = self.activation.lock().await;
        state.handle(InputEvent::Toggle, &[]);
        if !state.is_open() { state.handle(InputEvent::Toggle, &[]); }
        Ok(())
    }

    #[zbus(name = "HideMenu")]
    async fn hide_menu(&self) -> fdo::Result<()> {
        let mut state = self.activation.lock().await;
        if state.is_open() { state.handle(InputEvent::Toggle, &[]); }
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
        let actions = config.slots.iter().map(|slot| slot.action.clone()).collect::<Vec<_>>();
        let mut state = self.activation.lock().await;
        match state.handle(InputEvent::Select(index as usize), &actions) {
            Transition::Dispatch(action) => Ok(action),
            Transition::Closed => Ok(String::new()),
            _ => Err(to_dbus_error("menu is closed, slot is invalid, or action was already dispatched")),
        }
    }
}
