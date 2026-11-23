use crate::backend::shortcuts::{ActivationState, InputEvent};
use evdev::{Device, EventType, KeyCode, PropType, RelativeAxisCode};
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
    #[error("all compatible mouse devices are busy; close other mouse remapping tools and retry")]
    Busy,
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
        let paths = discover_mice(trigger, button)?;
        let mut selected = None;
        let mut busy = false;
        for path in paths {
            let mut device = Device::open(&path).map_err(|source| MouseError::Open {
                path: path.clone(),
                source,
            })?;
            match device.grab() {
                Ok(()) => {
                    selected = Some((path, device));
                    break;
                }
                Err(source) if source.raw_os_error() == Some(nix::libc::EBUSY) => busy = true,
                Err(source) => return Err(MouseError::Open { path, source }),
            }
        }
        let Some((path, device)) = selected else {
            return if busy {
                Err(MouseError::Busy)
            } else {
                Err(MouseError::NoDevice)
            };
        };
        let name = device.name().unwrap_or("Mouse").to_owned();
        let supported_keys = device.supported_keys().ok_or(MouseError::NoDevice)?;
        let relative_axes = device
            .supported_relative_axes()
            .ok_or(MouseError::NoDevice)?;

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

        // A mouse remapper may already hold the first device. We skip busy
        // devices and relay from the first compatible device we can grab.
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
                let mut state = activation.lock().await;
                if state.is_fullscreen_blocked() {
                    drop(state);
                    self.relay.emit(&[event]).map_err(MouseError::Relay)?;
                } else {
                    match event.value() {
                        1 => {
                            state.handle(InputEvent::Activated, &[]);
                        }
                        0 => {
                            state.handle(InputEvent::Deactivated, &[]);
                        }
                        _ => {} // Ignore kernel key-repeat events.
                    }
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

fn discover_mice(trigger: KeyCode, button: &str) -> Result<Vec<PathBuf>, MouseError> {
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
    let mut matching = Vec::new();
    for path in devices {
        let Ok(device) = Device::open(&path) else {
            continue;
        };
        // Linux input properties: BUTTONPAD (0x02), SEMI_MT (0x03), and
        // POINTING_STICK (0x05) identify devices that should not be grabbed as mice.
        let properties = device.properties();
        let is_touchpad_or_pointing_stick = [PropType(0x02), PropType(0x03), PropType(0x05)]
            .iter()
            .any(|property| properties.contains(*property));
        let is_relative_mouse = !is_touchpad_or_pointing_stick
            && device.supported_keys().is_some_and(|keys| {
                [
                    KeyCode::BTN_LEFT,
                    KeyCode::BTN_RIGHT,
                    KeyCode::BTN_MIDDLE,
                    KeyCode::BTN_SIDE,
                    KeyCode::BTN_EXTRA,
                ]
                .iter()
                .any(|key| keys.contains(*key))
            })
            && device.supported_relative_axes().is_some_and(|axes| {
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
            matching.push(path);
        }
    }

    if !matching.is_empty() {
        Ok(matching)
    } else if found_relative_mouse {
        Err(MouseError::UnsupportedButton(button.to_owned()))
    } else {
        Err(MouseError::NoDevice)
    }
}
