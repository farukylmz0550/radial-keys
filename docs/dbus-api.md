# D-Bus API

Radial Keys uses the session bus. The backend owns `io.github.radialkeys.Service` and exports `/io/github/radialkeys/Service` with interface `io.github.radialkeys.Service1`.

| Method | Signature | Behavior |
| --- | --- | --- |
| `Ping` | `() -> s` | Returns `pong`. |
| `GetVersion` | `() -> s` | Returns the backend package version. |
| `GetStatus` | `() -> s` | Returns JSON status, including renderer availability, activation type, configured mouse button, mouse helper status, and whether a full-screen window blocks activation. |
| `RendererHeartbeat` | `() -> ()` | Marks the renderer alive; status expires after three seconds without a heartbeat. |
| `RendererDisconnected` | `() -> ()` | Clears the renderer status when it exits normally. |
| `SetCursorPosition` | `(i, i) -> ()` | Stores compositor-reported global pointer coordinates for renderer placement. |
| `SetFullscreenActive` | `(b) -> ()` | Updates the KWin full-screen block state and closes an open menu when the value becomes true. |
| `GetConfiguration` | `() -> s` | Returns TOML configuration. |
| `GetSlots` | `() -> s` | Returns the eight slot labels, macro paths, and icon paths as JSON. |
| `GetBuiltinMacros` | `() -> s` | Returns the built-in macro catalog, parameter definitions, and icons as JSON. |
| `GetMacroStatus` | `() -> s` | Returns running macros, recent results, process settings, and microphone state as JSON. |
| `SetMacroOptions` | `(b, x) -> ()` | Sets single-instance mode and timeout seconds; `-1` disables timeout. |
| `TerminateMacro` | `(s) -> ()` | Stops a running macro by process ID. |
| `TerminateSlotMacros` | `(u) -> u` | Stops running macros for one slot and returns the number signalled. |
| `AcceptScriptUpdate` | `(u) -> ()` | Stores the current script SHA-256 as the explicitly accepted baseline. |
| `SetSlotPreset` | `(u, s, s, s, s) -> ()` | Saves a built-in macro ID, JSON parameters, label, and icon path for a slot. |
| `SetSlotPython` | `(u, s, s, s) -> ()` | Saves Python source authored or imported in the GUI to private backend-managed storage. |
| `SetSlot` | `(u, s, s) -> ()` | Validates and atomically saves a slot label and an absolute `.py` macro path (empty path clears the macro); preserves the slot icon. |
| `SetSlotDetails` | `(u, s, s, s) -> ()` | Atomically saves a slot label, macro path, and absolute PNG/JPEG/SVG icon path; empty paths clear those assignments. |
| `GetActivationShortcut` | `() -> s` | Returns canonical keyboard shortcut text. |
| `SetActivationShortcut` | `(s) -> ()` | Validates and persists the new shortcut before applying it. |
| `SetGlobalActivationEnabled` | `(b) -> ()` | Registers or closes the shortcut through the user-mediated portal. Registration is opt-in. |
| `SetMode` | `(s) -> ()` | Accepts `hold` or `toggle`, then persists the change. |
| `SetActivationDevice` | `(s, s) -> ()` | Stores `keyboard` or `mouse` input and one of `left`, `right`, `middle`, `back`, or `forward`. Input changes disable activation until re-enabled. |
| `ShowMenu` | `() -> ()` | Opens backend menu state; a connected renderer displays the overlay. |
| `HideMenu` | `() -> ()` | Closes backend menu state. |
| `CancelMenu` | `() -> ()` | Cancels menu state; the renderer also calls this on Escape. |
| `SelectSlot` | `(u) -> s` | Returns the selected action string; it does not execute it. |
| `ActivateSlot` | `(u) -> s` | Runs the configured built-in or Python macro, then closes the menu. |

Errors are returned as standard D-Bus method errors with a human-readable message. Invalid configuration is rejected before it replaces active state. A failed shortcut update leaves the old value and state intact.

The interface is versioned by its name. Compatible additions may stay on `Service1`; incompatible semantics require `Service2` or a new object path.

The session bus is not a per-application trust boundary: other processes running as the same user can call these methods. `ActivateSlot` additionally requires a recently active renderer and open menu to reduce accidental or unrelated invocation, but this does not isolate same-user callers.

Frontend status signals are not implemented yet.
