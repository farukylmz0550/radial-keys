use std::{env, path::PathBuf};

/// Returns the application's XDG configuration directory.
pub fn config_dir() -> Option<PathBuf> {
    config_dir_from(env::var_os("XDG_CONFIG_HOME"), env::var_os("HOME"))
}

fn config_dir_from(
    xdg_config_home: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    if let Some(path) = xdg_config_home.filter(|value| !value.is_empty()) {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Some(path.join("radial-keys"));
        }
    }

    home.filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|home| home.join(".config/radial-keys"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_config_home_is_used_when_absolute() {
        assert_eq!(
            config_dir_from(Some("/tmp/test-xdg".into()), Some("/home/test".into())),
            Some(PathBuf::from("/tmp/test-xdg/radial-keys"))
        );
    }

    #[test]
    fn relative_xdg_path_falls_back_to_home_config() {
        assert_eq!(
            config_dir_from(Some("relative".into()), Some("/home/test".into())),
            Some(PathBuf::from("/home/test/.config/radial-keys"))
        );
    }
}
