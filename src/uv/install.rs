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

    /// Returns what `uv --version` prints. An unsuccessful exit is an error
    /// that carries the exit status and stderr, so the report can show why.
    fn version(&mut self, uv: &Path) -> io::Result<String>;
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

    fn version(&mut self, uv: &Path) -> io::Result<String> {
        let output = Command::new(uv)
            .arg("--version")
            .stdin(Stdio::null())
            .output()?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        Err(io::Error::other(if stderr.is_empty() {
            output.status.to_string()
        } else {
            format!("{}: {stderr}", output.status)
        }))
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
        Ok(raw) => (Some(raw.clone()), raw),
        Err(e) => (None, e.to_string()),
    };
    match raw.as_deref().and_then(verified_version) {
        Some(version) => Outcome::Installed {
            path: uv.to_path_buf(),
            version,
        },
        None => Outcome::Failed {
            stage: Stage::Verify,
            detail: Some(detail),
        },
    }
}

/// Asks, then downloads, runs and verifies, stopping at the first failure.
///
/// Collisions are checked before asking, so nobody confirms an install that
/// cannot happen, and again right before the installer runs, because the
/// prompt and the download can take a long time and it overwrites unasked.
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
    if let Err(outcome) = run_step(runner, &plan.download, Stage::Download) {
        return outcome;
    }
    if let Some(path) = collision(&plan.dir) {
        return collided(path);
    }
    if let Err(outcome) = run_step(runner, &plan.run, Stage::Run) {
        return outcome;
    }
    verify(runner, &plan.uv)
}

#[cfg(test)]
mod tests;
