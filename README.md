<div align="center">

<img src="assets/radial-keys-logo.svg" width="128" alt="Radial Keys logo">

# radial-keys

**A customizable radial action menu for Linux.**

Rust · D-Bus · Qt 6/QML · Wayland-aware

</div>

Radial Keys uses an independent Rust session backend and a Qt 6/QML renderer. New configurations use the back mouse button for activation, initially disabled; enable it in the settings window. Keyboard activation remains available in the backend, with `Ctrl+Shift+Y` as the default shortcut. Four ready-to-use profiles cover Media, Productivity, System, and Gaming; users can save their own profiles from the current eight actions. On KDE Plasma, activation pauses while a full-screen window is active and the selected mouse button is passed through normally.

> **Version 2.0.0:** Flatpak-first release with complete action profiles, user-created profiles, action hover descriptions, and full-screen-aware activation.

## Current features

- Eight configurable slots with 16 built-in actions or custom Python macros
- Four complete ready-to-use profiles; users can save, select, and update their own profiles
- Hover descriptions for actions in the radial menu and action editor
- In-app Python editor; source is stored by the backend, so users do not need to manage script files
- Direct `python3 /absolute/path/to/macro.py` execution without a shell, with managed lifecycle, optional timeout, and single-instance mode
- Per-script SHA-256 change detection with explicit user acceptance
- GUI controls for macro status, errors, and stopping running scripts
- Original white SVG icons on transparent backgrounds for every built-in action, including separate muted/unmuted microphone states
- The back mouse button as the default activation input, with activation initially disabled; `Ctrl+Shift+Y` remains the default when keyboard activation is selected
- GUI configuration for keyboard or mouse activation input and a selectable mouse button; the evdev/uinput helper reserves only the configured button on the first detected mouse and relays the remaining events
- Subtle click-wheel-inspired sound feedback when moving between menu segments
- KDE Plasma Wayland cursor coordinates forwarded by a small KWin script so the menu can open at the pointer
- KWin full-screen tracking temporarily disables activation and passes the reserved mouse button through
- Flatpak built-in desktop actions run against host utilities and applications; Python macros stay in the Flatpak sandbox
- Canonical parsing for keys and modifier combinations
- Toggle activation by default, with cancellation and duplicate-event handling
- Validated TOML configuration with atomic writes
- Versioned session D-Bus API
- Eight-segment Qt Quick overlay with directional highlighting and Escape cancellation
- Native package definitions for DEB, RPM, and Arch Linux

