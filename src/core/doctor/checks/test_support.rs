//! Shared test-only helpers for the doctor checks.

use std::path::{Path, PathBuf};

/// RAII guard that chdir's into a fresh tempdir and restores the original cwd
/// on drop. Local `.scuv-version` resolution is relative to the process cwd,
/// so tests must actually move there — env vars can't substitute.
pub(super) struct TempDirCwdGuard {
    _tmp: tempfile::TempDir,
    new_cwd: PathBuf,
    original: PathBuf,
}

impl TempDirCwdGuard {
    pub(super) fn new() -> Self {
        let original = std::env::current_dir().expect("cwd readable");
        let tmp = tempfile::tempdir().unwrap();
        let new_cwd = tmp.path().to_path_buf();
        std::env::set_current_dir(&new_cwd).expect("chdir into tempdir");
        Self {
            _tmp: tmp,
            new_cwd,
            original,
        }
    }
    pub(super) fn path(&self) -> &Path {
        &self.new_cwd
    }
}

impl Drop for TempDirCwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}
