# Flatpak

Radial Keys uses the KDE 6.11 runtime. The manifest builds the Rust backend
and Qt renderer in one app; the launcher keeps the backend alive while the
renderer is open. Python macros run as separate child processes using the
runtime's `python3` interpreter, inside the same Flatpak sandbox.
Built-in desktop actions such as media controls, volume controls, screenshots,
and launching installed applications use `flatpak-spawn --host`, because
their command-line tools and applications are installed on the host rather
than in the KDE runtime. User-created profiles are stored in the frontend's
QSettings configuration and can be saved or overwritten from the Actions tab.

## Permissions

The app currently requests `--device=all` because global mouse activation
grabs the selected physical mouse and relays the other events through
`/dev/uinput`. This grants access to other device nodes too. Reducing this
permission requires replacing the current input relay design.

The app also requests Wayland, fallback X11, PulseAudio, its session D-Bus
service name, KDE KWin D-Bus access, and MPRIS access for desktop actions.
Host actions also use the Flatpak portal's `org.freedesktop.Flatpak` spawn
interface.
It does not request unrestricted home-directory access. Qt file dialogs and
the desktop portals mediate access to selected files.

## Build and install locally

From the repository root:

```sh
flatpak-builder --user --install-deps-from=flathub --force-clean \
  build/flatpak packaging/flatpak/io.github.farukylmz0550.radielkeys.yml
flatpak run io.github.farukylmz0550.radielkeys
```

The Cargo dependency list in `packaging/flatpak/cargo-sources.json` is
generated from `Cargo.lock` with `flatpak-cargo-generator.py`. Regenerate it
after changing Rust dependencies. Builds use Cargo offline mode so every
dependency is pinned and fetched by Flatpak Builder before compilation.

The manifest uses a local source directory for development. For a Flathub
submission, replace that source with a tagged Git source and pinned commit.
