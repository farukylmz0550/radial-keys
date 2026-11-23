# Shortcuts and KDE Wayland

## Configuration

New configurations default to the back mouse button in toggle mode; activation starts disabled and can be enabled in the settings window. Keyboard activation is also available by selecting Keyboard shortcut, with `Ctrl+Shift+Y` as its default. The shortcut is configurable through `SetActivationShortcut`.

Generic modifiers are `Ctrl`, `Alt`, `Shift`, and `Super`. Shortcuts are canonicalized in that order followed by the key. `Escape` is reserved for cancellation. A single-letter shortcut can conflict with normal typing; global activation must remain opt-in and must provide a recovery path.

## Wayland mechanism

The backend registers the shortcut through the XDG GlobalShortcuts portal. Portal activation is treated as one press: in toggle mode, press once to open and press again to close. Release events are ignored in toggle mode, so the user does not need to keep the keys held. Hold-to-release behavior still depends on whether the desktop portal provides reliable `Deactivated` events.

On KDE Plasma, one KWin script forwards `workspace.cursorPos` to the backend and a runtime-loaded script watches the active window's `fullScreen` state. While a full-screen window is active, activation is blocked, an open menu is closed, and the configured mouse button is relayed normally. Neither script registers a shortcut; the XDG portal remains the only keyboard shortcut handler. On other desktop environments, KWin full-screen detection is unavailable.

Mouse activation does not use the keyboard portal. When enabled, the backend searches compatible relative mice until it can grab one, verifies the chosen button, and forwards other events through a uinput virtual mouse. The configured button opens or closes the radial menu; while a full-screen window is active, that button is forwarded to the application instead. Supported choices are left, right, middle, back, and forward. If no device or permission is available, enabling the helper returns a clear D-Bus error and leaves activation disabled.

DEB, RPM, and Arch packages install `70-radial-keys-mouse.rules`, which grants the active desktop session access to udev-classified mouse devices and `/dev/uinput` for the virtual mouse relay. It does not grant access to keyboards or add the user to the broad `input` group. Reconnect a mouse after installing the package if its current device node does not yet have the session ACL. The helper releases the kernel grab when stopped or when the device disconnects, then retries discovery every two seconds while enabled.

## Current implementation status

Keyboard activation is user-mediated through `SetGlobalActivationEnabled(true)`. The current desktop must expose and accept the GlobalShortcuts portal binding. Mouse activation is separately opt-in and requires an accessible mouse event node plus `/dev/uinput` access.

## Next verification

On a real Plasma Wayland session, test portal binding for a modifier combination and an unmodified key separately. Confirm the portal user prompt, conflict behavior, toggle behavior, shortcut persistence, and behavior after restarting the user service.
