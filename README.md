<div align="center">

# radial-keys

**A small, keyboard-first foundation for radial menus on Linux.**

Rust · D-Bus · Wayland-aware

</div>

`radial-keys` is an early-stage backend for a radial keyboard menu. It provides an eight-slot TOML configuration, shortcut parsing, hold/toggle activation state, and a versioned session D-Bus interface. The backend is kept independent of any GUI toolkit so a future menu can be built separately.

> **Status:** Phase 1 foundation. There is no graphical menu yet, and global shortcut registration is not enabled. The XDG GlobalShortcuts portal is only probed for availability; keyboard activation and key-release behavior have not been verified on KDE Wayland.

## What’s here

- Eight configurable radial slots with stable IDs, labels, and action strings
- `X` as the default shortcut, in hold mode
- Canonical parsing for keys and modifier combinations
- Hold and toggle state transitions, including cancellation and duplicate-event handling
- Validated TOML configuration with atomic writes
- A versioned D-Bus API for status, configuration, shortcut changes, and menu requests
- A portal availability probe that reports its limits instead of claiming global activation works

## Requirements

- Rust and Cargo (edition 2024)
- A running user session D-Bus
- XDG desktop portal components for the optional GlobalShortcuts availability probe

No Qt or privileged input access is used. This phase does not install an evdev/uinput helper.

## Build and run

From the repository root:

```sh
cargo build --release
cargo test
cargo run --release
```

The backend claims the session-bus name `org.radialkeys.Backend1` and serves the object `/org/radialkeys/Backend1`. Press `Ctrl+C` to stop it. A systemd user unit and install/uninstall workflow have not been added yet.

## Configuration

The backend reads `config.toml` from its XDG application configuration directory. If the file is missing, it starts with these defaults:

```toml
version = 1
shortcut = "X"
mode = "hold"

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

Exactly eight slots are required. Slot IDs must be unique, labels must not be blank, and the shortcut must contain one key with optional modifiers. For example, `Ctrl+Shift+Space` is canonicalized to `Ctrl+Shift+Space`. `Escape` is reserved for cancellation. Configuration updates are validated and written through a temporary file before replacing the saved file; invalid updates leave the active configuration unchanged.

Supported modifiers are `Ctrl`, `Alt`, `Shift`, and `Super`. Common key names, alphanumeric keys, and `F1`–`F24` are accepted.

## D-Bus API

The interface is `org.radialkeys.Backend1` at `/org/radialkeys/Backend1`.

| Method | Purpose |
| --- | --- |
| `GetStatus()` | Returns JSON status, including backend/menu availability and portal-probe result |
| `GetConfiguration()` | Returns the current TOML configuration |
| `GetShortcut()` | Reads the current shortcut |
| `SetShortcut(s)` | Validates, saves, and applies a shortcut change |
| `SetMode(s)` | Sets `hold` or `toggle` mode |
| `ShowMenu()` | Requests that the backend open its menu state |
| `HideMenu()` | Requests that the backend close its menu state |
| `CancelMenu()` | Cancels the current menu state |
| `SelectSlot(u)` | Selects a slot and returns its configured action string |

For example, inspect the current status with `gdbus`:

```sh
gdbus call --session \
  --dest org.radialkeys.Backend1 \
  --object-path /org/radialkeys/Backend1 \
  --method org.radialkeys.Backend1.GetStatus
```

Menu methods currently operate on backend state only. There is no GUI to display, and a selected action string is not executed by this foundation.

## Wayland and global shortcuts

On Wayland, applications cannot generally observe arbitrary keyboard events globally. `radial-keys` checks whether the session portal exposes `org.freedesktop.portal.GlobalShortcuts`; it does not create a portal session or register the configured shortcut yet. In particular, portal availability alone does not prove that a requested key combination is supported or that activation and deactivation reliably correspond to key press and release.

Until those behaviors are demonstrated on the target compositor, hold-mode global activation is considered **unverified**. The backend does not silently change modes or use privileged input interception.

## Project structure

Keep implementation files grouped by area under `src/`: backend modules belong in `src/backend/` and frontend modules belong in `src/frontend/`. The current backend files have not yet been moved into that structure.

```text
src/
├── backend/
│   ├── config.rs
│   ├── dbus.rs
│   ├── portal.rs
│   └── shortcuts.rs
├── frontend/
└── main.rs
```

## License

This project is currently marked MIT in `Cargo.toml`. A standalone license file will be added with the project’s packaging setup.