The recommended installation is the 2.0.0 Flatpak. See [Flatpak build and install](docs/flatpak.md) and the [2.0.0 release](https://github.com/farukylmz0550/radial-keys/releases/tag/v2.0.0).

See [Macros](docs/macros.md) for the built-in actions, in-app Python editor, process controls, and security model.

Mouse activation is opt-in. The backend opens the first detected relative mouse, suppresses the configured activation button, and relays its other events through a virtual mouse. Packages install a udev rule granting the active desktop user access only to mouse-classified input devices. A running mouse may need to be reconnected after installing the package. The renderer runs separately and reports its heartbeat to the backend.

## Requirements and development

- Rust stable and Cargo (edition 2024)
- CMake and Qt 6 development modules to build the renderer
- A running user session D-Bus
- XDG Desktop Portal components for the optional GlobalShortcuts probe

From the repository root:

```sh
cargo build --release
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release
```

The backend claims `io.github.radialkeys.Service` and serves `/io/github/radialkeys/Service` with interface `io.github.radialkeys.Service1`. The frontend binary is `radial-keys-frontend`; native packages can run both processes as separate user services, while the Flatpak launcher manages them together.

## Configuration

The backend reads `config.toml` from its XDG application configuration directory. A first run uses these defaults:

```toml
version = 1

[activation]
type = "mouse"
key = "Y"
modifiers = ["Ctrl", "Shift"]
mode = "toggle"
enabled = false
mouse_button = "back"

[[slots]]
id = "north"
label = "Play / pause"
action = ""
icon = "qrc:/icons/media-play-pause.svg"

[slots.preset]
id = "media_play_pause"
parameters = []

[[slots]]
id = "north-east"
label = "Volume down"
action = ""
icon = "qrc:/icons/volume-down.svg"

[slots.preset]
id = "volume_down"
parameters = []

[[slots]]
id = "east"
label = "Volume up"
action = ""
icon = "qrc:/icons/volume-up.svg"

[slots.preset]
id = "volume_up"
parameters = []

[[slots]]
id = "south-east"
label = "Toggle mute"
action = ""
icon = "qrc:/icons/volume-toggle.svg"

[slots.preset]
id = "volume_toggle"
parameters = []

[[slots]]
id = "south"
label = "Next track"
action = ""
icon = "qrc:/icons/media-next.svg"

[slots.preset]
id = "media_next"
parameters = []

[[slots]]
id = "south-west"
label = "Previous track"
action = ""
icon = "qrc:/icons/media-previous.svg"

[slots.preset]
id = "media_previous"
parameters = []

[[slots]]
id = "west"
label = "Stop media"
action = ""
icon = "qrc:/icons/media-stop.svg"

[slots.preset]
id = "media_stop"
parameters = []

[[slots]]
id = "north-west"
label = "Spotify"
action = ""
icon = "qrc:/icons/open-website.svg"

[slots.preset]
id = "open_website"
parameters = ["https://open.spotify.com"]
```

Exactly eight slots are required; new configurations start with eight populated media actions. Built-in profiles also include Spotify, YouTube, and Netflix launch actions. Slot IDs must be unique, labels must not be blank, and the activation shortcut must contain one key with optional modifiers. `Escape` is reserved for cancellation. Supported generic modifiers are `Ctrl`, `Alt`, `Shift`, and `Super`; common keys, alphanumeric keys, and `F1`–`F24` are accepted.

Invalid updates leave the active configuration unchanged. Writes use a temporary file followed by an atomic replacement. Corrupt configuration is reported and preserved for recovery.

## D-Bus API

The interface is `io.github.radialkeys.Service1` at `/io/github/radialkeys/Service`.

| Method | Purpose |
| --- | --- |
| `Ping()` | Checks whether the service responds |
| `GetVersion()` | Returns the backend package version |
| `GetStatus()` | Returns JSON service, renderer, menu, full-screen, and portal status |
| `GetConfiguration()` | Returns current TOML configuration |
| `GetActivationShortcut()` | Reads the current shortcut |
| `SetActivationShortcut(s)` | Validates and persists a shortcut change |
| `SetGlobalActivationEnabled(b)` | Explicitly registers or disables the shortcut through the desktop portal |
| `SetFullscreenActive(b)` | Temporarily blocks activation while KWin reports a full-screen active window |
| `SetMode(s)` | Sets `hold` or `toggle` |
| `ShowMenu()` / `HideMenu()` / `CancelMenu()` | Changes backend menu state |
| `SelectSlot(u)` | Returns a configured action string for a selected slot |
| `GetSlots()` / `SetSlotDetails(u, s, s, s)` | Reads or saves slot labels, Python macro paths, and icon paths |
| `ActivateSlot(u)` | Launches the configured Python macro and closes the menu |

For example:

```sh
gdbus call --session \
  --dest io.github.radialkeys.Service \
  --object-path /io/github/radialkeys/Service \
  --method io.github.radialkeys.Service1.GetStatus
```

`ShowMenu()` opens the overlay for a preview. Assign a `.py` file to a slot in the settings window; selecting that segment starts the macro as the current user. `GetStatus()` reports renderer availability from its heartbeat.

## Wayland shortcut limits

Wayland does not let ordinary applications observe arbitrary global keyboard input. The desktop registers the shortcut through the XDG GlobalShortcuts portal. On KDE Plasma, KWin scripts forward cursor coordinates and report whether the active window is full-screen; they do not register a second shortcut or log keyboard input.

On KDE Plasma Wayland, the Flatpak loads a KWin monitor at startup to track full-screen windows and unloads it during normal shutdown. Native packages can additionally install the `radialkeys-cursor` KWin script for pointer coordinates; the Flatpak currently uses Qt's cursor-position fallback.

## Native packages

Package definitions are provided for Debian/Ubuntu (`debian/`), Fedora/RPM (`packaging/rpm/`), and Arch (`packaging/arch/`). Packages include the backend, Qt renderer, and separate user units. Build instructions and packaging limitations are in [docs/development.md](docs/development.md).

After installing a package, enable the backend and renderer at login:

```sh
systemctl --user enable --now io.github.radialkeys.Service.service io.github.radialkeys.Renderer.service
```

## Project structure

```text
backend/Cargo.toml Rust backend package
backend/src/       Rust backend process and modules
frontend/qt/       Qt 6/QML renderer and shortcut settings
packaging/         RPM, Arch, and systemd definitions
debian/            Debian package metadata
docs/              architecture and API documentation
```

## License

GPL-3.0-only; see [LICENSE](LICENSE). Third-party dependencies and bundled
components retain their respective licenses.

## Disclaimer

Radial Keys is provided “as is,” without any express or implied warranty. The
developers accept no liability for loss, damage, or other issues arising from
the use of this software. See [LICENSE](LICENSE) for the full terms.
