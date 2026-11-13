//! What an env has installed and where each package came from.
//!
//! `uv pip list` gives the inventory (name, version). The origin of a
//! package that did not come from a registry (an editable or local-path
//! install, an archive URL, a VCS checkout) is the PEP 610
//! `direct_url.json` its installer wrote; [`read_installed_packages`] reads
//! those through the env's own interpreter with `importlib.metadata`, so no
//! site-packages path has to be guessed. A package without a
//! `direct_url.json` has no recorded direct origin, which is not proof that
//! it came from PyPI.
//!
//! [`InstallPlan`] turns the packages back into `uv pip install` arguments,
//! and [`preflight`] checks, before `--force` deletes anything, that every
//! local source still exists and does not depend on the env being replaced.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ScoopError};
use crate::uv::UvPipListEntry;

/// PEP 503 name normalisation: lowercase, extras dropped, runs of `-`, `_`
/// and `.` folded to one `-`. Matching and comparison key; the original
/// spelling is kept for display.
///
/// # Examples
///
/// ```
/// use scoop_uv::core::packages::canonical_name;
///
/// assert_eq!(canonical_name("Foo__Bar.baz[extra]"), "foo-bar-baz");
/// ```
pub fn canonical_name(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    let trimmed = lower.split('[').next().unwrap_or(&lower);
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_dash = false;
    for c in trimmed.chars() {
        if c == '-' || c == '_' || c == '.' {
            if !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        } else {
            out.push(c);
            prev_dash = false;
        }
    }
    out
}

// ============================================================================
// PEP 610 direct_url.json
// ============================================================================

