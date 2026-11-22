# Shortcuts and KDE Wayland

## Configuration

The default is `Ctrl+Shift+Y` in toggle mode: press once to open and again to close; Escape cancels. The shortcut is configurable through `SetActivationShortcut`.

Generic modifiers are `Ctrl`, `Alt`, `Shift`, and `Super`. Shortcuts are canonicalized in that order followed by the key. `Escape` is reserved for cancellation. A single-letter shortcut can conflict with normal typing; global activation must remain opt-in and must provide a recovery path.

## Wayland mechanism

The backend registers the shortcut through the XDG GlobalShortcuts portal. Portal activation is treated as one press: in toggle mode, press once to open and press again to close. Release events are ignored in toggle mode, so the user does not need to keep the keys held. Hold-to-release behavior still depends on whether the desktop portal provides reliable `Deactivated` events.

On KDE Plasma, a small KWin script only forwards `workspace.cursorPos` to the backend. It does not register another shortcut, avoiding a duplicate binding. The radial menu's sectors and labels are painted together on one canvas so they share the same coordinates; the canvas updates immediately without a separate label animation.

Mouse activation does not use the keyboard portal. When enabled, the backend finds the first relative mouse under `/dev/input`, verifies that it supports the chosen button, grabs that one device, and forwards every non-trigger event through a uinput virtual mouse. Only the configured button is withheld from applications and used to open or close the radial menu. Supported choices are left, right, middle, back, and forward. If no device or permission is available, enabling the helper returns a clear D-Bus error and leaves activation disabled.

DEB, RPM, and Arch packages install `70-radial-keys-mouse.rules`, which grants the active desktop session access to udev-classified mouse devices and `/dev/uinput` for the virtual mouse relay. It does not grant access to keyboards or add the user to the broad `input` group. Reconnect a mouse after installing the package if its current device node does not yet have the session ACL. The helper releases the kernel grab when stopped or when the device disconnects, then retries discovery every two seconds while enabled.

## Current implementation status

Keyboard activation is user-mediated through `SetGlobalActivationEnabled(true)`. The current desktop must expose and accept the GlobalShortcuts portal binding. Mouse activation is separately opt-in and requires an accessible mouse event node plus `/dev/uinput` access.

## Next verification

On a real Plasma Wayland session, test portal binding for a modifier combination and an unmodified key separately. Confirm the portal user prompt, conflict behavior, toggle behavior, shortcut persistence, and behavior after restarting the user service.
