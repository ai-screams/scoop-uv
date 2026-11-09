//! What an env directory holds, judged from its files alone: the facts
//! `scuv verify` and `scuv doctor` both report on. Each command decides how
//! serious each fact is.

use std::path::Path;

use crate::core::Metadata;
use crate::paths;

/// The env's scuv metadata file.
#[derive(Debug)]
pub(crate) enum MetadataState {
    /// Present and parsed.
    Present(Metadata),
    /// No file at that path.
    Missing,
    /// Something is there but it does not parse as metadata.
    Unreadable,
}

/// The files an env needs, by presence.
#[derive(Debug)]
pub(crate) struct EnvLayout {
    pub(crate) metadata: MetadataState,
    pub(crate) python: bool,
    pub(crate) pyvenv_cfg: bool,
    pub(crate) activate: bool,
}

/// Reads the layout of the env at `path`.
///
/// Metadata that cannot be read counts as missing when nothing exists at
/// its path (`exists()` is false, which also covers a dangling or looping
/// symlink) and as unreadable otherwise, the split `scuv verify` has always
/// reported.
pub(crate) fn inspect_env(path: &Path) -> EnvLayout {
    let metadata = match super::read_metadata_file(path).ok().flatten() {
        Some(m) => MetadataState::Present(m),
        None if path.join(Metadata::FILE_NAME).exists() => MetadataState::Unreadable,
        None => MetadataState::Missing,
    };
    EnvLayout {
        metadata,
        python: paths::virtualenv_python_exe(path).exists(),
        pyvenv_cfg: path.join("pyvenv.cfg").exists(),
        activate: paths::virtualenv_activate_script(path).exists(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_metadata(dir: &Path) {
        let meta = Metadata::new("e".to_string(), "3.12".to_string(), None);
        std::fs::write(
            dir.join(Metadata::FILE_NAME),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn a_complete_env_has_every_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write_metadata(dir);
        let python = paths::virtualenv_python_exe(dir);
        std::fs::create_dir_all(python.parent().unwrap()).unwrap();
        std::fs::write(&python, "").unwrap();
        std::fs::write(paths::virtualenv_activate_script(dir), "").unwrap();
        std::fs::write(dir.join("pyvenv.cfg"), "").unwrap();

        let layout = inspect_env(dir);
        assert!(
            matches!(layout.metadata, MetadataState::Present(ref m) if m.python_version == "3.12")
        );
        assert!(layout.python && layout.pyvenv_cfg && layout.activate);
    }

    #[test]
    fn an_empty_dir_has_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = inspect_env(tmp.path());
        assert!(matches!(layout.metadata, MetadataState::Missing));
        assert!(!layout.python && !layout.pyvenv_cfg && !layout.activate);
    }

    /// Something at the metadata path that is not metadata is unreadable:
    /// malformed JSON, or a directory.
    #[test]
    fn metadata_that_does_not_parse_is_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(Metadata::FILE_NAME), "{not json").unwrap();
        assert!(matches!(
            inspect_env(tmp.path()).metadata,
            MetadataState::Unreadable
        ));

        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join(Metadata::FILE_NAME)).unwrap();
        assert!(matches!(
            inspect_env(tmp.path()).metadata,
            MetadataState::Unreadable
        ));
    }

    /// A symlink that loops back on itself fails to read and `exists()` is
    /// false, so it is missing, as verify has always reported it.
    #[cfg(unix)]
    #[test]
    fn a_looping_metadata_symlink_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let link = tmp.path().join(Metadata::FILE_NAME);
        std::os::unix::fs::symlink(&link, &link).unwrap();
        assert!(matches!(
            inspect_env(tmp.path()).metadata,
            MetadataState::Missing
        ));
    }
}