/// `direct_url.json` as PEP 610 writes it. Unknown fields are ignored,
/// except two uv adds that change what gets installed (`path`, and
/// `vcs_info.git_lfs`): they are read so an origin carrying them is refused
/// instead of reproduced without them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectUrl {
    url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subdirectory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dir_info: Option<DirInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vcs_info: Option<VcsInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_info: Option<ArchiveInfo>,
    /// uv's file inside a git repository (`git+..#path=dist/p.whl`).
    #[serde(default, skip_serializing)]
    path: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DirInfo {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct VcsInfo {
    vcs: String,
    commit_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    requested_revision: Option<String>,
    #[serde(default, skip_serializing)]
    git_lfs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArchiveInfo {
    /// Legacy single `<alg>=<hex>` form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hashes: Option<BTreeMap<String, String>>,
}

/// Where a package came from, checked: exactly one kind, the fields that
/// kind needs, and hashes that agree. Serialises as the PEP 610 object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "DirectUrl", into = "DirectUrl")]
pub enum Origin {
    /// A local directory (`dir_info`), editable or not.
    Dir {
        url: String,
        editable: bool,
        subdirectory: Option<String>,
    },
    /// An archive URL (`archive_info`): a wheel or sdist, local or remote.
    Archive {
        url: String,
        subdirectory: Option<String>,
        hashes: BTreeMap<String, String>,
    },
    /// A VCS checkout (`vcs_info`) pinned to the commit that was installed.
    Vcs {
        vcs: String,
        url: String,
        commit_id: String,
        requested_revision: Option<String>,
        subdirectory: Option<String>,
    },
}

/// An empty or `.` subdirectory means the root.
fn subdirectory(raw: Option<String>) -> Option<String> {
    raw.filter(|s| !s.is_empty() && s != ".")
}

impl TryFrom<DirectUrl> for Origin {
    type Error = String;

    fn try_from(d: DirectUrl) -> std::result::Result<Self, String> {
        if d.url.trim().is_empty() {
            return Err("direct_url.json has an empty url".to_string());
        }
        if d.path.is_some() {
            return Err(
                "direct_url.json has a path (a file inside a repository), which scuv does not reproduce"
                    .to_string(),
            );
        }
        let subdirectory = subdirectory(d.subdirectory);
        match (d.dir_info, d.vcs_info, d.archive_info) {
            (Some(dir), None, None) => {
                if !is_file_url(&d.url) {
                    return Err(format!(
                        "a local directory origin needs a file: URL, got {}",
                        d.url
                    ));
                }
                Ok(Origin::Dir {
                    url: d.url,
                    editable: dir.editable,
                    subdirectory,
                })
            }
            (None, Some(vcs), None) => {
                if vcs.vcs.trim().is_empty() || vcs.commit_id.trim().is_empty() {
                    return Err("vcs_info needs vcs and commit_id".to_string());
                }
                if vcs.git_lfs {
                    return Err("vcs_info has git_lfs, which scuv does not reproduce".to_string());
                }
                Ok(Origin::Vcs {
                    vcs: vcs.vcs,
                    url: d.url,
                    commit_id: vcs.commit_id,
                    requested_revision: vcs.requested_revision,
                    subdirectory,
                })
            }
            (None, None, Some(archive)) => Ok(Origin::Archive {
                url: d.url,
                subdirectory,
                hashes: archive_hashes(archive)?,
            }),
            _ => Err(
                "direct_url.json needs exactly one of dir_info, vcs_info, archive_info".to_string(),
            ),
        }
    }
}

/// The archive's hashes as `{alg: hex}`, the legacy `hash` folded in.
fn archive_hashes(archive: ArchiveInfo) -> std::result::Result<BTreeMap<String, String>, String> {
    let mut hashes: BTreeMap<String, String> = archive
        .hashes
        .unwrap_or_default()
        .into_iter()
        .map(|(alg, hex)| (alg.to_ascii_lowercase(), hex.to_ascii_lowercase()))
        .collect();
    if let Some(legacy) = archive.hash {
        let (alg, hex) = legacy
            .split_once('=')
            .ok_or_else(|| format!("archive hash {legacy:?} is not <alg>=<hex>"))?;
        let (alg, hex) = (alg.to_ascii_lowercase(), hex.to_ascii_lowercase());
        match hashes.get(&alg) {
            Some(existing) if *existing != hex => {
                return Err(format!("archive hash and hashes disagree for {alg}"));
            }
            _ => {
                hashes.insert(alg, hex);
            }
        }
    }
    Ok(hashes)
}

impl From<Origin> for DirectUrl {
    fn from(o: Origin) -> Self {
        match o {
            Origin::Dir {
                url,
                editable,
                subdirectory,
            } => DirectUrl {
                url,
                subdirectory,
                dir_info: Some(DirInfo { editable }),
                vcs_info: None,
                archive_info: None,
                path: None,
            },
            Origin::Archive {
                url,
                subdirectory,
                hashes,
            } => DirectUrl {
                url,
                subdirectory,
                dir_info: None,
                vcs_info: None,
                archive_info: Some(ArchiveInfo {
                    hash: None,
                    hashes: (!hashes.is_empty()).then_some(hashes),
                }),
                path: None,
            },
            Origin::Vcs {
                vcs,
                url,
                commit_id,
                requested_revision,
                subdirectory,
            } => DirectUrl {
                url,
                subdirectory,
                dir_info: None,
                vcs_info: Some(VcsInfo {
                    vcs,
                    commit_id,
                    requested_revision,
                    git_lfs: false,
                }),
                archive_info: None,
                path: None,
            },
        }
    }
}

/// Whether two installs of one package differ in where they came from,
/// by [`Origin::identity`]. No recorded origin on both sides is the same.
///
/// # Examples
///
/// ```
/// use scoop_uv::core::packages::origins_differ;
///
/// assert!(!origins_differ(None, None));
/// ```
pub fn origins_differ(a: Option<&Origin>, b: Option<&Origin>) -> bool {
    a.map(Origin::identity) != b.map(Origin::identity)
}

/// What two origins are compared by. Recorded data that does not change
/// what gets installed (a VCS `requested_revision`, a URL spelling of the
/// same directory) is left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginIdentity {
    Dir {
        /// The effective path's components as written, `..` kept: two
        /// spellings that only resolve to one place compare unequal.
        path: Vec<OsString>,
        editable: bool,
    },
    Archive {
        url: String,
        subdirectory: Option<String>,
        hashes: BTreeMap<String, String>,
    },
    Vcs {
        vcs: String,
        url: String,
        commit_id: String,
        subdirectory: Option<String>,
    },
}

impl Origin {
    /// The local path this origin installs from, if it is a `file:` URL:
    /// the directory (with its subdirectory), the archive file, or the
    /// local repository.
    ///
    /// # Errors
    ///
    /// A `file:` URL that cannot be turned into a path.
    pub fn local_path(&self) -> Option<std::result::Result<PathBuf, String>> {
        match self {
            Origin::Dir {
                url, subdirectory, ..
            } => Some(file_url_to_path(url).map(|p| join_subdirectory(p, subdirectory.as_deref()))),
            // An archive URL may carry `#sha256=` (uv records it so).
            Origin::Archive { url, .. } => is_file_url(url).then(|| {
                file_url_to_path(url.split_once('#').map_or(url.as_str(), |(base, _)| base))
            }),
            Origin::Vcs { url, .. } => is_file_url(url).then(|| file_url_to_path(url)),
        }
    }

    /// The comparison key (see [`OriginIdentity`]).
    pub fn identity(&self) -> OriginIdentity {
        match self {
            Origin::Dir {
                url,
                editable,
                subdirectory,
            } => {
                let path = match file_url_to_path(url) {
                    Ok(p) => join_subdirectory(p, subdirectory.as_deref()),
                    // Not a usable path: compare the URL as written.
                    Err(_) => join_subdirectory(PathBuf::from(url), subdirectory.as_deref()),
                };
                OriginIdentity::Dir {
                    path: path
                        .components()
                        .map(|c| c.as_os_str().to_os_string())
                        .collect(),
                    editable: *editable,
                }
            }
            // Fragment merged as for installing: uv's `p.whl#sha256=H` and
            // pip's `p.whl` with `sha256: H` are one origin.
            Origin::Archive {
                url,
                subdirectory,
                hashes,
            } => {
                let split = split_archive_url(url, subdirectory, hashes).unwrap_or(ArchiveUrl {
                    url: url.clone(),
                    subdirectory: subdirectory.clone(),
                    hashes: hashes.clone(),
                });
                OriginIdentity::Archive {
                    url: split.url,
                    subdirectory: split.subdirectory,
                    hashes: split.hashes,
                }
            }
            Origin::Vcs {
                vcs,
                url,
                commit_id,
                subdirectory,
                ..
            } => OriginIdentity::Vcs {
                vcs: vcs.clone(),
                url: url.clone(),
                commit_id: commit_id.clone(),
                subdirectory: subdirectory.clone(),
            },
        }
    }

    /// A short human description: `editable /path`, `directory /path`,
    /// `archive <url>`, `git <url>@<commit>`.
    pub fn describe(&self) -> String {
        let place = |url: &str, sub: &Option<String>| {
            let base = file_url_to_path(url)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| url.to_string());
            match sub {
                Some(s) => format!("{base} ({s})"),
                None => base,
            }
        };
        match self {
            Origin::Dir {
                url,
                editable: true,
                subdirectory,
            } => format!("editable {}", place(url, subdirectory)),
            Origin::Dir {
                url, subdirectory, ..
            } => format!("directory {}", place(url, subdirectory)),
            Origin::Archive {
                url, subdirectory, ..
            } => format!("archive {}", place(url, subdirectory)),
            Origin::Vcs {
                vcs,
                url,
                commit_id,
                subdirectory,
                ..
            } => format!("{vcs} {}@{commit_id}", place(url, subdirectory)),
        }
    }
}

fn join_subdirectory(path: PathBuf, subdirectory: Option<&str>) -> PathBuf {
    match subdirectory {
        Some(s) => path.join(s),
        None => path,
    }
}

