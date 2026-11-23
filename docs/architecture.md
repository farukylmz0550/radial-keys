# Architecture

## Process boundaries

The Rust backend owns configuration, activation state, shortcut parsing, and the session D-Bus API. Native packages run it as a `systemd --user` service; the Flatpak launcher keeps it alive with the Qt 6/QML renderer. The renderer is a settings client and does not read or write the backend's TOML file directly.

The backend remains usable when no renderer is connected. The renderer sends a heartbeat each second; status reports it available only while the heartbeat is current.

On KDE Plasma Wayland, `QCursor::pos()` cannot reliably report the pointer while it is over another application. The `radialkeys-cursor` KWin script forwards `workspace.cursorPos` to the backend over D-Bus. A second runtime-loaded KWin script reports the active window's full-screen state. The backend blocks activation and closes the menu while that state is true; the mouse helper passes the configured trigger button through normally. Other desktops retain the Qt cursor-position fallback and do not get KWin full-screen detection.

## Modules

- `backend/src/backend/config.rs`: TOML schema, validation, XDG config path, and atomic persistence.
- `backend/src/backend/shortcuts.rs`: canonical shortcut parsing and activation state transitions.
- `backend/src/backend/portal.rs`: XDG GlobalShortcuts registration and activation/release handling.
- `backend/src/backend/dbus.rs`: versioned, toolkit-independent service methods.
- `backend/src/backend/kwin.rs` and `kwin_fullscreen.js`: runtime installation of the KDE full-screen monitor.
- `backend/src/main.rs`: process startup, logging, session-bus ownership, and shutdown.
- `frontend/qt/src/BackendClient.cpp`: D-Bus status and renderer heartbeat.
- `frontend/qt/qml/menu/RadialMenu.qml`: overlay rendering and directional highlighting.

## Activation state

The current core models activation as state transitions instead of independent input flags. Toggle activation opens or closes on each shortcut press; Escape cancels the open menu. Hold activation remains available for portal-based integrations. Slot selection can dispatch at most once.

When explicitly enabled, the backend receives portal activation and deactivation events and updates menu state. The renderer polls that state, places the overlay on the pointer's screen, and computes the highlighted direction locally. The settings frontend saves slot labels, actions, and Python macro paths through D-Bus. Four built-in profiles contain eight actions each; user-created profiles are saved by the renderer in its QSettings store. Selecting a populated segment launches the selected built-in action or starts `python3` with the configured script path; no shell is involved. Under Flatpak, built-in desktop utilities run through `flatpak-spawn --host`, while Python macros remain in the sandbox.

## D-Bus compatibility

The service name is `io.github.radialkeys.Service`, object path `/io/github/radialkeys/Service`, interface `io.github.radialkeys.Service1`. Additive methods may be introduced within interface version 1. Breaking argument or semantic changes require a new interface version. Detailed methods are listed in [dbus-api.md](dbus-api.md).
