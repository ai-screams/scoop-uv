//! Unit tests for `uv::install`.

use super::*;
use rstest::rstest;
use tempfile::TempDir;

const ELIGIBLE: Context = Context {
    machine_output: false,
    unix: true,
    offline: false,
    terminal: true,
};

#[test]
fn check_allows_an_interactive_unix_session() {
    assert_eq!(check(ELIGIBLE), Ok(()));
}

#[rstest]
#[case::machine_output(Context { machine_output: true, ..ELIGIBLE }, Skip::MachineOutput)]
#[case::windows(Context { unix: false, ..ELIGIBLE }, Skip::Unsupported)]
#[case::offline(Context { offline: true, ..ELIGIBLE }, Skip::Offline)]
#[case::no_terminal(Context { terminal: false, ..ELIGIBLE }, Skip::NoTerminal)]
fn check_refuses_each_condition_on_its_own(#[case] context: Context, #[case] skip: Skip) {
    assert_eq!(check(context), Err(skip));
}

#[rstest]
#[case::unset(None, false)]
#[case::empty(Some(""), false)]
#[case::zero(Some("0"), false)]
#[case::falsy(Some("FALSE"), false)]
#[case::no(Some("no"), false)]
#[case::off(Some("Off"), false)]
#[case::one(Some("1"), true)]
#[case::truthy(Some("true"), true)]
fn offline_enabled_reads_the_variable(#[case] value: Option<&str>, #[case] expected: bool) {
    assert_eq!(offline_enabled(value.map(OsStr::new)), expected);
}

#[test]
fn install_dir_is_under_an_absolute_home() {
    assert_eq!(
        install_dir(Some(Path::new("/home/me")), None),
        Some(PathBuf::from("/home/me/.local/bin"))
    );
}

#[rstest]
#[case::no_home(None, None)]
#[case::relative_home(Some("me"), None)]
#[case::is_cargo_home(Some("/home/me"), Some("/home/me/.local/bin"))]
fn install_dir_refuses(#[case] home: Option<&str>, #[case] cargo_home: Option<&str>) {
    let dir = install_dir(home.map(Path::new), cargo_home.map(OsStr::new));
    assert_eq!(dir, None);
}

#[test]
fn install_dir_ignores_an_unrelated_cargo_home() {
    let dir = install_dir(
        Some(Path::new("/home/me")),
        Some(OsStr::new("/home/me/.cargo")),
    );
    assert_eq!(dir, Some(PathBuf::from("/home/me/.local/bin")));
}

#[test]
fn inherited_settings_lists_installer_names_sorted_without_values() {
    let vars = [
        ("UV_DOWNLOAD_URL", "https://token@mirror"),
        ("PATH", "/bin"),
        ("CARGO_DIST_FORCE_INSTALL_DIR", "/x"),
        ("INSTALLER_NO_MODIFY_PATH", "1"),
        ("UV_INSTALL_DIR", "/overridden"),
        ("MY_UV_SETTING", "1"),
    ]
    .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    assert_eq!(
        inherited_settings(vars.into_iter()),
        [
            "CARGO_DIST_FORCE_INSTALL_DIR",
            "INSTALLER_NO_MODIFY_PATH",
            "UV_DOWNLOAD_URL"
        ]
    );
}

fn plan_in(dir: &Path) -> Plan {
    Plan::new(
        "/usr/bin/curl".into(),
        "/bin/sh".into(),
        Path::new("/tmp/x/install.sh"),
        dir.to_path_buf(),
    )
}

#[test]
fn plan_shows_the_exact_commands() {
    let plan = plan_in(Path::new("/home/me/.local/bin"));
    assert_eq!(
        plan.download.display(),
        "/usr/bin/curl -q --proto =https --tlsv1.2 -fsSL --connect-timeout 10 \
             --max-time 60 https://astral.sh/uv/install.sh -o /tmp/x/install.sh"
    );
    assert_eq!(
        plan.run.display(),
        "UV_INSTALL_DIR=/home/me/.local/bin /bin/sh /tmp/x/install.sh"
    );
    assert_eq!(plan.uv, Path::new("/home/me/.local/bin/uv"));
}

#[rstest]
#[case::plain("/a/b-c_d.sh", "/a/b-c_d.sh")]
#[case::space("/a b", "'/a b'")]
#[case::quote("it's", r"'it'\''s'")]
#[case::dollar("$HOME", "'$HOME'")]
#[case::empty("", "''")]
fn quote_keeps_a_word_one_word(#[case] word: &str, #[case] expected: &str) {
    assert_eq!(quote(OsStr::new(word)), expected);
}