// ============================================================================
// file: URLs
// ============================================================================

/// The local path a `file:` URL names: `file:///p`, `file://localhost/p`
/// and `file:/p`, percent-decoded, the scheme in any case. A Windows drive (`file:///C:/p`) keeps
/// its letter. A remote authority (`file://host/share`, UNC) is not
/// supported, nor is a query or fragment.
///
/// # Examples
///
/// ```
/// use scoop_uv::core::packages::file_url_to_path;
/// # #[cfg(unix)]
/// assert_eq!(
///     file_url_to_path("file:///home/u/my%20proj").unwrap(),
///     std::path::PathBuf::from("/home/u/my proj")
/// );
/// ```
///
/// # Errors
///
/// A URL that is not `file:`, has a remote authority, a query or fragment,
/// bad percent-encoding, or (on Windows) does not decode to UTF-8.
pub fn file_url_to_path(url: &str) -> std::result::Result<PathBuf, String> {
    let rest = strip_file_scheme(url).ok_or_else(|| format!("{url} is not a file: URL"))?;
    if rest.contains('?') || rest.contains('#') {
        return Err(format!("{url} has a query or fragment"));
    }
    let path = if let Some(after) = rest.strip_prefix("//") {
        let (authority, path) = after.split_at(after.find('/').unwrap_or(after.len()));
        if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") {
            return Err(format!("{url} names a remote host, which is not supported"));
        }
        path
    } else {
        rest
    };
    if !path.starts_with('/') {
        return Err(format!("{url} has no absolute path"));
    }
    let bytes = percent_decode(path).ok_or_else(|| format!("{url} has bad percent-encoding"))?;
    #[cfg(unix)]
    return bytes_to_path(bytes, url);
    #[cfg(not(unix))]
    return bytes_to_windows_path(bytes, url);
}

/// `url` after its `file:` scheme, which (like any URL scheme) matches in
/// any case.
fn strip_file_scheme(url: &str) -> Option<&str> {
    url.get(..5)
        .filter(|scheme| scheme.eq_ignore_ascii_case("file:"))
        .map(|_| &url[5..])
}

fn is_file_url(url: &str) -> bool {
    strip_file_scheme(url).is_some()
}

/// `url` with a `file:` scheme in lower case: uv reads `FILE:///p` as a
/// relative path.
fn lowercase_file_scheme(url: &str) -> String {
    match strip_file_scheme(url) {
        Some(rest) => format!("file:{rest}"),
        None => url.to_string(),
    }
}

/// `file://` and `path`, percent-encoded so [`file_url_to_path`] reads it
/// back.
fn path_to_file_url(path: &Path) -> String {
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    };
    #[cfg(not(unix))]
    let bytes = windows_url_path(path);
    format!("file://{}", percent_encode(&bytes))
}

/// Every byte but ASCII letters, digits and `/-._~:` as `%XX`.
fn percent_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b.is_ascii_alphanumeric() || b"/-._~:".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `C:\x` -> `/C:/x`. Not compiled on Unix, so mutants of it are excluded
/// there (`.cargo/mutants.toml`).
#[cfg(not(unix))]
fn windows_url_path(path: &Path) -> Vec<u8> {
    format!("/{}", path.to_string_lossy().replace('\\', "/")).into_bytes()
}

/// `%XX` decoding; `None` on a malformed escape.
fn percent_decode(s: &str) -> Option<Vec<u8>> {
    let digit = |b: Option<u8>| char::from(b?).to_digit(16);
    let mut out = Vec::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let (high, low) = (digit(bytes.next())?, digit(bytes.next())?);
            out.push(u8::try_from(high * 16 + low).ok()?);
        } else {
            out.push(b);
        }
    }
    Some(out)
}

#[cfg(unix)]
fn bytes_to_path(bytes: Vec<u8>, _url: &str) -> std::result::Result<PathBuf, String> {
    use std::os::unix::ffi::OsStringExt;
    Ok(PathBuf::from(OsString::from_vec(bytes)))
}

/// Not compiled on Unix, so mutants of it are excluded there
/// (`.cargo/mutants.toml`); the drive handling is [`strip_drive_slash`].
#[cfg(not(unix))]
fn bytes_to_windows_path(bytes: Vec<u8>, url: &str) -> std::result::Result<PathBuf, String> {
    let s = String::from_utf8(bytes).map_err(|_| format!("{url} is not UTF-8"))?;
    Ok(PathBuf::from(strip_drive_slash(&s)))
}

/// `/C:/x` -> `C:/x`: a URL path before a Windows drive letter.
#[cfg_attr(unix, allow(dead_code))]
fn strip_drive_slash(path: &str) -> &str {
    match path.as_bytes() {
        [b'/', d, b':', ..] if d.is_ascii_alphabetic() => &path[1..],
        _ => path,
    }
}

// ============================================================================
// Reading
// ============================================================================

/// One installed package and its recorded origin, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPackage {
    /// The name as uv lists it.
    pub name: String,
    pub version: String,
    /// `None`: the interpreter found this distribution and it has no
    /// `direct_url.json`.
    pub origin: Option<Origin>,
    /// The tool that installed it, from the distribution's `INSTALLER`
    /// file (`pip`, `uv`, `conda`), when it has one.
    pub installer: Option<String>,
}

/// How long the env's interpreter may take to list its distributions.
pub const ORIGIN_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Above this, the interpreter's origin listing is refused, not trimmed.
const STDOUT_CAP: u64 = 8 * 1024 * 1024;
/// The interpreter's stderr is kept up to this for the error message.
const STDERR_CAP: u64 = 64 * 1024;

const MARKER: &str = "SCUV_ORIGINS:";
const TOO_OLD_MARKER: &str = "SCUV_ORIGINS_PYTHON:";

