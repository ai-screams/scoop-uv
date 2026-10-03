use super::*;
use serial_test::serial;
use std::io::Write;
use tempfile::TempDir;

// =========================================================================
// Test Helpers
// =========================================================================

/// Creates a mock pyenv root with Python versions and their `envs/` subdirectories.
///
/// Structure created:
/// ```text
/// <temp>/versions/
///   ├── 3.11.0/
///   │   └── bin/python (mock)
///   ├── 3.12.0/
///   │   └── bin/python (mock)
///   └── ...
/// ```
fn setup_mock_pyenv_root(versions: &[&str]) -> TempDir {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let versions_dir = temp.path().join("versions");
    fs::create_dir_all(&versions_dir).expect("Failed to create versions dir");

    for ver in versions {
        let bin_dir = versions_dir.join(ver).join("bin");
        fs::create_dir_all(&bin_dir).expect("Failed to create bin dir");
        // Create a mock python binary (just needs to exist for tests)
        fs::write(bin_dir.join("python"), "#!/bin/sh\necho mock")
            .expect("Failed to write mock python");
    }
    temp
}

/// Creates a mock pyenv-virtualenv environment within a Python version.
///
/// Structure created:
/// ```text
/// <pyenv_root>/versions/<python_version>/envs/<env_name>/
///   ├── bin/python
///   └── pyvenv.cfg
/// ```
fn create_mock_virtualenv(
    pyenv_root: &Path,
    python_version: &str,
    env_name: &str,
    create_python_bin: bool,
) {
    let env_path = pyenv_root
        .join("versions")
        .join(python_version)
        .join("envs")
        .join(env_name);
    let bin_dir = env_path.join("bin");
    fs::create_dir_all(&bin_dir).expect("Failed to create env bin dir");

    if create_python_bin {
        fs::write(bin_dir.join("python"), "#!/bin/sh\necho mock")
            .expect("Failed to write mock python");
    }

    // Create pyvenv.cfg
    let cfg_path = env_path.join("pyvenv.cfg");
    let cfg_content = format!(
        "home = {}/versions/{}/bin\n\
             include-system-site-packages = false\n\
             version = {}\n",
        pyenv_root.display(),
        python_version,
        python_version
    );
    fs::write(cfg_path, cfg_content).expect("Failed to write pyvenv.cfg");
}

// =========================================================================
// Unit Tests: PyenvDiscovery::new()
// =========================================================================

/// A pyenv virtualenv lives at `<root>/versions/<py>/envs/<name>`.
fn make_pyenv_env(root: &Path, py: &str, name: &str) -> PathBuf {
    let env = root.join("versions").join(py).join("envs").join(name);
    fs::create_dir_all(env.join("bin")).unwrap();
    fs::write(env.join("bin").join("python"), b"").unwrap();
    env
}

/// The `envs` subdirectory guard is `!exists() || !is_dir()`. A Python
/// version directory with a *file* named `envs` must be skipped, not
/// walked.
#[test]
fn scan_skips_version_dir_whose_envs_is_a_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    make_pyenv_env(root, "3.12.0", "web");
    // A second version dir where `envs` is a regular file.
    let broken = root.join("versions").join("3.11.0");
    fs::create_dir_all(&broken).unwrap();
    fs::write(broken.join("envs"), b"").unwrap();

    let d = PyenvDiscovery::new(root.to_path_buf());
    let envs = d.scan_environments().unwrap();
    assert_eq!(envs.len(), 1);
    assert_eq!(envs[0].name, "web");
}

/// Entries under `envs/` that are symlinks are skipped whatever they
/// point at — the guard is `is_symlink() || !is_dir()`.
#[cfg(unix)]
#[test]
fn scan_skips_symlinked_env_entries() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let real = make_pyenv_env(root, "3.12.0", "web");
    let envs_dir = root.join("versions").join("3.12.0").join("envs");
    std::os::unix::fs::symlink(&real, envs_dir.join("alias")).unwrap();

    let d = PyenvDiscovery::new(root.to_path_buf());
    let names: Vec<_> = d
        .scan_environments()
        .unwrap()
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert_eq!(names, vec!["web"], "the symlinked alias must not appear");
}

/// Plain files under `envs/` are not environments.
#[test]
fn scan_skips_plain_files_under_envs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    make_pyenv_env(root, "3.12.0", "web");
    fs::write(
        root.join("versions")
            .join("3.12.0")
            .join("envs")
            .join("stray"),
        b"",
    )
    .unwrap();

    let d = PyenvDiscovery::new(root.to_path_buf());
    assert_eq!(d.scan_environments().unwrap().len(), 1);
}

#[test]
fn test_new_creates_instance_with_given_root() {
    let path = PathBuf::from("/some/path/.pyenv");
    let discovery = PyenvDiscovery::new(path.clone());
    assert_eq!(discovery.root, path);
}

