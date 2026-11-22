<div align="center">

# radial-keys

**A native, customizable radial action menu for Linux.**

Rust · D-Bus · Qt 6/QML · Wayland-aware

</div>

Radial Keys uses an independent Rust session backend and a replaceable Qt 6/QML renderer. On KDE Plasma Wayland, press `Ctrl+Shift+Y` once to open the eight-segment menu and again to close it. Escape or a click outside a segment cancels it. The menu uses compositor cursor coordinates.

> **Status:** Backend and Qt Quick renderer are in place. Global shortcut registration is opt-in through the desktop portal. Eight Python macro slots can be configured from the GUI.

## Current features

- Eight configurable slots with stable IDs, labels, and action strings
- GUI editor for all eight slot labels, Python macro files, and PNG/JPEG/SVG icons
- Direct `python3 /absolute/path/to/macro.py` execution when a populated wheel segment is selected (no shell)
- `Ctrl+Shift+Y` as the default keyboard activation shortcut, stored in editable TOML configuration
- GUI configuration for keyboard or mouse activation input and a selectable mouse button; the evdev/uinput helper reserves only the configured button on the first detected mouse and relays the remaining events
- KDE Plasma Wayland cursor coordinates forwarded by a small KWin script so the menu can open at the pointer
- Canonical parsing for keys and modifier combinations
- Toggle activation by default, with cancellation and duplicate-event handling
- Validated TOML configuration with atomic writes
- Versioned session D-Bus API
- Eight-segment Qt Quick overlay with directional highlighting and Escape cancellation
- Native package definitions for DEB, RPM, and Arch Linux

See [Python macros](docs/macros.md) for assigning and writing slot scripts.

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

The backend claims `io.github.radialkeys.Service` and serves `/io/github/radialkeys/Service` with interface `io.github.radialkeys.Service1`. The frontend binary is `radial-keys-frontend`; both processes can run as separate user services.

## Configuration

The backend reads `config.toml` from its XDG application configuration directory. A first run uses these defaults:

```toml
version = 1

[activation]
type = "keyboard"
key = "Y"
modifiers = ["Ctrl", "Shift"]
mode = "toggle"
enabled = false

[[slots]]
id = "north"
label = "Slot 1"
action = ""

[[slots]]
id = "north-east"
label = "Slot 2"
action = ""

[[slots]]
id = "east"
label = "Slot 3"
action = ""

[[slots]]
id = "south-east"
label = "Slot 4"
action = ""

[[slots]]
id = "south"
label = "Slot 5"
action = ""

[[slots]]
id = "south-west"
label = "Slot 6"
action = ""

[[slots]]
id = "west"
label = "Slot 7"
action = ""

[[slots]]
id = "north-west"
label = "Slot 8"
action = ""
```

Exactly eight slots are required. Slot IDs must be unique, labels must not be blank, and the activation shortcut must contain one key with optional modifiers. For example, `Ctrl+Shift+Space` is canonicalized to `Ctrl+Shift+Space`. `Escape` is reserved for cancellation. Supported generic modifiers are `Ctrl`, `Alt`, `Shift`, and `Super`; common keys, alphanumeric keys, and `F1`–`F24` are accepted.

Invalid updates leave the active configuration unchanged. Writes use a temporary file followed by an atomic replacement. Corrupt configuration is reported and preserved for recovery.

## D-Bus API

The interface is `io.github.radialkeys.Service1` at `/io/github/radialkeys/Service`.

| Method | Purpose |
| --- | --- |
| `Ping()` | Checks whether the service responds |
| `GetVersion()` | Returns the backend package version |
| `GetStatus()` | Returns JSON service, renderer, menu, and portal status |
| `GetConfiguration()` | Returns current TOML configuration |
| `GetActivationShortcut()` | Reads the current shortcut |
| `SetActivationShortcut(s)` | Validates and persists a shortcut change |
| `SetGlobalActivationEnabled(b)` | Explicitly registers or disables the shortcut through the desktop portal |
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

Wayland does not let ordinary applications observe arbitrary global keyboard input. The desktop registers the shortcut through the XDG GlobalShortcuts portal. On KDE Plasma, the KWin script observes cursor coordinates only; it does not register a second shortcut, intercept or log keyboard input. Radial Keys does not use privileged input interception.

For accurate cursor placement on KDE Plasma Wayland, enable the **Radial Keys Cursor Position** script in System Settings → Window Management → KWin Scripts after installing the package. The script forwards cursor coordinates only; shortcut handling remains in the backend/desktop portal.

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

MIT; see [LICENSE](LICENSE).