/// Run with `-I`, so the working directory and `PYTHON*` variables do not
/// change what it sees. The env's own site hooks still run: `-I` is not a
/// sandbox. Only the marker line is read, so anything a hook prints is
/// ignored. Written for Python 2 syntax too, so an old interpreter reports
/// its version instead of a syntax error.
const ORIGIN_SCRIPT: &str = r#"import sys, json
if sys.version_info < (3, 8):
    sys.stdout.write("SCUV_ORIGINS_PYTHON:%d.%d\n" % tuple(sys.version_info[:2]))
    sys.exit(0)
import importlib.metadata as m
rows = []
for d in m.distributions():
    try:
        name = d.metadata["Name"]
    except Exception:
        name = None
    installer = d.read_text("INSTALLER")
    rows.append([name, d.version, d.read_text("direct_url.json"), installer and installer.strip()])
sys.stdout.write("SCUV_ORIGINS:" + json.dumps(rows) + "\n")
"#;

/// Every package in the env at `env_path`, with its origin.
///
/// # Errors
///
/// [`ScoopError::UvNotFound`] or the `uv pip list` error;
/// [`ScoopError::PackageOriginsUnreadable`] when the interpreter cannot be
/// run, is older than 3.8, times out or prints no usable listing, when a
/// `direct_url.json` is malformed, or when a package uv lists cannot be
/// matched to exactly one origin.
pub fn read_installed_packages(env_path: &Path) -> Result<Vec<InstalledPackage>> {
    read_installed_packages_with(&crate::uv::UvClient::new()?, env_path)
}

/// [`read_installed_packages`] with a uv client the caller already has.
///
/// # Errors
///
/// As [`read_installed_packages`], without the `UvNotFound` case.
pub fn read_installed_packages_with(
    uv: &crate::uv::UvClient,
    env_path: &Path,
) -> Result<Vec<InstalledPackage>> {
    let listed = uv.pip_list(env_path)?;
    if listed.is_empty() {
        return Ok(Vec::new());
    }
    let python = crate::paths::virtualenv_python_exe(env_path);
    let rows = run_origin_script(&python, ORIGIN_READ_TIMEOUT)
        .map_err(|reason| unreadable(env_path, reason))?;
    reconcile(listed, rows).map_err(|reason| unreadable(env_path, reason))
}

fn unreadable(env_path: &Path, reason: String) -> ScoopError {
    ScoopError::PackageOriginsUnreadable {
        env: env_path.display().to_string(),
        reason,
    }
}

/// One distribution as the interpreter reports it: name, version,
/// `direct_url.json` and `INSTALLER`. A row of three (no installer) is
/// accepted as one with no `INSTALLER`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<Option<String>>")]
pub(crate) struct OriginRow(
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl TryFrom<Vec<Option<String>>> for OriginRow {
    type Error = String;

    fn try_from(mut fields: Vec<Option<String>>) -> std::result::Result<Self, String> {
        if !(3..=4).contains(&fields.len()) {
            return Err(format!("a row has {} fields, not 3 or 4", fields.len()));
        }
        fields.resize(4, None);
        let mut fields = fields.into_iter();
        let mut next = || fields.next().flatten();
        Ok(Self(next(), next(), next(), next()))
    }
}

/// Runs [`ORIGIN_SCRIPT`] with `python` and returns its rows.
///
/// stdout and stderr go to files, not pipes: a process the env's site
/// hooks start can keep an inherited pipe open, and a reader waiting for
/// EOF would never finish. The wait is bounded by `timeout` (the child is
/// killed and reaped past it), and each file is read once, up to its cap,
/// from a handle of its own.
pub(crate) fn run_origin_script(
    python: &Path,
    timeout: Duration,
) -> std::result::Result<Vec<OriginRow>, String> {
    let stdout = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    let stderr = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    let mut child = Command::new(python)
        .arg("-I")
        .arg("-c")
        .arg(ORIGIN_SCRIPT)
        .stdin(Stdio::null())
        .stdout(stdout.reopen().map_err(|e| e.to_string())?)
        .stderr(stderr.reopen().map_err(|e| e.to_string())?)
        .spawn()
        .map_err(|e| {
            t!(
                "packages.interpreter_unrunnable",
                python = python.display(),
                error = e
            )
            .to_string()
        })?;

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(
                    t!("packages.interpreter_timeout", seconds = timeout.as_secs()).to_string(),
                );
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };

    let (out, _) = read_capped(stdout.path(), STDOUT_CAP)?;
    let (err, err_truncated) = read_capped(stderr.path(), STDERR_CAP)?;
    let err_text = {
        let mut s = String::from_utf8_lossy(&err).trim().to_string();
        if err_truncated {
            s.push_str(" [...]");
        }
        s
    };
    if !status.success() {
        return Err(t!(
            "packages.interpreter_failed",
            status = status,
            stderr = err_text
        )
        .to_string());
    }
    parse_origin_output(&out)
}

/// Up to `cap` bytes of the file at `path`, read through a fresh handle,
/// and whether there was more. The length is taken once, so a process
/// still writing cannot extend the read.
fn read_capped(path: &Path, cap: u64) -> std::result::Result<(Vec<u8>, bool), String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut buf = Vec::new();
    file.take(len.min(cap))
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok((buf, len > cap))
}

/// The rows from the first complete marker line in `out`.
pub(crate) fn parse_origin_output(out: &[u8]) -> std::result::Result<Vec<OriginRow>, String> {
    if out.len() as u64 >= STDOUT_CAP {
        return Err(t!(
            "packages.listing_too_large",
            mib = STDOUT_CAP / (1024 * 1024)
        )
        .to_string());
    }
    let text = String::from_utf8_lossy(out);
    // Only newline-terminated lines: a marker cut off by the read is not one.
    let mut complete = text.split_inclusive('\n').filter(|l| l.ends_with('\n'));
    for line in complete.by_ref() {
        let line = line.trim_end_matches(['\n', '\r']);
        if let Some(version) = line.strip_prefix(TOO_OLD_MARKER) {
            return Err(t!("packages.python_too_old", version = version).to_string());
        }
        if let Some(json) = line.strip_prefix(MARKER) {
            return serde_json::from_str(json)
                .map_err(|e| t!("packages.listing_unparseable", error = e).to_string());
        }
    }
    Err(t!("packages.listing_missing").to_string())
}

