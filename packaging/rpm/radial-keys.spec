Name:           radial-keys
Version:        2.0.0
Release:        1%{?dist}
Summary:        Customizable radial action menu for Linux
License:        GPL-3.0-only
URL:            https://github.com/farukylmz0550/radial-keys
Source0:        %{name}-%{version}.tar.gz
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  systemd-rpm-macros
BuildRequires:  cmake
BuildRequires:  qt6-qtbase-devel
BuildRequires:  qt6-qtdeclarative-devel
Requires:       dbus
Requires:       python3
Requires:       qt6-qtbase
Requires:       qt6-qtdeclarative
Requires:       qt6-qtquickdialogs
Requires:       qt6-qtwayland

%description
Radial Keys provides an independent Rust D-Bus backend and a Qt Quick renderer
for a customizable eight-slot radial action menu.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked
cmake -S frontend/qt -B build/frontend -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=%{_prefix}
cmake --build build/frontend --parallel

%install
install -Dpm0755 target/release/radial-keys %{buildroot}%{_bindir}/radial-keys
DESTDIR=%{buildroot} cmake --install build/frontend
install -Dpm0644 packaging/systemd/io.github.radialkeys.Service.service \
  %{buildroot}%{_userunitdir}/io.github.radialkeys.Service.service
install -Dpm0644 packaging/systemd/io.github.radialkeys.Renderer.service \
  %{buildroot}%{_userunitdir}/io.github.radialkeys.Renderer.service
install -Dpm0644 packaging/kwin/radialkeys-cursor/metadata.json \
  %{buildroot}%{_datadir}/kwin/scripts/radialkeys-cursor/metadata.json
install -Dpm0644 packaging/kwin/radialkeys-cursor/contents/code/main.js \
  %{buildroot}%{_datadir}/kwin/scripts/radialkeys-cursor/contents/code/main.js
install -Dpm0644 packaging/udev/70-radial-keys-mouse.rules \
  %{buildroot}%{_prefix}/lib/udev/rules.d/70-radial-keys-mouse.rules
install -Dpm0644 assets/radial-keys-logo.svg \
  %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.radialkeys.RadialKeys.svg

%files
%license LICENSE
%doc README.md docs
%{_bindir}/radial-keys
%{_bindir}/radial-keys-frontend
%{_userunitdir}/io.github.radialkeys.Service.service
%{_userunitdir}/io.github.radialkeys.Renderer.service
%{_datadir}/kwin/scripts/radialkeys-cursor/metadata.json
%{_datadir}/kwin/scripts/radialkeys-cursor/contents/code/main.js
%{_prefix}/lib/udev/rules.d/70-radial-keys-mouse.rules
%{_datadir}/icons/hicolor/scalable/apps/io.github.radialkeys.RadialKeys.svg

%changelog
* Mon Nov 23 2026 farukylmz0550 <219123924+farukylmz0550@users.noreply.github.com> - 2.0.0-1
- Add Flatpak packaging, full-screen-aware activation, complete action profiles, custom profiles, and hover descriptions

* Mon Nov 23 2026 farukylmz0550 <219123924+farukylmz0550@users.noreply.github.com> - 1.3.0-1
- Add click-wheel-inspired sound feedback when moving between menu segments

* Mon Nov 23 2026 farukylmz0550 <219123924+farukylmz0550@users.noreply.github.com> - 1.2.0-1
- Keep activation enabled while changing the trigger button, hide windows while the full-screen radial menu is open, and ship four default actions

* Sun Nov 22 2026 farukylmz0550 <219123924+farukylmz0550@users.noreply.github.com> - 1.1.0-1
- Add GUI-managed built-in and Python macros, process controls, integrity checks, and application icon
