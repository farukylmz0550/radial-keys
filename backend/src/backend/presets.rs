use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuiltinMacroConfig {
    pub id: String,
    #[serde(default)]
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresetInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub parameter_labels: &'static [&'static str],
    pub defaults: &'static [&'static str],
    pub icon: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_muted: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroInvocation {
    Process(CommandSpec),
    ToggleDesktop,
}

const PRESETS: &[PresetInfo] = &[
    preset(
        "open_website",
        "Open website",
        "Open an HTTP or HTTPS address in the default browser.",
        &["Website URL"],
        &["https://www.youtube.com"],
        "open-website.svg",
    ),
    preset(
        "launch_application",
        "Open application",
        "Launch an application by executable name.",
        &["Application command"],
        &["konsole"],
        "launch-application.svg",
    ),
    preset(
        "run_command",
        "Run command",
        "Run a configured executable with optional arguments, without a shell.",
        &["Executable", "Arguments"],
        &["", ""],
        "run-command.svg",
    ),
    preset(
        "open_folder",
        "Open folder",
        "Open a folder in the default file manager.",
        &["Folder path"],
        &["~/Downloads"],
        "open-folder.svg",
    ),
    preset(
        "screenshot_full",
        "Full screenshot",
        "Capture the full screen with Spectacle.",
        &[],
        &[],
        "screenshot-full.svg",
    ),
    preset(
        "screenshot_region",
        "Area screenshot",
        "Select an area to capture with Spectacle.",
        &[],
        &[],
        "screenshot-region.svg",
    ),
    preset(
        "media_play_pause",
        "Play / pause",
        "Toggle playback in the active media player.",
        &[],
        &[],
        "media-play-pause.svg",
    ),
    preset(
        "media_next",
        "Next track",
        "Skip to the next track.",
        &[],
        &[],
        "media-next.svg",
    ),
    preset(
        "media_previous",
        "Previous track",
        "Return to the previous track.",
        &[],
        &[],
        "media-previous.svg",
    ),
    preset(
        "media_stop",
        "Stop media",
        "Stop playback in the active media player.",
        &[],
        &[],
        "media-stop.svg",
    ),
    preset(
        "volume_toggle",
        "Toggle volume mute",
        "Mute or unmute the default output device.",
        &[],
        &[],
        "volume-toggle.svg",
    ),
    preset(
        "volume_up",
        "Volume up",
        "Raise the default output volume by five percent.",
        &[],
        &[],
        "volume-up.svg",
    ),
    preset(
        "volume_down",
        "Volume down",
        "Lower the default output volume by five percent.",
        &[],
        &[],
        "volume-down.svg",
    ),
    PresetInfo {
        id: "microphone_toggle",
        label: "Toggle microphone mute",
        description: "Mute or unmute the default microphone. The wheel icon reflects its current state.",
        parameter_labels: &[],
        defaults: &[],
        icon: "microphone-unmuted.svg",
        icon_muted: Some("microphone-muted.svg"),
    },
    preset(
        "lock_screen",
        "Lock screen",
        "Lock the current desktop session.",
        &[],
        &[],
        "lock-screen.svg",
    ),
    preset(
        "show_desktop",
        "Show / hide desktop",
        "Toggle the desktop view in KDE Plasma.",
        &[],
        &[],
        "show-desktop.svg",
    ),
];

const fn preset(
    id: &'static str,
    label: &'static str,
    description: &'static str,
    parameter_labels: &'static [&'static str],
    defaults: &'static [&'static str],
    icon: &'static str,
) -> PresetInfo {
    PresetInfo {
        id,
        label,
        description,
        parameter_labels,
        defaults,
        icon,
        icon_muted: None,
    }
}

pub fn catalog() -> &'static [PresetInfo] {
    PRESETS
}

pub fn definition(id: &str) -> Option<&'static PresetInfo> {
    PRESETS.iter().find(|preset| preset.id == id)
}