/// Joins uv's inventory with the interpreter's rows by canonical name and
/// version. Every uv package must match rows that agree on one origin and
/// one installer. A package uv calls editable must have an editable
/// directory origin; an old `setup.py develop` install (an `.egg-link`)
/// records none, and then the origin is the editable project location uv
/// reports.
pub(crate) fn reconcile(
    listed: Vec<UvPipListEntry>,
    rows: Vec<OriginRow>,
) -> std::result::Result<Vec<InstalledPackage>, String> {
    type Found = (Option<Origin>, Option<String>);
    let mut by_key: BTreeMap<(String, String), Vec<Found>> = BTreeMap::new();
    for OriginRow(name, version, direct_url, installer) in rows {
        let (Some(name), Some(version)) = (name, version) else {
            continue;
        };
        let origin = match direct_url {
            None => None,
            Some(raw) => Some(serde_json::from_str::<Origin>(&raw).map_err(|e| {
                t!("packages.bad_direct_url", package = &name, error = e).to_string()
            })?),
        };
        let installer = installer.filter(|i| !i.is_empty());
        by_key
            .entry((canonical_name(&name), version))
            .or_default()
            .push((origin, installer));
    }

    listed
        .into_iter()
        .map(|entry| {
            let key = (canonical_name(&entry.name), entry.version.clone());
            let found = by_key.get(&key).ok_or_else(|| {
                t!(
                    "packages.not_seen",
                    package = &entry.name,
                    version = &entry.version
                )
                .to_string()
            })?;
            let (origin, installer) = found[0].clone();
            if found.iter().any(|(o, _)| *o != origin) {
                return Err(t!(
                    "packages.ambiguous",
                    package = &entry.name,
                    version = &entry.version
                )
                .to_string());
            }
            // The installer decides how migrate treats a package (conda's
            // builds go to the registry), so rows must agree on it too.
            if found.iter().any(|(_, i)| *i != installer) {
                return Err(t!(
                    "packages.installer_ambiguous",
                    package = &entry.name,
                    version = &entry.version
                )
                .to_string());
            }
            let origin = match (origin, entry.editable_project_location.as_deref()) {
                (Some(o @ Origin::Dir { editable: true, .. }), _) => Some(o),
                // An `.egg-link` install records no direct_url.json; uv
                // still reports where the project is.
                (None, Some(location)) if Path::new(location).is_absolute() => Some(Origin::Dir {
                    url: path_to_file_url(Path::new(location)),
                    editable: true,
                    subdirectory: None,
                }),
                (_, Some(_)) => {
                    return Err(
                        t!("packages.editable_without_origin", package = &entry.name).to_string(),
                    );
                }
                (origin, None) => origin,
            };
            Ok(InstalledPackage {
                name: entry.name,
                version: entry.version,
                origin,
                installer,
            })
        })
        .collect()
}

// ============================================================================
// Installing
// ============================================================================

/// A package's local source, for [`preflight`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSource {
    pub package: String,
    pub path: PathBuf,
    /// A directory uv builds (`Origin::Dir`), which [`preflight`] checks is
    /// that package's project; an archive or repository is not.
    pub project: bool,
}

/// `uv pip install` arguments that put the packages back as they came.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    args: Vec<String>,
    packages: usize,
    local_sources: Vec<LocalSource>,
    hashed_archive: Option<String>,
}

