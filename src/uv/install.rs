//! Installing uv for `scuv doctor --fix`.
//!
//! Only that command reaches this module: running a remote script needs a
//! person who saw the plan and said yes, so ordinary commands and
//! [`UvClient`](super::UvClient) never install anything.
//!
//! The order of the steps lives here rather than in the CLI handler so that it
//! is mutation-tested: confirming, stopping after a failed download and
//! checking the result are the parts that must not silently stop working.

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::version;

const INSTALLER_URL: &str = "https://astral.sh/uv/install.sh";

/// Where to send people the automatic path cannot serve.
pub(crate) const MANUAL_URL: &str = "https://docs.astral.sh/uv/getting-started/installation/";

/// What the Unix installer puts in the install directory.
const EXECUTABLES: [&str; 2] = ["uv", "uvx"];

/// Variable prefixes the installer reads. It honours `UV_DOWNLOAD_URL`,
/// `INSTALLER_DOWNLOAD_URL`, `UV_UNMANAGED_INSTALL` and more, any of which can
/// make it differ from what the plan says.
const INSTALLER_ENV_PREFIXES: [&str; 3] = ["UV_", "INSTALLER_", "CARGO_DIST_"];

/// The one installer variable scuv sets itself.
const INSTALL_DIR_VAR: &str = "UV_INSTALL_DIR";

/// Why `doctor --fix` will not offer to install uv.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Skip {
    /// `--json` or `--quiet`: nowhere to show a plan and ask.
    MachineOutput,
    /// The installer run here is the Unix one.
    Unsupported,
    /// `UV_OFFLINE` is on.
    Offline,
    /// No person at a terminal to answer.
    NoTerminal,
}

/// The facts [`check`] decides on, gathered by the caller.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Context {
    pub(crate) machine_output: bool,
    pub(crate) unix: bool,
    pub(crate) offline: bool,
    /// Both stdin and stderr: the prompt is written to one and read from the
    /// other.
    pub(crate) terminal: bool,
}

/// Decides whether installing may be offered at all.
pub(crate) fn check(context: Context) -> Result<(), Skip> {
    if context.machine_output {
        Err(Skip::MachineOutput)
    } else if !context.unix {
        Err(Skip::Unsupported)
    } else if context.offline {
        Err(Skip::Offline)
    } else if !context.terminal {
        Err(Skip::NoTerminal)
    } else {
        Ok(())
    }
}

/// Reads `UV_OFFLINE`: off when unset, empty, `0`, `false`, `no` or `off`.
pub(crate) fn offline_enabled(value: Option<&OsStr>) -> bool {
    let Some(value) = value else { return false };
    let value = value.to_string_lossy().to_ascii_lowercase();
    !matches!(value.as_str(), "" | "0" | "false" | "no" | "off")
}

/// The directory uv is installed into: `<home>/.local/bin`.
///
/// Fixed instead of following the installer's own default, which also depends
/// on `XDG_BIN_HOME` and `XDG_DATA_HOME`, so the plan can name the exact path
/// it will verify. `None` without an absolute home, and when the directory is
/// `CARGO_HOME`: the installer then nests binaries under `bin/`, and the path
/// in the plan would be wrong.
pub(crate) fn install_dir(home: Option<&Path>, cargo_home: Option<&OsStr>) -> Option<PathBuf> {
    let dir = home.filter(|home| home.is_absolute())?.join(".local/bin");
    (cargo_home.map(Path::new) != Some(dir.as_path())).then_some(dir)
}

/// Names of the inherited installer settings, sorted. Only names: a mirror URL
/// can carry a token, and the plan is shown on screen.
pub(crate) fn inherited_settings(vars: impl Iterator<Item = (OsString, OsString)>) -> Vec<String> {
    let mut names: Vec<String> = vars
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| {
            name != INSTALL_DIR_VAR
                && INSTALLER_ENV_PREFIXES
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
        })
        .collect();
    names.sort();
    names
}

/// One process to run, kept as data so the plan shows what really executes.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Step {
    program: PathBuf,
    args: Vec<OsString>,
    env: Option<(&'static str, PathBuf)>,
}