pub fn validate(config: &BuiltinMacroConfig) -> Result<(), String> {
    let definition = definition(&config.id).ok_or_else(|| "unknown built-in macro".to_owned())?;
    if config.parameters.len() != definition.parameter_labels.len() {
        return Err(format!(
            "{} expects {} parameter(s)",
            definition.label,
            definition.parameter_labels.len()
        ));
    }
    if config
        .parameters
        .iter()
        .any(|value| value.len() > 2048 || value.contains('\0'))
    {
        return Err(
            "macro parameters must be at most 2048 bytes and contain no NUL characters".into(),
        );
    }
    match config.id.as_str() {
        "open_website" => {
            let url = &config.parameters[0];
            if !(url.starts_with("https://") || url.starts_with("http://"))
                || url.chars().any(char::is_whitespace)
            {
                return Err(
                    "website address must start with http:// or https:// and contain no spaces"
                        .into(),
                );
            }
        }
        "launch_application" if config.parameters[0].trim().is_empty() => {
            return Err("application command must not be empty".into());
        }
        "run_command" if config.parameters[0].trim().is_empty() => {
            return Err("executable must not be empty".into());
        }
        "open_folder" if config.parameters[0].trim().is_empty() => {
            return Err("folder path must not be empty".into());
        }
        _ => {}
    }
    Ok(())
}

pub fn invocation(config: &BuiltinMacroConfig) -> Result<MacroInvocation, String> {
    validate(config)?;
    let process = |program: &str, args: &[&str]| {
        MacroInvocation::Process(CommandSpec {
            program: program.into(),
            args: args.iter().map(|arg| (*arg).into()).collect(),
        })
    };
    let invocation = match config.id.as_str() {
        "open_website" => process("xdg-open", &[&config.parameters[0]]),
        "launch_application" => process(&config.parameters[0], &[]),
        "run_command" => {
            let args = parse_arguments(&config.parameters[1])?;
            MacroInvocation::Process(CommandSpec {
                program: config.parameters[0].clone(),
                args,
            })
        }
        "open_folder" => process("xdg-open", &[&expand_home(&config.parameters[0])]),
        "screenshot_full" => process("spectacle", &["-b", "-f"]),
        "screenshot_region" => process("spectacle", &["-b", "-r"]),
        "media_play_pause" => process("playerctl", &["play-pause"]),
        "media_next" => process("playerctl", &["next"]),
        "media_previous" => process("playerctl", &["previous"]),
        "media_stop" => process("playerctl", &["stop"]),
        "volume_toggle" => process("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]),
        "volume_up" => process("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"]),
        "volume_down" => process("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]),
        "microphone_toggle" => process("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]),
        "lock_screen" => process("loginctl", &["lock-session"]),
        "show_desktop" => MacroInvocation::ToggleDesktop,
        _ => return Err("unknown built-in macro".into()),
    };
    Ok(invocation)
}

fn expand_home(path: &str) -> String {
    if (path == "~" || path.starts_with("~/"))
        && let Some(home) = std::env::var_os("HOME")
    {
        return format!("{}{}", home.to_string_lossy(), &path[1..]);
    }
    path.to_owned()
}

/// Parse an argument field as tokens only. No variable expansion, redirects,
/// command substitution, or other shell behavior is performed.
pub fn parse_arguments(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut active = false;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            active = true;
        } else if character == '\\' && quote != Some('\'') {
            escaped = true;
            active = true;
        } else if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            } else {
                current.push(character);
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            active = true;
        } else if character.is_whitespace() {
            if active {
                args.push(std::mem::take(&mut current));
                active = false;
            }
        } else {
            current.push(character);
            active = true;
        }
    }
    if escaped || quote.is_some() {
        return Err("arguments contain an unclosed quote or trailing escape".into());
    }
    if active {
        args.push(current);
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_sixteen_actions_and_two_microphone_icons() {
        assert_eq!(catalog().len(), 16);
        let microphone = definition("microphone_toggle").unwrap();
        assert_eq!(microphone.icon_muted, Some("microphone-muted.svg"));
        assert_eq!(microphone.icon, "microphone-unmuted.svg");
    }

    #[test]
    fn parses_arguments_without_shell_expansion() {
        assert_eq!(
            parse_arguments("--title 'hello world' $HOME").unwrap(),
            ["--title", "hello world", "$HOME"]
        );
        assert!(parse_arguments("'unfinished").is_err());
    }

    #[test]
    fn validates_targets_and_returns_direct_commands() {
        let website = BuiltinMacroConfig {
            id: "open_website".into(),
            parameters: vec!["https://example.com".into()],
        };
        assert_eq!(
            invocation(&website).unwrap(),
            MacroInvocation::Process(CommandSpec {
                program: "xdg-open".into(),
                args: vec!["https://example.com".into()]
            })
        );
        let bad = BuiltinMacroConfig {
            id: "open_website".into(),
            parameters: vec!["javascript:alert(1)".into()],
        };
        assert!(validate(&bad).is_err());
    }
}