impl InstallPlan {
    /// Builds the arguments: `name==version` for a package with no recorded
    /// origin, `--editable <path>` for an editable directory (the
    /// subdirectory joined on, since uv ignores `#subdirectory=` there),
    /// `name @ file://<path>` for a local directory (the subdirectory joined
    /// on for the same reason), `name @ <url>` for an archive, and
    /// `name @ <vcs>+<url>@<commit>` for a VCS checkout (the commit before
    /// any `?query`). A URL carries at most one `#` fragment:
    /// `subdirectory=` and, for a hashed archive, the strongest recorded
    /// hash, joined with `&`.
    ///
    /// # Errors
    ///
    /// [`ScoopError::PackageOriginsUnreadable`], naming `env` (the env or
    /// export file the packages came from), when a local source's `file:`
    /// URL cannot be turned into a path (or, for `--editable`, into UTF-8),
    /// when a local archive has a subdirectory (uv ignores it there), or
    /// when an archive's recorded hashes are none of sha256, sha384 and
    /// sha512 (installing it would skip the check the hash asks for).
    pub fn build(packages: &[InstalledPackage], env: &str) -> Result<Self> {
        let bad_source = |package: &str, reason: String| ScoopError::PackageOriginsUnreadable {
            env: env.to_string(),
            reason: format!("{package}: {reason}"),
        };
        let mut args = Vec::new();
        let mut local_sources = Vec::new();
        let mut hashed_archive = None;
        for p in packages {
            let Some(origin) = &p.origin else {
                args.push(format!("{}=={}", p.name, p.version));
                continue;
            };
            if let Some(path) = origin.local_path() {
                let path = path.map_err(|reason| bad_source(&p.name, reason))?;
                if path.components().any(|c| c == Component::ParentDir) {
                    return Err(bad_source(
                        &p.name,
                        format!(
                            "{} has a `..`, which uv folds away without following symlinks, so it could install from a path other than the one checked",
                            path.display()
                        ),
                    ));
                }
                local_sources.push(LocalSource {
                    package: p.name.clone(),
                    path,
                    project: matches!(origin, Origin::Dir { .. }),
                });
            }
            match origin {
                Origin::Dir { editable, .. } => {
                    let path = &local_sources
                        .last()
                        .expect("a Dir origin has a local path")
                        .path;
                    if *editable {
                        let path = path.to_str().ok_or_else(|| {
                            bad_source(&p.name, format!("{} is not UTF-8", path.display()))
                        })?;
                        if path.contains(['#', '[', ']', '$']) {
                            return Err(bad_source(
                                &p.name,
                                format!(
                                    "uv cannot install {path} as editable: it reads # as a URL fragment, [..] as extras and ${{..}} as a variable"
                                ),
                            ));
                        }
                        args.push("--editable".to_string());
                        args.push(path.to_string());
                    } else {
                        // uv reads a URL ending in `.git` as a git checkout
                        // and installs its committed HEAD, not the working
                        // tree (measured); `--editable` keeps it a directory.
                        let git_named = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|n| n.to_ascii_lowercase().ends_with(".git"));
                        if git_named {
                            return Err(bad_source(
                                &p.name,
                                format!(
                                    "uv would install {} as a git checkout, not as the directory",
                                    path.display()
                                ),
                            ));
                        }
                        args.push(format!("{} @ {}", p.name, path_to_file_url(path)));
                    }
                }
                Origin::Archive {
                    url,
                    subdirectory,
                    hashes,
                } => {
                    let ArchiveUrl {
                        url,
                        subdirectory,
                        hashes,
                    } = split_archive_url(url, subdirectory, hashes)
                        .map_err(|reason| bad_source(&p.name, reason))?;
                    let (url, subdirectory, hashes) = (&url, &subdirectory, &hashes);
                    check_fragment_subdirectory(subdirectory)
                        .map_err(|reason| bad_source(&p.name, reason))?;
                    let hash = strongest_hash(hashes);
                    if hash.is_none() && !hashes.is_empty() {
                        let algs: Vec<&str> = hashes.keys().map(String::as_str).collect();
                        return Err(bad_source(
                            &p.name,
                            format!(
                                "its recorded hashes ({}) include no sha256, sha384 or sha512",
                                algs.join(", ")
                            ),
                        ));
                    }
                    if subdirectory.is_some() && is_file_url(url) {
                        return Err(bad_source(
                            &p.name,
                            "uv ignores #subdirectory= in a local archive".to_string(),
                        ));
                    }
                    if hash.is_some() && hashed_archive.is_none() {
                        hashed_archive = Some(p.name.clone());
                    }
                    let url = lowercase_file_scheme(url);
                    args.push(direct_reference(&p.name, &url, subdirectory, hash));
                }
                Origin::Vcs {
                    vcs,
                    url,
                    commit_id,
                    subdirectory,
                    ..
                } => {
                    if url.contains('#') {
                        return Err(bad_source(&p.name, format!("{url} has a fragment")));
                    }
                    if vcs != "git" {
                        return Err(bad_source(
                            &p.name,
                            format!("uv installs only git checkouts, not {vcs}"),
                        ));
                    }
                    check_fragment_subdirectory(subdirectory)
                        .map_err(|reason| bad_source(&p.name, reason))?;
                    let url = lowercase_file_scheme(url);
                    let (base, query) = url.split_at(url.find('?').unwrap_or(url.len()));
                    let pinned = format!("{vcs}+{base}@{commit_id}{query}");
                    args.push(direct_reference(&p.name, &pinned, subdirectory, None));
                }
            }
        }
        Ok(Self {
            args,
            packages: packages.len(),
            local_sources,
            hashed_archive,
        })
    }

    /// The arguments for one `uv pip install` call.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// How many packages, not arguments (`--editable` takes two).
    pub fn packages(&self) -> usize {
        self.packages
    }

    /// Every package installed from a local path.
    pub fn local_sources(&self) -> &[LocalSource] {
        &self.local_sources
    }

    /// The first package whose archive hash only a new enough uv checks.
    pub fn hashed_archive(&self) -> Option<&str> {
        self.hashed_archive.as_deref()
    }
}

/// An archive URL without its `#fragment`, and the subdirectory and hashes
/// with the fragment's merged in.
#[derive(Debug, PartialEq, Eq)]
struct ArchiveUrl {
    url: String,
    subdirectory: Option<String>,
    hashes: BTreeMap<String, String>,
}

/// Splits the fragment off an archive URL. uv records a hashed install's
/// `#sha256=` in the URL and leaves `archive_info` empty, so a fragment
/// hash counts as a recorded one; `subdirectory=` merges likewise. `egg=`
/// and keys with no value (`#download`) are dropped. Any other key is
/// taken for a hash algorithm, so an unknown one is refused later rather
/// than lost. Passing the fragment through would not do: uv reads
/// `#download#sha256=..` as no hash at all.
///
/// # Errors
///
/// A second `#`, or a fragment value that disagrees with the recorded one.
fn split_archive_url(
    url: &str,
    subdirectory: &Option<String>,
    hashes: &BTreeMap<String, String>,
) -> std::result::Result<ArchiveUrl, String> {
    let mut out = ArchiveUrl {
        url: url.to_string(),
        subdirectory: subdirectory.clone(),
        hashes: hashes.clone(),
    };
    let Some((base, fragment)) = url.split_once('#') else {
        return Ok(out);
    };
    if fragment.contains('#') {
        return Err(format!("{url} has more than one #"));
    }
    out.url = base.to_string();
    for (key, value) in fragment.split('&').filter_map(|pair| pair.split_once('=')) {
        // Digests compare as hex, in lower case like `archive_hashes`.
        let (slot, value) = match key {
            "egg" => continue,
            "subdirectory" => (
                out.subdirectory.get_or_insert_with(|| value.to_string()),
                value.to_string(),
            ),
            alg => {
                let hex = value.to_ascii_lowercase();
                let slot = out
                    .hashes
                    .entry(alg.to_ascii_lowercase())
                    .or_insert_with(|| hex.clone());
                (slot, hex)
            }
        };
        if *slot != value {
            return Err(format!("{url} and its recorded origin disagree on {key}"));
        }
    }
    Ok(out)
}