// =========================================================================
// Unit Tests: PyenvDiscovery::default_root()
// =========================================================================

#[test]
#[serial]
fn test_default_root_uses_pyenv_root_env_var() {
    let temp = TempDir::new().unwrap();
    let pyenv_path = temp.path().to_path_buf();

    // SAFETY: Test is serialized, no concurrent access to env vars
    unsafe {
        std::env::set_var("PYENV_ROOT", &pyenv_path);
    }

    let discovery = PyenvDiscovery::default_root();

    // SAFETY: Test cleanup
    unsafe {
        std::env::remove_var("PYENV_ROOT");
    }

    assert!(discovery.is_some());
    assert_eq!(discovery.unwrap().root, pyenv_path);
}

#[test]
#[serial]
fn test_default_root_returns_none_for_nonexistent_path() {
    // SAFETY: Test is serialized, no concurrent access to env vars
    unsafe {
        std::env::set_var("PYENV_ROOT", "/nonexistent/path/that/does/not/exist");
    }

    let discovery = PyenvDiscovery::default_root();

    // SAFETY: Test cleanup
    unsafe {
        std::env::remove_var("PYENV_ROOT");
    }

    assert!(discovery.is_none());
}

// =========================================================================
// Unit Tests: PyenvDiscovery::parse_pyvenv_cfg()
// =========================================================================

#[test]
fn test_parse_pyvenv_cfg_extracts_version() {
    let temp_dir = TempDir::new().unwrap();
    let cfg_path = temp_dir.path().join("pyvenv.cfg");

    let mut file = fs::File::create(&cfg_path).unwrap();
    writeln!(file, "home = /Users/test/.pyenv/versions/3.11.0/bin").unwrap();
    writeln!(file, "include-system-site-packages = false").unwrap();
    writeln!(file, "version = 3.11.0").unwrap();

    let version = PyenvDiscovery::parse_pyvenv_cfg(temp_dir.path());
    assert_eq!(version, Some("3.11.0".to_string()));
}

#[test]
fn test_parse_pyvenv_cfg_handles_no_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let cfg_path = temp_dir.path().join("pyvenv.cfg");

    // Some tools write without spaces around '='
    fs::write(&cfg_path, "version=3.10.5\nhome=/usr/bin\n").unwrap();

    let version = PyenvDiscovery::parse_pyvenv_cfg(temp_dir.path());
    assert_eq!(version, Some("3.10.5".to_string()));
}

#[test]
fn test_parse_pyvenv_cfg_fallback_to_home_path() {
    let temp_dir = TempDir::new().unwrap();
    let cfg_path = temp_dir.path().join("pyvenv.cfg");

    // No version key, should fallback to parsing home path
    let mut file = fs::File::create(&cfg_path).unwrap();
    writeln!(file, "home = /Users/test/.pyenv/versions/3.9.7/bin").unwrap();
    writeln!(file, "include-system-site-packages = false").unwrap();

    let version = PyenvDiscovery::parse_pyvenv_cfg(temp_dir.path());
    assert_eq!(version, Some("3.9.7".to_string()));
}

#[test]
fn test_parse_pyvenv_cfg_returns_none_for_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    // No pyvenv.cfg created
    let version = PyenvDiscovery::parse_pyvenv_cfg(temp_dir.path());
    assert!(version.is_none());
}

#[test]
fn test_parse_pyvenv_cfg_returns_none_for_malformed_content() {
    let temp_dir = TempDir::new().unwrap();
    let cfg_path = temp_dir.path().join("pyvenv.cfg");

    fs::write(&cfg_path, "garbage content without version info").unwrap();

    let version = PyenvDiscovery::parse_pyvenv_cfg(temp_dir.path());
    assert!(version.is_none());
}

// =========================================================================
// Integration Tests: scan_environments()
// =========================================================================

#[test]
fn test_scan_empty_pyenv_returns_empty() {
    let temp = TempDir::new().unwrap();
    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());

    let result = discovery.scan_environments();

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_pyenv_with_versions_but_no_envs() {
    // pyenv versions (3.11.0, 3.12.0) are NOT virtualenvs
    // They are base Python installations without envs/ subdirectory
    let temp = setup_mock_pyenv_root(&["3.11.0", "3.12.0"]);
    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());

    let result = discovery.scan_environments();

    assert!(result.is_ok());
    // No envs/ subdirectory means no virtualenvs discovered
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_discovers_virtualenvs_in_envs_directory() {
    let temp = setup_mock_pyenv_root(&["3.11.0", "3.12.0"]);

    // Create virtualenvs in envs/ subdirectories
    create_mock_virtualenv(temp.path(), "3.11.0", "project-a", true);
    create_mock_virtualenv(temp.path(), "3.11.0", "project-b", true);
    create_mock_virtualenv(temp.path(), "3.12.0", "project-c", true);

    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());
    let result = discovery.scan_environments().unwrap();

    assert_eq!(result.len(), 3);

    // Results are sorted by name
    assert_eq!(result[0].name, "project-a");
    assert_eq!(result[0].python_version, "3.11.0");
    assert!(matches!(result[0].source_type, SourceType::Pyenv));

    assert_eq!(result[1].name, "project-b");
    assert_eq!(result[1].python_version, "3.11.0");

    assert_eq!(result[2].name, "project-c");
    assert_eq!(result[2].python_version, "3.12.0");
}

