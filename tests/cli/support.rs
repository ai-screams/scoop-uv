//! Shared fixtures and helpers for the CLI tests.

pub use assert_cmd::Command;
pub use predicates::prelude::*;
pub use std::path::PathBuf;
pub use tempfile::TempDir;

/// Test fixture for scoop tests
pub(crate) struct TestFixture {
    /// Temporary directory: holds `scoop_home` (and any project directory a
    /// test creates) until the fixture is dropped
    pub(crate) temp_dir: TempDir,
    /// SCUV_HOME path
    pub(crate) scoop_home: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated SCUV_HOME
    pub(crate) fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let scoop_home = temp_dir.path().join(".scuv");
        Self {
            temp_dir,
            scoop_home,
        }
    }
}

/// Helper to get a fresh command with SCUV_HOME set
pub(crate) fn scoop_cmd(scoop_home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("scuv").unwrap();
    cmd.env("SCUV_HOME", scoop_home);
    // Force English locale for consistent test assertions
    cmd.env("SCUV_LANG", "en");
    cmd
}

/// Locate a shell binary: PATH first, then the usual install prefixes. When
/// `SCUV_REQUIRE_<NAME>` is set (`SCUV_REQUIRE_FISH`, `SCUV_REQUIRE_ZSH`,
/// `SCUV_REQUIRE_PWSH`) a missing shell is a failure, not a skip: the CI Test
/// and MSRV jobs install those shells and set the variables, so a silently
/// skipped test cannot look like a pass there. Jobs that run the suite
/// without them (coverage, mutants) leave the variables unset and skip.
pub(crate) fn find_shell(name: &str) -> Option<std::path::PathBuf> {
    let from_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|p| p.is_file())
    });
    let found = from_path.or_else(|| {
        ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"]
            .iter()
            .map(|dir| std::path::Path::new(dir).join(name))
            .find(|p| p.is_file())
    });
    let require = format!("SCUV_REQUIRE_{}", name.to_ascii_uppercase());
    if found.is_none() && std::env::var_os(&require).is_some() {
        panic!("{require} is set but no {name} binary was found");
    }
    found
}

pub(crate) const ESC: &str = "\x1b[";

/// A home with one env, `demo`, that `SCUV_ACTIVE` marks active so `list`
/// highlights it (the only colored stdout).
pub(crate) fn fixture_with_active_env() -> TestFixture {
    let fixture = TestFixture::new();
    let env = fixture.scoop_home.join("virtualenvs").join("demo");
    std::fs::create_dir_all(env.join("bin")).unwrap();
    std::fs::write(env.join("pyvenv.cfg"), "version = 3.12.1\n").unwrap();
    fixture
}

/// A directory holding a fake `uv` (unix): `--version`, an empty
/// `python list` and `cache prune`. Put it first on PATH so commands get
/// past their uv lookup without a real uv. Mirrors `test_utils::FakeUv`,
/// which the lib tests use; integration tests cannot reach that module.
#[cfg(unix)]
pub(crate) fn fake_uv_dir() -> TempDir {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let uv = dir.path().join("uv");
    std::fs::write(
        &uv,
        "#!/bin/sh\ncase \"$1 $2\" in\n  \"--version \"*) echo \"uv 0.12.22\" ;;\n  \"python list\") echo \"[]\" ;;\n  \"cache prune\") ;;\n  *) echo \"fake uv: unsupported: $*\" >&2; exit 2 ;;\nesac\n",
    )
    .unwrap();
    std::fs::set_permissions(&uv, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir
}