/// A subdirectory that survives a `#subdirectory=` fragment as written:
/// uv splits the fragment on `&`, ends it at `#`, and does not decode the
/// percent-encoding URL parsing adds (`café` is looked up as `caf%C3%A9`,
/// measured), so only ASCII letters, digits and `-._~/+@,=` pass.
fn check_fragment_subdirectory(subdirectory: &Option<String>) -> std::result::Result<(), String> {
    let fits = |b: u8| b.is_ascii_alphanumeric() || b"-._~/+@,=".contains(&b);
    match subdirectory {
        Some(s) if !s.bytes().all(fits) => Err(format!(
            "subdirectory {s} has a character a URL fragment does not carry as written"
        )),
        _ => Ok(()),
    }
}

/// `name @ url[#subdirectory=..&<alg>=..]`.
fn direct_reference(
    name: &str,
    url: &str,
    subdirectory: &Option<String>,
    hash: Option<(&str, &str)>,
) -> String {
    let mut fragment = Vec::new();
    if let Some(s) = subdirectory {
        fragment.push(format!("subdirectory={s}"));
    }
    if let Some((alg, hex)) = hash {
        fragment.push(format!("{alg}={hex}"));
    }
    if fragment.is_empty() {
        format!("{name} @ {url}")
    } else {
        format!("{name} @ {url}#{}", fragment.join("&"))
    }
}

/// sha512, then sha384, then sha256; other algorithms are not passed.
fn strongest_hash(hashes: &BTreeMap<String, String>) -> Option<(&str, &str)> {
    ["sha512", "sha384", "sha256"]
        .into_iter()
        .find_map(|alg| hashes.get(alg).map(|hex| (alg, hex.as_str())))
}

