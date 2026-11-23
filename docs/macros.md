# Macro editing and execution

Radial Keys provides 16 built-in actions and an in-app Python editor for custom
macros. Open **Actions**, choose a slot, and select either a built-in action or
**Custom Python…**. Write or paste code in the editor and save it there. The
backend stores the text in its private application data directory and records
the SHA-256 hash; users do not need to create or manage script files. The
optional **Import Python code** button copies an existing `.py` file into the
editor. Built-in actions expose their parameters directly in the settings UI.

The available built-ins are: open a website, open an application, run an
executable with arguments, open a folder, take a full or area screenshot,
control media playback (play/pause, next, previous, stop), toggle or adjust
volume, toggle microphone mute, lock the screen, and show/hide the desktop.
The microphone icon changes between the muted and unmuted white SVG assets when
PipeWire reports the current source state.

## Python permissions and execution

Python macros deliberately run with the logged-in user's permissions and the
user's Python environment. Radial Keys does not sandbox Python, restrict its
modules, or claim to detect malware. Only save code you trust. The backend uses
`tokio::process::Command` to execute `python3` with a script path; it does not
invoke a shell. Environment variables are inherited to preserve virtualenv and
user environment compatibility, including `PYTHONPATH` and `PYTHONHOME`.

Scripts authored in the GUI are written atomically with private file
permissions under the application data directory. A configured path must be an
absolute `.py` path to a regular file. Symlinks are resolved to their canonical
target before execution and hashing. A file could still be replaced between
validation and interpreter open; this is a filesystem race, not a security
boundary.

For a configured script, Radial Keys stores a SHA-256 baseline. If its content
changes, execution is blocked until the user reviews the code and chooses
**Accept current script**. Hash comparison only detects a byte change; it does
not establish whether code is safe. No script is deleted, quarantined, or
modified by the integrity check.

## Process management

The settings UI can enforce one running instance per macro and configure an
optional timeout. A timeout of **Disabled** means no time limit; Radial Keys
does not impose an arbitrary duration. Running processes and their most recent
results are visible in the settings UI, where a running macro can be stopped.
Stopping or timing out a macro sends termination to its process group, waits,
and escalates if needed; the backend reaps the direct child. Captured output is
capped at 16 KiB per stream and recent results are capped at 20 entries.

## Security and remaining limitations

The backend's D-Bus service is on the per-user session bus. Methods are
available to processes that can access that bus as the logged-in user; Unix UID
checks cannot distinguish trusted apps from other processes owned by the same
user. `ActivateSlot` additionally requires a recently active renderer and open
menu to reduce accidental or unrelated invocation, but this is not a
same-user authorization boundary. The session bus must not be treated as a
security sandbox.

Mouse activation uses the udev `uaccess` tag for mouse-classified event
devices, which grants access to the active local session through ACLs rather
than requiring the application to run as root. The helper grabs only a selected
relative mouse while active and relays unrelated events through uinput. Device
classification is heuristic and support depends on the installed udev rules,
session ACLs, and available uinput permissions.

Built-in macros run as the user and use desktop utilities such as `playerctl`,
`wpctl`, `spectacle`, and `loginctl`. Flatpak launches these built-ins through
`flatpak-spawn --host` so they can use host utilities and applications; Python
macros continue to run in a separate process inside the Flatpak sandbox. Native
packages invoke built-in executables directly. These actions use direct
executable-and-argument invocation, not a shell.