#[rstest]
#[case::at_floor("uv 0.5.19", Some("0.5.19"))]
#[case::newer("uv 0.13.6 (Homebrew 2026-05-21 aarch64-apple-darwin)", Some("0.13.6"))]
#[case::below_floor("uv 0.5.18", None)]
#[case::unreadable("uv custom", None)]
fn verified_version_needs_a_supported_release(#[case] raw: &str, #[case] expected: Option<&str>) {
    assert_eq!(verified_version(raw).as_deref(), expected);
}

/// Records calls; each step's answer comes from the test.
struct Fake {
    calls: Vec<&'static str>,
    download: fn() -> io::Result<bool>,
    run: fn() -> io::Result<bool>,
    version: fn() -> io::Result<String>,
}

impl Fake {
    fn working() -> Self {
        Self {
            calls: Vec::new(),
            download: || Ok(true),
            run: || Ok(true),
            version: || Ok("uv 0.13.6".to_string()),
        }
    }
}

impl Runner for Fake {
    fn run(&mut self, step: &Step) -> io::Result<bool> {
        if step.env.is_some() {
            self.calls.push("run");
            (self.run)()
        } else {
            self.calls.push("download");
            (self.download)()
        }
    }

    fn version(&mut self, _uv: &Path) -> io::Result<String> {
        self.calls.push("version");
        (self.version)()
    }
}

fn denied() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, "denied")
}

#[test]
fn install_shows_the_plan_before_anything_runs_then_runs_in_order() {
    let home = TempDir::new().unwrap();
    let plan = plan_in(home.path());
    let mut runner = Fake::working();
    let mut asked = false;

    let outcome = install(
        &plan,
        |shown| {
            asked = true;
            assert_eq!(shown, &plan);
            true
        },
        &mut runner,
    );

    assert!(asked);
    assert_eq!(runner.calls, ["download", "run", "version"]);
    assert_eq!(
        outcome,
        Outcome::Installed {
            path: plan.uv.clone(),
            version: "0.13.6".to_string()
        }
    );
}

#[test]
fn install_runs_nothing_when_declined() {
    let home = TempDir::new().unwrap();
    let mut runner = Fake::working();

    let outcome = install(&plan_in(home.path()), |_| false, &mut runner);

    assert_eq!(outcome, Outcome::Declined);
    assert!(runner.calls.is_empty());
}

#[rstest]
#[case::uv("uv")]
#[case::uvx("uvx")]
fn install_never_asks_over_an_existing_executable(#[case] name: &str) {
    let home = TempDir::new().unwrap();
    std::fs::write(home.path().join(name), "mine").unwrap();
    let mut runner = Fake::working();

    let outcome = install(
        &plan_in(home.path()),
        |_| panic!("asked to confirm an install that cannot happen"),
        &mut runner,
    );

    assert_eq!(
        outcome,
        Outcome::Collision {
            path: home.path().join(name),
            dir: home.path().to_path_buf()
        }
    );
    assert!(runner.calls.is_empty());
}

#[cfg(unix)]
#[test]
fn install_treats_a_dangling_link_as_a_collision() {
    let home = TempDir::new().unwrap();
    std::os::unix::fs::symlink("/nonexistent/uv", home.path().join("uv")).unwrap();

    let outcome = install(&plan_in(home.path()), |_| true, &mut Fake::working());

    assert!(matches!(outcome, Outcome::Collision { .. }), "{outcome:?}");
}

#[test]
fn install_treats_an_uninspectable_path_as_a_collision() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("not-a-dir");
    std::fs::write(&file, "").unwrap();

    let outcome = install(&plan_in(&file), |_| true, &mut Fake::working());

    assert!(matches!(outcome, Outcome::Collision { .. }), "{outcome:?}");
}

#[test]
fn install_rechecks_collisions_after_the_prompt() {
    let home = TempDir::new().unwrap();
    let uv = home.path().join("uv");
    let mut runner = Fake::working();

    let outcome = install(
        &plan_in(home.path()),
        |_| {
            std::fs::write(&uv, "appeared while waiting").unwrap();
            true
        },
        &mut runner,
    );

    assert!(matches!(outcome, Outcome::Collision { .. }), "{outcome:?}");
    assert_eq!(runner.calls, ["download"], "the installer must not run");
}

/// Something else creates uv while the download is in progress.
struct RacedDownload {
    uv: PathBuf,
    ran_installer: bool,
}

impl Runner for RacedDownload {
    fn run(&mut self, step: &Step) -> io::Result<bool> {
        if step.env.is_some() {
            self.ran_installer = true;
        } else {
            std::fs::write(&self.uv, "appeared during the download")?;
        }
        Ok(true)
    }

    fn version(&mut self, _uv: &Path) -> io::Result<String> {
        Ok("uv 0.13.6".to_string())
    }
}

#[test]
fn install_rechecks_collisions_after_the_download() {
    let home = TempDir::new().unwrap();
    let mut runner = RacedDownload {
        uv: home.path().join("uv"),
        ran_installer: false,
    };

    let outcome = install(&plan_in(home.path()), |_| true, &mut runner);

    assert!(matches!(outcome, Outcome::Collision { .. }), "{outcome:?}");
    assert!(!runner.ran_installer);
}