/// Checks a plan before anything is deleted or created. Every local source
/// must exist, and a directory source must be the package's project
/// ([`check_project`]); these run whatever `roots` holds. Then, for each
/// root that exists (an env `--force` would delete, a source `migrate
/// --delete-source` would remove), no local source may be reached through
/// an entry inside it.
///
/// # Errors
///
/// [`ScoopError::PackageSourceMissing`],
/// [`ScoopError::PackageSourceNotAProject`] or
/// [`ScoopError::PackageSourceInsideTarget`].
pub fn preflight(plan: &InstallPlan, roots: &[&Path]) -> Result<()> {
    let mut existing = Vec::new();
    for root in roots {
        if root.try_exists()? {
            existing.push(std::fs::canonicalize(root)?);
        }
    }
    for source in plan.local_sources() {
        let visited =
            resolve_visiting(&source.path).map_err(|_| ScoopError::PackageSourceMissing {
                package: source.package.clone(),
                path: source.path.clone(),
            })?;
        if source.project {
            check_project(&source.package, &source.path).map_err(|reason| {
                ScoopError::PackageSourceNotAProject {
                    package: source.package.clone(),
                    path: source.path.clone(),
                    reason,
                }
            })?;
        }
        for root in &existing {
            for entry in &visited {
                if is_within(entry, root)? {
                    return Err(ScoopError::PackageSourceInsideTarget {
                        package: source.package.clone(),
                        path: source.path.clone(),
                        target: root.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Whether uv would build `dir` as `package`: it needs a `pyproject.toml`
/// or a `setup.py` there (uv's own refusal otherwise, measured), and a
/// `pyproject.toml` that names a project statically (PEP 621 forbids a
/// dynamic name) must name this one. An old src-layout `.egg-link` points
/// at `src/`, which has neither file. A `setup.py`-only project cannot be
/// checked by name.
fn check_project(package: &str, dir: &Path) -> std::result::Result<(), String> {
    let pyproject = dir.join("pyproject.toml");
    let has_pyproject = pyproject.is_file();
    if !has_pyproject && !dir.join("setup.py").is_file() {
        return Err(t!("packages.no_project_file").to_string());
    }
    if !has_pyproject {
        return Ok(());
    }
    let text = std::fs::read_to_string(&pyproject).map_err(|e| e.to_string())?;
    let doc: toml::Table = toml::from_str(&text)
        .map_err(|e| t!("packages.pyproject_unreadable", error = e.message()).to_string())?;
    let name = doc
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str());
    match name {
        Some(name) if canonical_name(name) != canonical_name(package) => {
            Err(t!("packages.pyproject_names_another", name = name).to_string())
        }
        _ => Ok(()),
    }
}

/// Whether `entry` is `target` or lies under it: by device and inode on
/// Unix, by canonical name elsewhere (see the two helpers).
///
/// # Errors
///
/// Reading `target`'s metadata, or (elsewhere) canonicalizing either path.
pub(crate) fn is_within(entry: &Path, target: &Path) -> std::io::Result<bool> {
    #[cfg(unix)]
    return is_within_by_id(entry, target);
    #[cfg(not(unix))]
    return is_within_by_name(entry, target);
}

/// Whether `entry` is `target` or lies under it, compared by device and
/// inode: on a case-insensitive file system `/envs/DST/p` lies under
/// `/envs/dst` though no path prefix says so.
#[cfg(unix)]
fn is_within_by_id(entry: &Path, target: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let target = std::fs::metadata(target)?;
    let id = (target.dev(), target.ino());
    Ok(entry
        .ancestors()
        .any(|a| std::fs::symlink_metadata(a).is_ok_and(|m| (m.dev(), m.ino()) == id)))
}

/// Windows has no stable file id in std: both sides are canonicalized
/// (which also expands 8.3 short names) and compared ignoring case. A
/// symlink entry keeps its own name, under its canonical parent. Not
/// compiled on Unix, so mutants of it are excluded there.
#[cfg(not(unix))]
fn is_within_by_name(entry: &Path, target: &Path) -> std::io::Result<bool> {
    let target = std::fs::canonicalize(target)?;
    let is_link = std::fs::symlink_metadata(entry)?.file_type().is_symlink();
    let entry = match (is_link, entry.parent(), entry.file_name()) {
        (true, Some(parent), Some(name)) => std::fs::canonicalize(parent)?.join(name),
        _ => std::fs::canonicalize(entry)?,
    };
    let lower = |p: &Path| PathBuf::from(p.to_string_lossy().to_lowercase());
    Ok(lower(&entry).starts_with(lower(&target)))
}

/// Refuses a plan with a hashed archive when the uv `version` reports
/// (its `uv --version` line, asked only then) is older than
/// [`HASH_FRAGMENT_MIN_VERSION`](crate::uv::HASH_FRAGMENT_MIN_VERSION): an
/// older uv installs the archive without checking the hash. A version line
/// that does not parse is refused too.
///
/// # Errors
///
/// [`ScoopError::PackageHashUnenforced`], or the error from `version`.
pub fn check_hash_support(
    plan: &InstallPlan,
    version: impl FnOnce() -> Result<String>,
) -> Result<()> {
    let Some(package) = plan.hashed_archive() else {
        return Ok(());
    };
    let required = crate::uv::HASH_FRAGMENT_MIN_VERSION;
    let raw = version()?;
    if crate::uv::version::parse(&raw).is_some_and(|v| v >= required) {
        return Ok(());
    }
    Err(ScoopError::PackageHashUnenforced {
        package: package.to_string(),
        uv_version: raw,
        required: crate::uv::version::format_version(required),
    })
}

/// Everything checked before an install that may replace `target`: the
/// plan is built, its local sources pass [`preflight`], and the installed
/// uv can enforce any recorded archive hash. Call it before `--force`
/// deletes anything, a Python is installed, or an env is created.
///
/// # Errors
///
/// As [`InstallPlan::build`], [`preflight`] and [`check_hash_support`].
pub fn prepare_install(
    packages: &[InstalledPackage],
    from: &str,
    target: &Path,
) -> Result<InstallPlan> {
    prepare_install_with(&crate::uv::UvClient::new()?, packages, from, &[target])
}

/// [`prepare_install`] with a uv client the caller already has, and any
/// number of directories that may be removed (`roots`): an env `--force`
/// replaces, a source `migrate --delete-source` removes.
///
/// # Errors
///
/// As [`InstallPlan::build`], [`preflight`] and [`check_hash_support`].
pub fn prepare_install_with(
    uv: &crate::uv::UvClient,
    packages: &[InstalledPackage],
    from: &str,
    roots: &[&Path],
) -> Result<InstallPlan> {
    let plan = InstallPlan::build(packages, from)?;
    preflight(&plan, roots)?;
    check_hash_support(&plan, || uv.version())?;
    Ok(plan)
}

/// The symlinks a path resolution may expand before it is refused.
const MAX_SYMLINK_EXPANSIONS: usize = 40;

/// Resolves `path` the way the kernel does, one component at a time
/// against the filesystem (`..` steps back from the physical path reached
/// so far, a symlink is replaced by its target), and returns every entry it
/// visited, by physical location, ending with the resolved path. No
/// lexical `..` folding: `a/link/../b` goes through `link`.
///
/// # Errors
///
/// A relative path, a missing entry, or more than
/// [`MAX_SYMLINK_EXPANSIONS`] symlinks.
pub(crate) fn resolve_visiting(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    resolve(path, false)
}

/// [`resolve_visiting`] for a path that may not exist yet: it stops at the
/// first missing entry and returns what it visited up to there, so the
/// existing part of a prospective path is walked like any other.
///
/// # Errors
///
/// A relative path, or more than [`MAX_SYMLINK_EXPANSIONS`] symlinks.
pub(crate) fn resolve_visiting_existing(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    resolve(path, true)
}

fn resolve(path: &Path, stop_at_missing: bool) -> std::io::Result<Vec<PathBuf>> {
    use std::io::{Error, ErrorKind};
    if !path.is_absolute() {
        return Err(Error::new(ErrorKind::InvalidInput, "relative path"));
    }
    let mut pending: Vec<OsString> = Vec::new();
    let mut resolved = PathBuf::new();
    push_components(path, &mut pending, &mut resolved);
    let mut visited = Vec::new();
    let mut expanded: Vec<PathBuf> = Vec::new();
    while let Some(part) = pending.pop() {
        if part == ".." {
            resolved.pop();
            continue;
        }
        let next = resolved.join(&part);
        let meta = match std::fs::symlink_metadata(&next) {
            Ok(meta) => meta,
            Err(e) if stop_at_missing && e.kind() == ErrorKind::NotFound => break,
            Err(e) => return Err(e),
        };
        visited.push(next.clone());
        if meta.file_type().is_symlink() {
            expanded.push(next.clone());
            if expanded.len() > MAX_SYMLINK_EXPANSIONS {
                return Err(Error::other("too many symlinks"));
            }
            let target = std::fs::read_link(&next)?;
            push_components(&target, &mut pending, &mut resolved);
        } else {
            resolved = next;
        }
    }
    visited.push(resolved);
    Ok(visited)
}

/// Pushes `path`'s components onto `pending` (a stack, so in reverse); an
/// absolute `path` resets `resolved` to its root first.
fn push_components(path: &Path, pending: &mut Vec<OsString>, resolved: &mut PathBuf) {
    if path.has_root() {
        *resolved = path
            .components()
            .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
            .collect();
    }
    let parts: Vec<OsString> = path
        .components()
        .filter_map(|c| match c {
            Component::ParentDir => Some(OsString::from("..")),
            Component::Normal(n) => Some(n.to_os_string()),
            Component::Prefix(_) | Component::RootDir | Component::CurDir => None,
        })
        .collect();
    pending.extend(parts.into_iter().rev());
}

#[cfg(test)]
mod tests;
