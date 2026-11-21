use zbus::Connection;

/// Checks whether the session portal exposes the XDG GlobalShortcuts interface.
/// This does not register a shortcut: registration requires an interactive portal
/// request and activation semantics must be verified on the active compositor.
pub async fn probe() -> String {
    let connection = match Connection::session().await {
        Ok(connection) => connection,
        Err(error) => return format!("unavailable: cannot connect to session bus ({error})"),
    };
    let proxy = match zbus::Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.GlobalShortcuts",
    ).await {
        Ok(proxy) => proxy,
        Err(error) => return format!("unavailable: GlobalShortcuts interface not found ({error})"),
    };
    match proxy.get_property::<u32>("version").await {
        Ok(version) => format!("available (portal interface version {version}; not registered; hold/release unverified)"),
        Err(error) => format!("interface found but version unavailable ({error}); not registered; activation unverified"),
    }
}