#[rstest]
#[case::exit_status(|| Ok(false), None)]
#[case::spawn(|| Err(denied()), Some("denied"))]
fn install_stops_after_a_failed_download(
    #[case] download: fn() -> io::Result<bool>,
    #[case] detail: Option<&str>,
) {
    let home = TempDir::new().unwrap();
    let mut runner = Fake {
        download,
        ..Fake::working()
    };

    let outcome = install(&plan_in(home.path()), |_| true, &mut runner);

    assert_eq!(runner.calls, ["download"]);
    assert_eq!(
        outcome,
        Outcome::Failed {
            stage: Stage::Download,
            detail: detail.map(str::to_string)
        }
    );
}

#[test]
fn install_does_not_verify_after_a_failed_run() {
    let home = TempDir::new().unwrap();
    let mut runner = Fake {
        run: || Ok(false),
        ..Fake::working()
    };

    let outcome = install(&plan_in(home.path()), |_| true, &mut runner);

    assert_eq!(runner.calls, ["download", "run"]);
    assert_eq!(
        outcome,
        Outcome::Failed {
            stage: Stage::Run,
            detail: None
        }
    );
}

#[rstest]
#[case::fails(|| Err(denied()), "denied")]
#[case::too_old(|| Ok("uv 0.1.0".to_string()), "uv 0.1.0")]
#[case::unreadable(|| Ok("mystery".to_string()), "mystery")]
fn install_fails_when_the_result_does_not_verify(
    #[case] version: fn() -> io::Result<String>,
    #[case] detail: &str,
) {
    let home = TempDir::new().unwrap();
    let mut runner = Fake {
        version,
        ..Fake::working()
    };

    let outcome = install(&plan_in(home.path()), |_| true, &mut runner);

    assert_eq!(
        outcome,
        Outcome::Failed {
            stage: Stage::Verify,
            detail: Some(detail.to_string())
        }
    );
}

/// Writes an executable script; `curl` stands in for the real downloader.
#[cfg(unix)]
fn script(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn process_runner_drives_a_whole_install_with_real_processes() {
    let root = TempDir::new().unwrap();
    let dir = root.path().join("bin dir");
    let curl = root.path().join("curl");
    // The fake download writes the fake installer to the `-o` argument, and
    // that installer writes uv where UV_INSTALL_DIR says. Absolute paths:
    // tests elsewhere in this process empty PATH for a moment.
    script(
        &curl,
        r#"for last; do :; done
/bin/cat > "$last" <<'EOF'
/bin/mkdir -p "$UV_INSTALL_DIR"
printf '#!/bin/sh\necho uv 0.13.6\n' > "$UV_INSTALL_DIR/uv"
/bin/chmod +x "$UV_INSTALL_DIR/uv"
EOF
"#,
    );
    let plan = Plan::new(
        curl,
        "/bin/sh".into(),
        &root.path().join("install.sh"),
        dir.clone(),
    );

    let outcome = install(&plan, |_| true, &mut ProcessRunner);

    assert_eq!(
        outcome,
        Outcome::Installed {
            path: dir.join("uv"),
            version: "0.13.6".to_string()
        }
    );
}

#[cfg(unix)]
#[test]
fn process_runner_reports_unsuccessful_exits() {
    let root = TempDir::new().unwrap();
    let failing = root.path().join("failing");
    script(&failing, "exit 3\n");
    let step = Step {
        program: failing.clone(),
        args: Vec::new(),
        env: None,
    };

    assert!(!ProcessRunner.run(&step).unwrap());
    assert!(ProcessRunner.version(&failing).is_err());
}

// A uv that installs but cannot run explains why on stderr; the report
// must carry that, not only "checking the installed uv failed".
#[cfg(unix)]
#[rstest]
#[case::with_stderr("echo 'missing libfoo.so' >&2\nexit 127\n", "127: missing libfoo.so")]
#[case::silent("exit 3\n", "3")]
fn process_runner_keeps_why_uv_version_failed(#[case] body: &str, #[case] ends: &str) {
    let root = TempDir::new().unwrap();
    let uv = root.path().join("uv");
    script(&uv, body);

    let detail = ProcessRunner.version(&uv).unwrap_err().to_string();

    assert!(detail.ends_with(ends), "{detail}");
}

#[test]
fn process_runner_surfaces_a_program_that_cannot_start() {
    let missing = Path::new("/nonexistent/uv");
    let step = Step {
        program: missing.to_path_buf(),
        args: Vec::new(),
        env: None,
    };

    assert!(ProcessRunner.run(&step).is_err());
    assert!(ProcessRunner.version(missing).is_err());
}
