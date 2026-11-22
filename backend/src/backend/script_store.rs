use super::macros::sha256_bytes;
use std::{fs, io::Write, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptStoreError {
    #[error("could not determine the Radial Keys configuration directory")]
    ConfigDirectory,
    #[error("Python source is too large")]
    TooLarge,
    #[error("could not store Python source: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct StoredScript {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct MacroScriptStore {
    directory: PathBuf,
}

impl MacroScriptStore {
    pub fn default_path() -> Result<Self, ScriptStoreError> {
        let directories = directories::ProjectDirs::from("org", "radial-keys", "radial-keys")
            .ok_or(ScriptStoreError::ConfigDirectory)?;
        Ok(Self {
            directory: directories.config_dir().join("macros"),
        })
    }

    #[cfg(test)]
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn store(&self, slot: usize, source: &str) -> Result<StoredScript, ScriptStoreError> {
        if slot >= 8 {
            return Err(ScriptStoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "slot index is out of range",
            )));
        }
        if source.len() > 16 * 1024 * 1024 {
            return Err(ScriptStoreError::TooLarge);
        }
        let hash = sha256_bytes(source.as_bytes());
        fs::create_dir_all(&self.directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.directory, fs::Permissions::from_mode(0o700))?;
        }
        let target = self.directory.join(format!("slot-{}-{hash}.py", slot + 1));
        if !target.exists() {
            let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                temporary
                    .as_file()
                    .set_permissions(fs::Permissions::from_mode(0o600))?;
            }
            temporary.write_all(source.as_bytes())?;
            temporary.as_file().sync_all()?;
            temporary.persist(&target).map_err(|error| error.error)?;
            if let Ok(directory) = fs::File::open(&self.directory) {
                let _ = directory.sync_all();
            }
        }
        let canonical = target.canonicalize()?;
        Ok(StoredScript {
            path: canonical.display().to_string(),
            sha256: hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    #[test]
    fn stores_python_source_privately_by_content_hash() {
        let directory = tempfile::tempdir().unwrap();
        let store = MacroScriptStore::new(directory.path().join("macros"));
        let first = store.store(0, "print('first')\n").unwrap();
        let second = store.store(0, "print('second')\n").unwrap();
        assert_ne!(first.path, second.path);
        assert!(first.path.ends_with(&format!("{}.py", first.sha256)));
        assert_eq!(fs::read(&first.path).unwrap(), b"print('first')\n");
        assert_eq!(
            fs::metadata(&first.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(directory.path().join("macros"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
}
