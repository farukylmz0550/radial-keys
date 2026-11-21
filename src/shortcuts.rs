use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActivationMode { Hold, Toggle }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut { modifiers: Vec<String>, key: String }

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
        if input.trim().is_empty() { return Err(ShortcutError::Empty); }
        let mut modifiers = HashSet::new();
        let mut key = None;
        for part in input.split('+') {
            let part = part.trim();
            if part.is_empty() { return Err(ShortcutError::Duplicate); }
            let lower = part.to_ascii_lowercase();
            if let Some(modifier) = match lower.as_str() {
                "ctrl" | "control" => Some("Ctrl"),
                "alt" => Some("Alt"),
                "shift" => Some("Shift"),
                "super" | "meta" | "win" => Some("Super"),
                _ => None,
            } {
                if !modifiers.insert(modifier) { return Err(ShortcutError::Duplicate); }
                continue;
            }
            if is_modifier_spelling(&lower) { return Err(ShortcutError::Modifier(part.into())); }
            if key.is_some() { return Err(ShortcutError::KeyCount); }
            key = Some(canonical_key(part)?);
        }
        let key = key.ok_or(ShortcutError::KeyCount)?;
        let mut ordered = ["Ctrl", "Alt", "Shift", "Super"].into_iter()
            .filter(|modifier| modifiers.contains(modifier)).map(str::to_owned).collect::<Vec<_>>();
        ordered.push(key);
        Ok(Self { modifiers: ordered[..ordered.len()-1].to_vec(), key: ordered.pop().unwrap() })
    }

    pub fn canonical(&self) -> String {
        self.modifiers.iter().cloned().chain(std::iter::once(self.key.clone())).collect::<Vec<_>>().join("+")
    }
}

fn is_modifier_spelling(value: &str) -> bool {
    ["ctrl", "control", "alt", "shift", "super", "meta", "win", "hyper", "mod1", "mod2", "mod3", "mod4", "mod5"].contains(&value)
}

fn canonical_key(key: &str) -> Result<String, ShortcutError> {
    let upper = key.to_ascii_uppercase();
    if upper.len() == 1 && upper.as_bytes()[0].is_ascii_alphanumeric() { return Ok(upper); }
    let canonical = match key.to_ascii_lowercase().as_str() {
        "space" => "Space", "escape" | "esc" => "Escape", "enter" | "return" => "Enter",
        "tab" => "Tab", "backspace" => "Backspace", "delete" => "Delete",
        "up" => "Up", "down" => "Down", "left" => "Left", "right" => "Right",
        "home" => "Home", "end" => "End", "pageup" => "PageUp", "pagedown" => "PageDown",
        "insert" => "Insert", "plus" => "Plus", "minus" => "Minus",
        other if other.len() >= 2 && other.starts_with('f') && other[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)) => {
            return Ok(other.to_ascii_uppercase())
        }
        _ => return Err(ShortcutError::Key(key.into())),
    };
    if canonical == "Escape" { return Err(ShortcutError::Key("Escape (reserved for cancel)".into())); }
    Ok(canonical.into())
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.canonical()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent { Activated, Deactivated, Escape, Select(usize), DeadZone, Toggle }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition { None, Opened, Closed, Cancelled, Dispatch(String) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationState { mode: ActivationMode, open: bool, active: bool, dispatched: bool }

impl ActivationState {
    pub fn new(mode: ActivationMode) -> Self { Self { mode, open: false, active: false, dispatched: false } }
    pub fn is_open(&self) -> bool { self.open }

    pub fn handle(&mut self, event: InputEvent, actions: &[String]) -> Transition {
        match event {
            InputEvent::Activated => {
                if self.active { return Transition::None; }
                self.active = true;
                self.dispatched = false;
                match self.mode {
                    ActivationMode::Hold if !self.open => { self.open = true; Transition::Opened }
                    ActivationMode::Toggle => { self.open = !self.open; if self.open { Transition::Opened } else { Transition::Closed } }
                    _ => Transition::None,
                }
            }
            InputEvent::Deactivated => {
                if !self.active { return Transition::None; }
                self.active = false;
                if self.mode == ActivationMode::Hold && self.open { self.open = false; Transition::Closed } else { Transition::None }
            }
            InputEvent::Escape | InputEvent::DeadZone => {
                if !self.open { return Transition::None; }
                self.open = false;
                self.active = false;
                self.dispatched = true;
                Transition::Cancelled
            }
            InputEvent::Toggle => {
                self.open = !self.open;
                self.active = false;
                if self.open { Transition::Opened } else { Transition::Closed }
            }
            InputEvent::Select(index) => {
                if !self.open || self.dispatched { return Transition::None; }
                self.open = false;
                self.dispatched = true;
                actions.get(index).filter(|action| !action.is_empty()).cloned().map(Transition::Dispatch).unwrap_or(Transition::Closed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_canonicalizes_modifier_combinations() {
        assert_eq!(Shortcut::parse("shift+ctrl+space").unwrap().canonical(), "Ctrl+Shift+Space");
        assert_eq!(Shortcut::parse("x").unwrap().canonical(), "X");
    }
    #[test]
    fn rejects_invalid_keys_and_combinations() {
        for input in ["", "Ctrl++A", "Ctrl+Ctrl+A", "Ctrl+Escape", "Ctrl", "Foo+A", "A+B"] {
            assert!(Shortcut::parse(input).is_err(), "accepted {input}");
        }
    }
    #[test]
    fn hold_activation_repeats_cancel_and_dispatch_once() {
        let mut state = ActivationState::new(ActivationMode::Hold);
        let actions = vec!["open-app".to_owned()];
        assert_eq!(state.handle(InputEvent::Activated, &actions), Transition::Opened);
        assert_eq!(state.handle(InputEvent::Activated, &actions), Transition::None);
        assert_eq!(state.handle(InputEvent::Select(0), &actions), Transition::Dispatch("open-app".into()));
        assert_eq!(state.handle(InputEvent::Select(0), &actions), Transition::None);
        assert_eq!(state.handle(InputEvent::Deactivated, &actions), Transition::None);
    }
    #[test]
    fn hold_release_and_cancel_close_menu() {
        let mut state = ActivationState::new(ActivationMode::Hold);
        let actions = vec![];
        state.handle(InputEvent::Activated, &actions);
        assert_eq!(state.handle(InputEvent::Deactivated, &actions), Transition::Closed);
        state.handle(InputEvent::Activated, &actions);
        assert_eq!(state.handle(InputEvent::Escape, &actions), Transition::Cancelled);
        assert_eq!(state.handle(InputEvent::DeadZone, &actions), Transition::None);
    }
    #[test]
    fn toggle_activation_changes_only_on_edges() {
        let mut state = ActivationState::new(ActivationMode::Toggle);
        assert_eq!(state.handle(InputEvent::Activated, &[]), Transition::Opened);
        assert_eq!(state.handle(InputEvent::Activated, &[]), Transition::None);
        assert_eq!(state.handle(InputEvent::Deactivated, &[]), Transition::None);
        assert_eq!(state.handle(InputEvent::Activated, &[]), Transition::Closed);
    }
}