impl Step {
    /// The command line as the plan shows it.
    pub(crate) fn display(&self) -> String {
        let env = self
            .env
            .iter()
            .map(|(name, value)| format!("{name}={}", quote(value.as_os_str())));
        let command = std::iter::once(quote(self.program.as_os_str()))
            .chain(self.args.iter().map(|arg| quote(arg)));
        env.chain(command).collect::<Vec<_>>().join(" ")
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some((name, value)) = &self.env {
            command.env(name, value);
        }
        command
    }
}

/// Quotes a word for display, so a path with a space stays one word.
fn quote(word: &OsStr) -> String {
    let word = word.to_string_lossy();
    let safe = |c: char| c.is_ascii_alphanumeric() || "-_=./:".contains(c);
    if !word.is_empty() && word.chars().all(safe) {
        word.into_owned()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

/// What will run, in order. Downloading and running are separate steps so a
/// pipe cannot hide a failed download.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    pub(crate) download: Step,
    pub(crate) run: Step,
    pub(crate) dir: PathBuf,
    pub(crate) uv: PathBuf,
}

impl Plan {
    /// Plans a download to `script` and an install into `dir`.
    ///
    /// `-q` comes first because curl only honours it there, and it keeps a
    /// `.curlrc` from changing this download.
    pub(crate) fn new(curl: PathBuf, sh: PathBuf, script: &Path, dir: PathBuf) -> Self {
        let download = [
            "-q",
            "--proto",
            "=https",
            "--tlsv1.2",
            "-fsSL",
            "--connect-timeout",
            "10",
            "--max-time",
            "60",
            INSTALLER_URL,
            "-o",
        ];
        let args = download.iter().map(OsString::from).chain([script.into()]);
        Self {
            download: Step {
                program: curl,
                args: args.collect(),
                env: None,
            },
            run: Step {
                program: sh,
                args: vec![script.into()],
                env: Some((INSTALL_DIR_VAR, dir.clone())),
            },
            uv: dir.join("uv"),
            dir,
        }
    }
}

/// The stage an installation failed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    Download,
    Run,
    Verify,
}

/// How an offered installation ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Installed {
        path: PathBuf,
        version: String,
    },
    Declined,
    /// Something already sits where uv would go. The installer moves files in
    /// without asking, so scuv stops rather than replace it.
    Collision {
        path: PathBuf,
        dir: PathBuf,
    },
    Failed {
        stage: Stage,
        detail: Option<String>,
    },
}

/// Runs processes for [`install`]; a trait so tests can make any step fail.
pub(crate) trait Runner {
    /// Runs a step to the end; `Ok(false)` when it exits unsuccessfully.
    fn run(&mut self, step: &Step) -> io::Result<bool>;

    /// Returns what `uv --version` prints, `None` when it exits unsuccessfully.
    fn version(&mut self, uv: &Path) -> io::Result<Option<String>>;
}

/// The real [`Runner`].
#[derive(Debug)]
pub(crate) struct ProcessRunner;

impl Runner for ProcessRunner {
    fn run(&mut self, step: &Step) -> io::Result<bool> {
        // stdout joins stderr: doctor's own stdout stays its report.
        let status = step
            .command()
            .stdin(Stdio::null())
            .stdout(Stdio::from(io::stderr()))
            .status()?;
        Ok(status.success())
    }

    fn version(&mut self, uv: &Path) -> io::Result<Option<String>> {
        let output = Command::new(uv)
            .arg("--version")
            .stdin(Stdio::null())
            .output()?;
        Ok(output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string()))
    }
}

/// Finds an existing entry where the installer would write. A dangling link
/// counts, and an entry that cannot be inspected counts too: only a clear
/// "nothing there" lets the install go ahead.
fn collision(dir: &Path) -> Option<PathBuf> {
    EXECUTABLES.iter().map(|name| dir.join(name)).find(
        |path| !matches!(path.symlink_metadata(), Err(e) if e.kind() == io::ErrorKind::NotFound),
    )
}

