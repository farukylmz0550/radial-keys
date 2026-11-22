# Development and packaging

## Rust checks

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked
```

The Qt 6/QML renderer lives in `frontend/qt/`. Build it with `cmake -S frontend/qt -B build/frontend -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr && cmake --build build/frontend --parallel`. On Plasma, the app uses KDE's `org.kde.desktop` Qt Quick Controls style when available, unless `QT_QUICK_CONTROLS_STYLE` is explicitly set. The settings window uses a dark palette with white text for active, inactive, and disabled controls; native KDE file dialogs retain the desktop's own theme.

## User service

The Rust backend and Qt renderer have separate user units. They do not request elevated privileges and are not enabled automatically by package installation. After installing a package, enable both with:

```sh
systemctl --user enable --now io.github.radialkeys.Service.service io.github.radialkeys.Renderer.service
```

To stop and disable it:

```sh
systemctl --user disable --now io.github.radialkeys.Renderer.service io.github.radialkeys.Service.service
```

## Native package definitions

The repository includes:

- Debian/Ubuntu metadata in `debian/`, built with `dpkg-buildpackage -us -uc`.
- Fedora/RPM spec in `packaging/rpm/radial-keys.spec`; create a source archive with `git archive --format=tar.gz --prefix=radial-keys-0.1.0/ -o radial-keys-0.1.0.tar.gz HEAD`, place it in the RPM source directory, then build with `rpmbuild -ba`.
- Arch PKGBUILD in `packaging/arch/PKGBUILD`; it is checkout-oriented and builds from the repository root with `makepkg`.

The RPM spec and Arch recipe currently use version `0.1.0`; update their version and source references when cutting a release. The Debian maintainer address is a reserved `.invalid` placeholder and must be replaced by the package maintainer before publishing. Package installation does not enable either user service.

## Environment limits

The development host has Rust, CMake, and Qt 6 development packages. The Qt frontend builds successfully. The live Plasma Wayland session uses the XDG GlobalShortcuts portal for activation and a KWin script only for compositor cursor coordinates.
