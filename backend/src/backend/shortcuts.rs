use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActivationMode {
    Hold,
    #[default]
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: Vec<String>,
    key: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShortcutError {
    #[error("shortcut is empty")]
    Empty,
    #[error("shortcut contains an empty or duplicate component")]
    Duplicate,
    #[error("unsupported modifier '{0}'")]
    Modifier(String),
    #[error("unsupported key '{0}'")]
    Key(String),
    #[error("a shortcut needs exactly one non-modifier key")]
    KeyCount,
}

impl Shortcut {
    pub fn parse(input: &str) -> Result<Self, ShortcutError> {
        if input.trim().is_empty() {
            return Err(ShortcutError::Empty);
        }
        let mut modifiers = HashSet::new();
        let mut key = None;
        for part in input.split('+') {
            let part = part.trim();
            if part.is_empty() {
                return Err(ShortcutError::Duplicate);
            }
            let lower = part.to_ascii_lowercase();
            if let Some(modifier) = match lower.as_str() {
                "ctrl" | "control" => Some("Ctrl"),
                "leftctrl" | "ctrlleft" | "leftcontrol" | "controlleft" => Some("CtrlLeft"),
                "rightctrl" | "ctrlright" | "rightcontrol" | "controlright" => Some("CtrlRight"),
                "alt" => Some("Alt"),
                "leftalt" | "altleft" => Some("AltLeft"),
                "rightalt" | "altright" => Some("AltRight"),
                "shift" => Some("Shift"),
                "leftshift" | "shiftleft" => Some("ShiftLeft"),
                "rightshift" | "shiftright" => Some("ShiftRight"),
                "super" | "meta" | "win" => Some("Super"),
                "leftsuper" | "superleft" | "leftmeta" | "metaleft" => Some("SuperLeft"),
                "rightsuper" | "superright" | "rightmeta" | "metaright" => Some("SuperRight"),
                _ => None,
            } {
                if !modifiers.insert(modifier) {
                    return Err(ShortcutError::Duplicate);
                }
                continue;
            }
            if is_modifier_spelling(&lower) {
                return Err(ShortcutError::Modifier(part.into()));
            }
            if key.is_some() {
                return Err(ShortcutError::KeyCount);
            }
            key = Some(canonical_key(part)?);
        }
        let key = key.ok_or(ShortcutError::KeyCount)?;
        let mut ordered = [
            "Ctrl",
            "CtrlLeft",
            "CtrlRight",
            "Alt",
            "AltLeft",
            "AltRight",
            "Shift",
            "ShiftLeft",
            "ShiftRight",
            "Super",
            "SuperLeft",
            "SuperRight",
        ]
        .into_iter()
        .filter(|modifier| modifiers.contains(modifier))
        .map(str::to_owned)
        .collect::<Vec<_>>();
        ordered.push(key);
        Ok(Self {
            modifiers: ordered[..ordered.len() - 1].to_vec(),
            key: ordered.pop().unwrap(),
        })
    }

    pub fn canonical(&self) -> String {
        self.modifiers
            .iter()
            .cloned()
            .chain(std::iter::once(self.key.clone()))
            .collect::<Vec<_>>()
            .join("+")
    }

    pub fn portal_trigger(&self) -> Result<String, ShortcutError> {
        let mut parts = Vec::with_capacity(self.modifiers.len() + 1);
        for modifier in &self.modifiers {
            parts.push(
                match modifier.as_str() {
                    "Ctrl" => "CTRL",
                    "Alt" => "ALT",
                    "Shift" => "SHIFT",
                    "Super" => "LOGO",
                    _ => return Err(ShortcutError::Modifier(modifier.clone())),
                }
                .to_owned(),
            );
        }
        let key = match self.key.as_str() {
            "Space" => "space",
            "Enter" => "Return",
            "Backspace" => "BackSpace",
            "PageUp" => "Page_Up",
            "PageDown" => "Page_Down",
            "Plus" => "plus",
            "Minus" => "minus",
            value if value.len() == 1 && value.as_bytes()[0].is_ascii_alphabetic() => {
                return Ok(parts
                    .into_iter()
                    .chain(std::iter::once(value.to_ascii_lowercase()))
                    .collect::<Vec<_>>()
                    .join("+"));
            }
            value => value,
        };
        parts.push(key.to_owned());
        Ok(parts.join("+"))
    }
}

fn is_modifier_spelling(value: &str) -> bool {
    [
        "ctrl",
        "control",
        "alt",
        "shift",
        "super",
        "meta",
        "win",
        "hyper",
        "mod1",
        "mod2",
        "mod3",
        "mod4",
        "mod5",
        "leftctrl",
        "rightctrl",
        "ctrlleft",
        "ctrlright",
        "leftcontrol",
        "rightcontrol",
        "controlleft",
        "controlright",
        "leftalt",
        "rightalt",
        "altleft",
        "altright",
        "leftshift",
        "rightshift",
        "shiftleft",
        "shiftright",
        "leftsuper",
        "rightsuper",
        "superleft",
        "superright",
        "leftmeta",
        "rightmeta",
        "metaleft",
        "metaright",
    ]
    .contains(&value)
}

fn canonical_key(key: &str) -> Result<String, ShortcutError> {
    let upper = key.to_ascii_uppercase();
    if upper.len() == 1 && upper.as_bytes()[0].is_ascii_alphanumeric() {
        return Ok(upper);
    }
    let canonical = match key.to_ascii_lowercase().as_str() {
        "space" => "Space",
        "escape" | "esc" => "Escape",
        "enter" | "return" => "Enter",
        "tab" => "Tab",
        "backspace" => "Backspace",
        "delete" => "Delete",
        "up" => "Up",
        "down" => "Down",
        "left" => "Left",
        "right" => "Right",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "insert" => "Insert",
        "plus" => "Plus",
        "minus" => "Minus",
        other
            if other.len() >= 2
                && other.starts_with('f')
                && other[1..]
                    .parse::<u8>()
                    .is_ok_and(|n| (1..=24).contains(&n)) =>
        {
            return Ok(other.to_ascii_uppercase());
        }
        _ => return Err(ShortcutError::Key(key.into())),
    };
    if canonical == "Escape" {
        return Err(ShortcutError::Key("Escape (reserved for cancel)".into()));
    }
    Ok(canonical.into())
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Input source integration is intentionally pending; the state machine is tested independently.
pub enum InputEvent {
    Activated,
    Deactivated,
    Escape,
    Select(usize),
    DeadZone,
    Show,
    Hide,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {
    None,
    Opened,
    Closed,
    Cancelled,
    Dispatch(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationState {
    mode: ActivationMode,
    open: bool,
    active: bool,
    dispatched: bool,
}

impl ActivationState {
    pub fn new(mode: ActivationMode) -> Self {
        Self {
            mode,
            open: false,
            active: false,
            dispatched: false,
        }
    }
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn handle(&mut self, event: InputEvent, actions: &[String]) -> Transition {
        match event {
            InputEvent::Activated => {
                if self.active {
                    return Transition::None;
                }
                self.active = true;
                self.dispatched = false;
                match self.mode {
                    ActivationMode::Hold if !self.open => {
                        self.open = true;
                        Transition::Opened
                    }
                    ActivationMode::Toggle => {
                        self.open = !self.open;
                        if self.open {
                            Transition::Opened
                        } else {
                            Transition::Closed
                        }
                    }
                    _ => Transition::None,
                }
            }
            InputEvent::Deactivated => {
                if !self.active {
                    return Transition::None;
                }
                self.active = false;
                if self.mode == ActivationMode::Hold && self.open {
                    self.open = false;
                    Transition::Closed
                } else {
                    Transition::None
                }
            }
            InputEvent::Escape | InputEvent::DeadZone => {
                if !self.open {
                    return Transition::None;
                }
                self.open = false;
                self.active = false;
                self.dispatched = true;
                Transition::Cancelled
            }
            InputEvent::Show => {
                if self.open {
                    Transition::None
                } else {
                    self.open = true;
                    self.dispatched = false;
                    Transition::Opened
                }
            }
            InputEvent::Hide => {
                if self.open {
                    self.open = false;
                    Transition::Closed
                } else {
                    Transition::None
                }
            }
            InputEvent::Toggle => {
                self.active = false;
                if self.open {
                    self.open = false;
                    self.dispatched = true;
                    Transition::Closed
                } else {
                    self.open = true;
                    self.dispatched = false;
                    Transition::Opened
                }
            }
            InputEvent::Select(index) => {
                if !self.open || self.dispatched {
                    return Transition::None;
                }
                self.open = false;
                self.dispatched = true;
                actions
                    .get(index)
                    .filter(|action| !action.is_empty())
                    .cloned()
                    .map(Transition::Dispatch)
                    .unwrap_or(Transition::Closed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_canonicalizes_modifier_combinations() {
        assert_eq!(
            Shortcut::parse("shift+ctrl+space").unwrap().canonical(),
            "Ctrl+Shift+Space"
        );
        assert_eq!(Shortcut::parse("x").unwrap().canonical(), "X");
        assert_eq!(
            Shortcut::parse("RightCtrl+LeftAlt+x").unwrap().canonical(),
            "CtrlRight+AltLeft+X"
        );
    }

    #[test]
    fn converts_supported_shortcuts_to_portal_trigger_names() {
        assert_eq!(Shortcut::parse("x").unwrap().portal_trigger().unwrap(), "x");
        assert_eq!(
            Shortcut::parse("Ctrl+Shift+Y")
                .unwrap()
                .portal_trigger()
                .unwrap(),
            "CTRL+SHIFT+y"
        );
        assert_eq!(
            Shortcut::parse("Ctrl+Alt+X")
                .unwrap()
                .portal_trigger()
                .unwrap(),
            "CTRL+ALT+x"
        );
        assert_eq!(
            Shortcut::parse("Ctrl+PageUp")
                .unwrap()
                .portal_trigger()
                .unwrap(),
            "CTRL+Page_Up"
        );
        assert!(
            Shortcut::parse("RightCtrl+X")
                .unwrap()
                .portal_trigger()
                .is_err()
        );
    }
    #[test]
    fn rejects_invalid_keys_and_combinations() {
        for input in [
            "",
            "Ctrl++A",
            "Ctrl+Ctrl+A",
            "Ctrl+Escape",
            "Ctrl",
            "Foo+A",
            "A+B",
        ] {
            assert!(Shortcut::parse(input).is_err(), "accepted {input}");
        }
    }
    #[test]
    fn hold_activation_repeats_cancel_and_dispatch_once() {
        let mut state = ActivationState::new(ActivationMode::Hold);
        let actions = vec!["open-app".to_owned()];
        assert_eq!(
            state.handle(InputEvent::Activated, &actions),
            Transition::Opened
        );
        assert_eq!(
            state.handle(InputEvent::Activated, &actions),
            Transition::None
        );
        assert_eq!(
            state.handle(InputEvent::Select(0), &actions),
            Transition::Dispatch("open-app".into())
        );
        assert_eq!(
            state.handle(InputEvent::Select(0), &actions),
            Transition::None
        );
        assert_eq!(
            state.handle(InputEvent::Deactivated, &actions),
            Transition::None
        );
    }
    #[test]
    fn hold_release_and_cancel_close_menu() {
        let mut state = ActivationState::new(ActivationMode::Hold);
        let actions = vec![];
        state.handle(InputEvent::Activated, &actions);
        assert_eq!(
            state.handle(InputEvent::Deactivated, &actions),
            Transition::Closed
        );
        state.handle(InputEvent::Activated, &actions);
        assert_eq!(
            state.handle(InputEvent::Escape, &actions),
            Transition::Cancelled
        );
        assert_eq!(
            state.handle(InputEvent::DeadZone, &actions),
            Transition::None
        );
    }
    #[test]
    fn toggle_activation_changes_only_on_edges() {
        let mut state = ActivationState::new(ActivationMode::Toggle);
        assert_eq!(state.handle(InputEvent::Activated, &[]), Transition::Opened);
        assert_eq!(state.handle(InputEvent::Activated, &[]), Transition::None);
        assert_eq!(state.handle(InputEvent::Deactivated, &[]), Transition::None);
        assert_eq!(state.handle(InputEvent::Activated, &[]), Transition::Closed);
    }

    #[test]
    fn direct_toggle_can_open_and_close_without_a_release_event() {
        let mut state = ActivationState::new(ActivationMode::Toggle);
        assert_eq!(state.handle(InputEvent::Toggle, &[]), Transition::Opened);
        assert!(state.is_open());
        assert_eq!(state.handle(InputEvent::Toggle, &[]), Transition::Closed);
        assert!(!state.is_open());
    }
}