/// Returns the version if `raw` proves a supported uv. Stricter than the
/// doctor check, which lets an unreadable banner pass: there uv already
/// existed, here scuv is claiming success for what it just installed.
fn verified_version(raw: &str) -> Option<String> {
    version::parse(raw)
        .filter(|parsed| version::meets_minimum(*parsed))
        .map(version::format_version)
}

fn run_step(runner: &mut impl Runner, step: &Step, stage: Stage) -> Result<(), Outcome> {
    match runner.run(step) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Outcome::Failed {
            stage,
            detail: None,
        }),
        Err(e) => Err(Outcome::Failed {
            stage,
            detail: Some(e.to_string()),
        }),
    }
}

fn verify(runner: &mut impl Runner, uv: &Path) -> Outcome {
    let (raw, detail) = match runner.version(uv) {
        Ok(Some(raw)) => (Some(raw.clone()), Some(raw)),
        Ok(None) => (None, None),
        Err(e) => (None, Some(e.to_string())),
    };
    match raw.as_deref().and_then(verified_version) {
        Some(version) => Outcome::Installed {
            path: uv.to_path_buf(),
            version,
        },
        None => Outcome::Failed {
            stage: Stage::Verify,
            detail,
        },
    }
}

/// Asks, then downloads, runs and verifies, stopping at the first failure.
///
/// Collisions are checked before asking, so nobody confirms an install that
/// cannot happen, and again after, because the prompt can wait for a long time.
pub(crate) fn install(
    plan: &Plan,
    confirm: impl FnOnce(&Plan) -> bool,
    runner: &mut impl Runner,
) -> Outcome {
    let collided = |path| Outcome::Collision {
        path,
        dir: plan.dir.clone(),
    };
    if let Some(path) = collision(&plan.dir) {
        return collided(path);
    }
    if !confirm(plan) {
        return Outcome::Declined;
    }
    if let Some(path) = collision(&plan.dir) {
        return collided(path);
    }
    let steps = [(&plan.download, Stage::Download), (&plan.run, Stage::Run)];
    for (step, stage) in steps {
        if let Err(outcome) = run_step(runner, step, stage) {
            return outcome;
        }
    }
    verify(runner, &plan.uv)
}

#[cfg(test)]
mod tests {
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
    fn verified_version_needs_a_supported_release(
        #[case] raw: &str,
        #[case] expected: Option<&str>,
    ) {
        assert_eq!(verified_version(raw).as_deref(), expected);
    }

    /// Records calls; each step's answer comes from the test.
    struct Fake {
        calls: Vec<&'static str>,
        download: fn() -> io::Result<bool>,
        run: fn() -> io::Result<bool>,
        version: fn() -> io::Result<Option<String>>,
    }

    impl Fake {
        fn working() -> Self {
            Self {
                calls: Vec::new(),
                download: || Ok(true),
                run: || Ok(true),
                version: || Ok(Some("uv 0.13.6".to_string())),
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

        fn version(&mut self, _uv: &Path) -> io::Result<Option<String>> {
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
        assert!(runner.calls.is_empty());
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
    #[case::exit_status(|| Ok(None), None)]
    #[case::spawn(|| Err(denied()), Some("denied"))]
    #[case::too_old(|| Ok(Some("uv 0.1.0".to_string())), Some("uv 0.1.0"))]
    #[case::unreadable(|| Ok(Some("mystery".to_string())), Some("mystery"))]
    fn install_fails_when_the_result_does_not_verify(
        #[case] version: fn() -> io::Result<Option<String>>,
        #[case] detail: Option<&str>,
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
                detail: detail.map(str::to_string)
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
        // that installer writes uv where UV_INSTALL_DIR says.
        script(
            &curl,
            r#"for last; do :; done
cat > "$last" <<'EOF'
mkdir -p "$UV_INSTALL_DIR"
printf '#!/bin/sh\necho uv 0.13.6\n' > "$UV_INSTALL_DIR/uv"
chmod +x "$UV_INSTALL_DIR/uv"
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
        assert_eq!(ProcessRunner.version(&failing).unwrap(), None);
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
}
