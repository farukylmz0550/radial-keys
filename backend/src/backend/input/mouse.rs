use crate::backend::shortcuts::{ActivationState, InputEvent};
use evdev::{Device, EventType, KeyCode, RelativeAxisCode};
use std::{fs, path::PathBuf, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum MouseError {
    #[error(
        "no usable mouse was found under /dev/input; connect a mouse and check the Radial Keys udev permissions"
    )]
    NoDevice,
    #[error("cannot access mouse input device {path}: {source}")]
    Open {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("the selected mouse button '{0}' is not supported by the detected mouse")]
    UnsupportedButton(String),
    #[error("cannot create the virtual mouse relay: {0}")]
    Relay(std::io::Error),
}

pub struct MouseRunner {
    stream: evdev::EventStream,
    relay: evdev::uinput::VirtualDevice,
    trigger: KeyCode,
    device_name: String,
}

impl MouseRunner {
    pub fn connect(button: &str) -> Result<Self, MouseError> {
        let trigger =
            key_code(button).ok_or_else(|| MouseError::UnsupportedButton(button.to_owned()))?;
        let path = discover_mouse(trigger, button)?;
        let mut device = Device::open(&path).map_err(|source| MouseError::Open {
            path: path.clone(),
            source,
        })?;
        let name = device.name().unwrap_or("Mouse").to_owned();

        let supported_keys = device.supported_keys().ok_or(MouseError::NoDevice)?;
        if !supported_keys.contains(trigger) {
            return Err(MouseError::UnsupportedButton(button.to_owned()));
        }
        let relative_axes = device
            .supported_relative_axes()
            .ok_or(MouseError::NoDevice)?;
        if !relative_axes.contains(RelativeAxisCode::REL_X)
            || !relative_axes.contains(RelativeAxisCode::REL_Y)
        {
            return Err(MouseError::NoDevice);
        }

        let builder = evdev::uinput::VirtualDevice::builder()
            .map_err(MouseError::Relay)?
            .name("Radial Keys mouse relay")
            .with_keys(supported_keys)
            .map_err(MouseError::Relay)?
            .with_relative_axes(relative_axes)
            .map_err(MouseError::Relay)?
            .with_properties(device.properties())
            .map_err(MouseError::Relay)?;
        let builder = if let Some(misc) = device.misc_properties() {
            builder.with_msc(misc).map_err(MouseError::Relay)?
        } else {
            builder
        };
        let relay = builder.build().map_err(MouseError::Relay)?;

        // Grab only the first detected relative mouse. Every non-trigger event
        // is relayed to uinput; dropping this device handle releases the grab.
        device.grab().map_err(|source| MouseError::Open {
            path: path.clone(),
            source,
        })?;
        let stream = match device.into_event_stream() {
            Ok(stream) => stream,
            Err(source) => return Err(MouseError::Open { path, source }),
        };

        Ok(Self {
            stream,
            relay,
            trigger,
            device_name: name,
        })
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub async fn run(mut self, activation: Arc<Mutex<ActivationState>>) -> Result<(), MouseError> {
        loop {
            let event = self
                .stream
                .next_event()
                .await
                .map_err(|source| MouseError::Open {
                    path: self
                        .stream
                        .device()
                        .physical_path()
                        .map(PathBuf::from)
                        .unwrap_or_default(),
                    source,
                })?;

            if event.event_type() == EventType::KEY && event.code() == self.trigger.code() {
                match event.value() {
                    1 => {
                        activation.lock().await.handle(InputEvent::Activated, &[]);
                    }
                    0 => {
                        activation.lock().await.handle(InputEvent::Deactivated, &[]);
                    }
                    _ => {} // Ignore kernel key-repeat events.
                }
            } else if event.event_type() != EventType::SYNCHRONIZATION {
                self.relay.emit(&[event]).map_err(MouseError::Relay)?;
            }
        }
    }
}

fn key_code(button: &str) -> Option<KeyCode> {
    match button {
        "left" => Some(KeyCode::BTN_LEFT),
        "right" => Some(KeyCode::BTN_RIGHT),
        "middle" => Some(KeyCode::BTN_MIDDLE),
        "back" => Some(KeyCode::BTN_SIDE),
        "forward" => Some(KeyCode::BTN_EXTRA),
        _ => None,
    }
}

fn discover_mouse(trigger: KeyCode, button: &str) -> Result<PathBuf, MouseError> {
    let mut devices = fs::read_dir("/dev/input")
        .map_err(|_| MouseError::NoDevice)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("event"))
        })
        .collect::<Vec<_>>();
    devices.sort();

    let mut found_relative_mouse = false;
    for path in devices {
        let Ok(device) = Device::open(&path) else {
            continue;
        };
        let is_relative_mouse = device.supported_keys().is_some_and(|keys| {
            [
                KeyCode::BTN_LEFT,
                KeyCode::BTN_RIGHT,
                KeyCode::BTN_MIDDLE,
                KeyCode::BTN_SIDE,
                KeyCode::BTN_EXTRA,
            ]
            .iter()
            .any(|key| keys.contains(*key))
        }) && device.supported_relative_axes().is_some_and(|axes| {
            axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
        });
        if !is_relative_mouse {
            continue;
        }
        found_relative_mouse = true;
        if device
            .supported_keys()
            .is_some_and(|keys| keys.contains(trigger))
        {
            return Ok(path);
        }
    }

    if found_relative_mouse {
        Err(MouseError::UnsupportedButton(button.to_owned()))
    } else {
        Err(MouseError::NoDevice)
    }
}