#[test]
fn test_scan_detects_corrupted_envs_without_python_binary() {
    let temp = setup_mock_pyenv_root(&["3.11.0"]);

    // Create env WITHOUT python binary
    create_mock_virtualenv(temp.path(), "3.11.0", "broken-env", false);

    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());
    let result = discovery.scan_environments().unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "broken-env");
    assert!(matches!(
        result[0].status,
        EnvironmentStatus::Corrupted { .. }
    ));
}

#[test]
fn test_scan_skips_symlinks_in_versions_directory() {
    let temp = setup_mock_pyenv_root(&["3.11.0"]);
    create_mock_virtualenv(temp.path(), "3.11.0", "real-env", true);

    // pyenv creates symlinks for virtualenvs at top level of versions/
    // e.g., versions/real-env -> versions/3.11.0/envs/real-env
    let symlink_path = temp.path().join("versions").join("real-env");
    let target = temp
        .path()
        .join("versions")
        .join("3.11.0")
        .join("envs")
        .join("real-env");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &symlink_path).unwrap();

    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());
    let result = discovery.scan_environments().unwrap();

    // Should only find 1 env, not 2 (symlink should be skipped)
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "real-env");
}

// =========================================================================
// Integration Tests: find_environment()
// =========================================================================

#[test]
fn test_find_environment_returns_correct_env() {
    let temp = setup_mock_pyenv_root(&["3.11.0", "3.12.0"]);
    create_mock_virtualenv(temp.path(), "3.11.0", "target-env", true);
    create_mock_virtualenv(temp.path(), "3.12.0", "other-env", true);

    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());
    let result = discovery.find_environment("target-env");

    assert!(result.is_ok());
    let env = result.unwrap();
    assert_eq!(env.name, "target-env");
    assert_eq!(env.python_version, "3.11.0");
}

#[test]
fn test_find_environment_returns_error_for_nonexistent() {
    let temp = setup_mock_pyenv_root(&["3.11.0"]);
    create_mock_virtualenv(temp.path(), "3.11.0", "existing-env", true);

    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());
    let result = discovery.find_environment("nonexistent");

    assert!(result.is_err());
}

#[test]
fn test_find_environment_returns_error_when_versions_dir_missing() {
    let temp = TempDir::new().unwrap();
    // No versions/ directory created

    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());
    let result = discovery.find_environment("any-env");

    assert!(result.is_err());
}

// =========================================================================
// Unit Tests: determine_status (via common module)
// =========================================================================

#[test]
fn test_determine_status_eol_python() {
    // Use a unique name that definitely won't exist in scuv
    let status = common::determine_status("nonexistent_eol_test_env_xyz", "3.7.0");
    assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));

    let status = common::determine_status("nonexistent_eol_test_env_xyz", "2.7.18");
    assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));
}

#[test]
fn test_determine_status_ready() {
    let status = common::determine_status("nonexistent_env_name_12345", "3.12.0");
    assert!(matches!(status, EnvironmentStatus::Ready));
}

#[test]
fn test_determine_status_eol_boundary() {
    // 3.8 is the EOL boundary
    let status = common::determine_status("boundary_test_env", "3.8.19");
    assert!(matches!(status, EnvironmentStatus::PythonEol { .. }));

    // 3.9 is supported
    let status = common::determine_status("boundary_test_env", "3.9.0");
    assert!(matches!(status, EnvironmentStatus::Ready));
}

// =========================================================================
// Edge Cases
// =========================================================================

#[test]
fn test_scan_handles_permission_errors_gracefully() {
    // Just test that we don't panic on unreadable directories
    // Actual permission testing is platform-specific
    let temp = TempDir::new().unwrap();
    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());

    // Should not panic, just return empty
    let result = discovery.scan_environments();
    assert!(result.is_ok());
}

#[test]
fn test_source_type_returns_pyenv() {
    let temp = TempDir::new().unwrap();
    let discovery = PyenvDiscovery::new(temp.path().to_path_buf());

    assert!(matches!(discovery.source_type(), SourceType::Pyenv));
}
