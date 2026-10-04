//! Uninstall command

use std::io::IsTerminal;

use dialoguer::Confirm;
use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::{CascadeFailure, Output, UninstallData, UnverifiedEnv};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::uv::UvClient;
use crate::validate::PythonVersion;

/// Execute the uninstall command
pub fn execute(output: &Output, version: &str, cascade: bool, force: bool) -> Result<()> {
    // A cascade works out from the version numbers which installs uv will
    // remove; for a request that names more (`3.13t`, `3.12.0rc1`,
    // `cpython@3.12`) the numbers do not say, and a working env could be
    // deleted. Refuse before anything is touched.
    if cascade && !is_plain_version(version) {
        return Err(ScoopError::InvalidArgument {
            message: t!("uninstall.cascade_needs_plain_version", version = version).to_string(),
        });
    }

    let uv = UvClient::new()?;

    // Decide (and confirm) which envs go before anything is removed, but
    // remove them only once uv has uninstalled the Python: if that fails,
    // every env is still there.
    let plan = if cascade {
        Some(plan_cascade(output, &uv, version, force)?)
    } else {
        None
    };

    output.info(&t!("uninstall.uninstalling", version = version));

    uv.uninstall_python(version)?;

    let (removed_envs, failed_envs, unverified_envs) = match plan {
        Some(plan) => {
            let (removed, failed) = remove_envs(output, &plan.planned);
            (Some(removed), failed, plan.unverified)
        }
        None => (None, Vec::new(), Vec::new()),
    };

    let data = UninstallData {
        version: version.to_string(),
        removed_envs,
        failed_envs,
        unverified_envs,
    };

    // Envs left without a Python must not pass as a success: the Python is
    // gone either way, so report what was removed and what was not, and
    // exit non-zero.
    if !data.failed_envs.is_empty() {
        let err = ScoopError::CascadeIncomplete {
            failed_count: data.failed_envs.len(),
        };
        if output.is_json() {
            println!("{}", incomplete_json(&err, &data));
        } else {
            output.error(&err.to_string());
        }
        return Err(err);
    }

    if output.is_json() {
        output.json_success("uninstall", data);
        return Ok(());
    }

    output.success(&t!("uninstall.success", version = version));

    Ok(())
}

/// The JSON for a cascade that left envs behind: the error, plus the same
/// `data` a success carries, so scripts see which envs went and which did not.
fn incomplete_json(err: &ScoopError, data: &UninstallData) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "status": "error",
        "command": "uninstall",
        "error": { "code": err.code(), "message": err.to_string() },
        "data": data,
    }))
    .unwrap_or_default()
}

/// An env the cascade plans to remove: where it is, which directory that
/// was when planned, and the interpreter directory its `pyvenv.cfg` pointed
/// at. All three are checked again after the uninstall.
struct Planned {
    name: String,
    path: PathBuf,
    identity: DirIdentity,
    home: PathBuf,
}

/// What the cascade decided before uninstalling: the envs to remove, and
/// the envs whose interpreter could not be read and are left alone.
struct Plan {
    planned: Vec<Planned>,
    unverified: Vec<UnverifiedEnv>,
}

/// Work out which envs the uninstall takes the Python away from, list them
/// and ask for confirmation (unless `--force` or `--json`).
fn plan_cascade(output: &Output, uv: &UvClient, version: &str, force: bool) -> Result<Plan> {
    let service = VirtualenvService::auto()?;
    let envs = service.list()?;

    let mut unverified = Vec::new();
    let planned: Vec<Planned> = match PythonVersion::parse(version) {
        Some(filter) => {
            let install_dir = uv.python_dir()?;
            // Only a request naming the patch depends on the uv version.
            let skips = filter.patch.is_some()
                && patch_request_skips_prereleases(uv.version().ok().as_deref());
            let (removed, remaining): (Vec<String>, Vec<String>) = installed_keys(&install_dir)
                .into_iter()
                .partition(|key| removed_by(key, &filter, skips));
            // Resolved once; a directory that does not resolve holds no
            // install any env can be linked to.
            let install_root = std::fs::canonicalize(&install_dir).ok();
            let mut links: Vec<(String, Option<String>, Planned)> = Vec::new();
            for e in envs {
                match snapshot(&e.path) {
                    Ok((identity, home)) => {
                        let linked = install_root
                            .as_deref()
                            .and_then(|root| linked_install(&home, root));
                        let planned = Planned {
                            name: e.name.clone(),
                            path: e.path,
                            identity,
                            home,
                        };
                        links.push((e.name, linked, planned));
                    }
                    Err(unknown) => unverified.push(unknown.into_unverified(e.name)),
                }
            }
            let doomed = affected_envs(
                links.iter().map(|(n, l, _)| (n.clone(), l.clone())),
                &removed,
                &remaining,
            );
            links
                .into_iter()
                .filter(|(name, _, _)| doomed.contains(name))
                .map(|(_, _, planned)| planned)
                .collect()
        }
        None => Vec::new(),
    };

    // An env whose interpreter cannot be read may or may not lose it: it is
    // never removed, but the user hears about it before deciding.
    for (env, localized) in &unverified {
        output.warn(&t!(
            "uninstall.cascade_unverified",
            name = &env.name,
            reason = localized
        ));
    }
    let unverified: Vec<UnverifiedEnv> = unverified.into_iter().map(|(env, _)| env).collect();

    // No matching environments
    if planned.is_empty() {
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_none", version = version));
        }
        return Ok(Plan {
            planned,
            unverified,
        });
    }

    // Show matching environments and confirm (unless --force or --json)
    if !force && !output.is_json() {
        output.info(&t!(
            "uninstall.cascade_found",
            count = planned.len(),
            version = version
        ));
        for env in &planned {
            output.info(&t!("uninstall.cascade_env", name = &env.name));
        }

        // Check if stdin is a TTY; if not, abort rather than hanging
        if !std::io::stdin().is_terminal() {
            return Err(ScoopError::CascadeAborted);
        }

        let confirmed = Confirm::new()
            .with_prompt(t!("uninstall.cascade_confirm", version = version).to_string())
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            output.info(&t!("uninstall.cascade_cancelled"));
            return Err(ScoopError::CascadeAborted);
        }
    }

    Ok(Plan {
        planned,
        unverified,
    })
}

/// Remove the planned envs whose interpreter is really gone now.
///
/// The plan was made before uv ran, so each env is checked again (see
/// [`recheck`]): one uv kept working, or that was replaced in the meantime,
/// is kept. Every env is attempted; failures are reported, not fatal, so
/// one stuck env does not leave the rest broken and unreported.
fn remove_envs(output: &Output, planned: &[Planned]) -> (Vec<String>, Vec<CascadeFailure>) {
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    let service = match VirtualenvService::auto() {
        Ok(service) => service,
        Err(e) => {
            for env in planned {
                failed.push(CascadeFailure {
                    name: env.name.clone(),
                    error: e.to_string(),
                });
            }
            return (removed, failed);
        }
    };
    for env in planned {
        match recheck(env) {
            Recheck::Remove => {}
            Recheck::Gone => continue,
            Recheck::Replaced => {
                output.warn(&t!("uninstall.cascade_replaced", name = &env.name));
                continue;
            }
            Recheck::Works => continue,
            Recheck::Unknown(error) => {
                output.warn(&t!(
                    "uninstall.cascade_remove_failed",
                    name = &env.name,
                    error = &error
                ));
                failed.push(CascadeFailure {
                    name: env.name.clone(),
                    error,
                });
                continue;
            }
        }
        if !output.is_json() {
            output.info(&t!("uninstall.cascade_removing", name = &env.name));
        }
        match service.delete(&env.name) {
            Ok(()) => removed.push(env.name.clone()),
            Err(e) => {
                output.warn(&t!(
                    "uninstall.cascade_remove_failed",
                    name = &env.name,
                    error = e.to_string()
                ));
                failed.push(CascadeFailure {
                    name: env.name.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    if !output.is_json() && !removed.is_empty() {
        output.info(&t!("uninstall.cascade_removed", count = removed.len()));
    }

    (removed, failed)
}

/// What a planned env turned out to be once uv has run.
#[derive(Debug, PartialEq)]
enum Recheck {
    /// Same env, and its interpreter is gone: remove it.
    Remove,
    /// Its interpreter is still there (uv kept a link alive): keep it.
    Works,
    /// The env directory is already gone.
    Gone,
    /// Another env now has the name (other directory or other `home`): keep it.
    Replaced,
    /// Could not tell; not removed, and reported as a failure.
    Unknown(String),
}

/// Check a planned env again after the uninstall. It is removed only when
/// every question has a definite answer: still the same directory, still
/// the same `home`, and that `home` confirmed absent. An error on the way
/// (`Path::exists` would read it as "absent") keeps the env and reports it.
///
/// This catches an env replaced while uv ran. It does not serialize scuv
/// commands: an env recreated under the name between this check and the
/// removal by name is not caught (a lock across commands would be needed).
fn recheck(env: &Planned) -> Recheck {
    match dir_identity(&env.path) {
        Ok(now) if now == env.identity => {}
        Ok(_) => return Recheck::Replaced,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Recheck::Gone,
        Err(e) => return Recheck::Unknown(e.to_string()),
    }
    match env_home(&env.path) {
        Ok(home) if home == env.home => {}
        Ok(_) => return Recheck::Replaced,
        Err(unknown) => return Recheck::Unknown(unknown.reason(&rust_i18n::locale())),
    }
    match env.home.try_exists() {
        Ok(true) => Recheck::Works,
        Ok(false) => Recheck::Remove,
        Err(e) => Recheck::Unknown(format!("{}: {e}", env.home.display())),
    }
}

/// Identifies a directory beyond its path, so an env removed and created
/// again under the same name is told apart: device and inode on unix, the
/// creation time elsewhere (Windows). Neither is unique for all time: a
/// filesystem may reuse an inode once the old directory is gone, and a copy
/// can keep the original's creation time; with the same `home` such a
/// replacement passes. (The crate does not build for Windows yet, so that
/// branch is not exercised.) Reading it fails, with `NotFound`, when the
/// directory is gone.
type DirIdentity = (u64, u64);

#[cfg(unix)]
fn dir_identity(path: &Path) -> std::io::Result<DirIdentity> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::symlink_metadata(path)?;
    Ok((meta.dev(), meta.ino()))
}

#[cfg(not(unix))]
fn dir_identity(path: &Path) -> std::io::Result<DirIdentity> {
    let created = std::fs::symlink_metadata(path)?.created()?;
    let since = created
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?;
    Ok((since.as_secs(), u64::from(since.subsec_nanos())))
}

/// An env's identity and the `home` its `pyvenv.cfg` points at, read in
/// that order: if the env is replaced between the two reads, the identity
/// is the old directory's, so the recheck sees a different directory and
/// keeps the replacement.
fn snapshot(env_path: &Path) -> std::result::Result<(DirIdentity, PathBuf), HomeUnknown> {
    let identity = dir_identity(env_path).map_err(|e| HomeUnknown::DirUnreadable(e.to_string()))?;
    Ok((identity, env_home(env_path)?))
}

/// Why an env's interpreter could not be read from its `pyvenv.cfg`.
#[derive(Debug, PartialEq)]
enum HomeUnknown {
    NoConfig,
    Unreadable(String),
    NoHomeLine,
    DirUnreadable(String),
}

impl HomeUnknown {
    /// The reason in `locale`.
    fn reason(&self, locale: &str) -> String {
        match self {
            Self::NoConfig => t!("uninstall.cascade_reason_no_config", locale = locale),
            Self::NoHomeLine => t!("uninstall.cascade_reason_no_home", locale = locale),
            Self::Unreadable(e) => {
                t!(
                    "uninstall.cascade_reason_cfg_unreadable",
                    locale = locale,
                    error = e
                )
            }
            Self::DirUnreadable(e) => {
                t!(
                    "uninstall.cascade_reason_dir_unreadable",
                    locale = locale,
                    error = e
                )
            }
        }
        .to_string()
    }

    /// The env as JSON lists it (reason in English, stable for scripts),
    /// with the reason in the user's language for the warning.
    fn into_unverified(self, name: String) -> (UnverifiedEnv, String) {
        let reason = self.reason("en");
        let localized = self.reason(&rust_i18n::locale());
        (UnverifiedEnv { name, reason }, localized)
    }
}

/// The `home` line of an env's `pyvenv.cfg`: the directory its interpreter
/// lives in.
fn env_home(env_path: &Path) -> std::result::Result<PathBuf, HomeUnknown> {
    let cfg = match std::fs::read_to_string(env_path.join("pyvenv.cfg")) {
        Ok(cfg) => cfg,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(HomeUnknown::NoConfig),
        Err(e) => return Err(HomeUnknown::Unreadable(e.to_string())),
    };
    cfg.lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "home").then(|| PathBuf::from(value.trim()))
        })
        .ok_or(HomeUnknown::NoHomeLine)
}

/// The uv-managed install an env's interpreter `home` lies in: the
/// directory name under `install_root` (uv's install directory, already
/// canonical) — `cpython-3.12.14-…` for a patch install, `cpython-3.12-…`
/// for uv's minor-version link. `None` for a Python outside it (Homebrew,
/// a `--python-path` interpreter): uninstalling a uv Python cannot break it.
fn linked_install(home: &Path, install_root: &Path) -> Option<String> {
    // `<key>/bin` on POSIX; on Windows the interpreter sits in `<key>`
    // itself; older uv layouts add an `install` level (`<key>/install[/bin]`).
    let mut entry = home.to_path_buf();
    for level in ["bin", "install"] {
        if entry.file_name().is_some_and(|n| n == level) {
            entry = entry.parent()?.to_path_buf();
        }
    }
    // Compare real paths (`/tmp` vs `/private/tmp`); the entry itself may be
    // the minor-version symlink, so resolve only its parent.
    let parent = std::fs::canonicalize(entry.parent()?).ok()?;
    if parent != install_root {
        return None;
    }
    Some(entry.file_name()?.to_string_lossy().into_owned())
}

/// The installs uv keeps under `install_dir`, by directory name (`key`):
/// entries whose version part names a patch release. The minor-version
/// links (`cpython-3.12-…`, symlinks or junctions) and uv's own files
/// (`.lock`, `.temp`) have none and are left out. Read from the directory, not
/// `uv python list`, which shows only installs for the current platform
/// while `uv python uninstall` removes matching ones for any platform.
fn installed_keys(install_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(install_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|key| version_part(key).is_some_and(|(numbers, _)| numbers.split('.').count() >= 3))
        .collect()
}

/// The version part of an install key, split into its numbers and its
/// variant (`cpython-3.13.1+freethreaded-…` → `("3.13.1", Some("freethreaded"))`).
fn version_part(key: &str) -> Option<(&str, Option<&str>)> {
    let version = key.split('-').nth(1)?;
    Some(match version.split_once('+') {
        Some((numbers, variant)) => (numbers, Some(variant)),
        None => (version, None),
    })
}

/// Whether this uv leaves a patch's pre-releases when asked to uninstall the
/// patch (`3.14.0` keeps `3.14.0rc1`): uv 0.9.1 and later
/// (astral-sh/uv#16210); earlier releases take them too. An unknown version
/// counts as taking them, the safe side: an env planned for removal is
/// still kept if its interpreter survives the uninstall.
fn patch_request_skips_prereleases(uv_version: Option<&str>) -> bool {
    uv_version
        .and_then(crate::uv::version::parse)
        .is_some_and(|v| v >= (0, 9, 1))
}

/// Whether `uv python uninstall <plain version>` removes the install `key`:
/// its numbers match and it is the default build. A plain request means
/// the default variant to uv; free-threaded and debug builds need their own
/// request (`3.13t`). With `patch_skips_prereleases` (see
/// [`patch_request_skips_prereleases`]), a request that names the patch
/// (`3.14.0`) means that release, not its pre-releases (`3.14.0rc1`); `3.14`
/// takes both either way.
fn removed_by(key: &str, filter: &PythonVersion, patch_skips_prereleases: bool) -> bool {
    version_part(key).is_some_and(|(numbers, variant)| {
        let prerelease = !numbers
            .split('.')
            .all(|part| part.bytes().all(|b| b.is_ascii_digit()));
        variant.is_none()
            && !(prerelease && filter.patch.is_some() && patch_skips_prereleases)
            && PythonVersion::parse(numbers).is_some_and(|v| filter.matches(&v))
    })
}

/// The name of uv's minor-version link for an install key: the patch dropped
/// from the version part, any variant kept (`cpython-3.12.14-macos-…` →
/// `cpython-3.12-macos-…`, `cpython-3.13.1+freethreaded-…` →
/// `cpython-3.13+freethreaded-…`).
fn minor_link_name(key: &str) -> Option<String> {
    let (numbers, variant) = version_part(key)?;
    let minor: Vec<&str> = numbers.split('.').take(2).collect();
    if minor.len() < 2 {
        return None;
    }
    let mut minor = minor.join(".");
    if let Some(variant) = variant {
        minor = format!("{minor}+{variant}");
    }
    let mut parts: Vec<&str> = key.split('-').collect();
    parts[1] = &minor;
    Some(parts.join("-"))
}

/// The envs an uninstall takes the Python away from, given what each env's
/// interpreter is linked to (see [`linked_install`]) and which install keys
/// go and stay.
///
/// An env linked to a removed install directly is affected. An env linked
/// to uv's minor-version link is affected only when no remaining install of
/// the same implementation, variant and platform takes over that link (uv
/// points it at the newest such install left). Envs linked to nothing uv
/// manages, or to an install that stays, are not.
fn affected_envs(
    envs: impl IntoIterator<Item = (String, Option<String>)>,
    removed: &[String],
    remaining: &[String],
) -> Vec<String> {
    let removed_keys: HashSet<&str> = removed.iter().map(String::as_str).collect();
    let removed_links: HashSet<String> =
        removed.iter().filter_map(|k| minor_link_name(k)).collect();
    let kept_links: HashSet<String> = remaining
        .iter()
        .filter_map(|k| minor_link_name(k))
        .collect();
    envs.into_iter()
        .filter(|(_, linked)| {
            linked.as_deref().is_some_and(|name| {
                removed_keys.contains(name)
                    || (removed_links.contains(name) && !kept_links.contains(name))
            })
        })
        .map(|(name, _)| name)
        .collect()
}

/// Whether `version` is one to three dot-separated numbers (`3`, `3.12`,
/// `3.12.14`): a request whose meaning to uv is its numbers.
fn is_plain_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() <= 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ScoopError;
    use crate::uv::UvClient;

    // =========================================================================
    // UninstallData JSON Structure Tests
    // =========================================================================

    /// Failed removals appear as `failed_envs` with name and error; the key
    /// is absent when nothing failed. Fails if the field is dropped or
    /// always written.
    #[test]
    fn uninstall_data_json_reports_failed_envs() {
        let mut data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string()]),
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };
        assert!(
            serde_json::to_value(&data)
                .unwrap()
                .get("failed_envs")
                .is_none()
        );
        data.failed_envs.push(CascadeFailure {
            name: "1bad".to_string(),
            error: "Invalid".to_string(),
        });
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["failed_envs"][0]["name"], "1bad");
        assert_eq!(json["failed_envs"][0]["error"], "Invalid");
    }

    /// A cascade that left envs behind reports an error envelope that still
    /// carries the data. Fails if the status, code or data is dropped.
    #[test]
    fn incomplete_json_is_an_error_with_the_data() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string()]),
            failed_envs: vec![CascadeFailure {
                name: "1bad".to_string(),
                error: "Invalid".to_string(),
            }],
            unverified_envs: vec![UnverifiedEnv {
                name: "nocfg".to_string(),
                reason: "no pyvenv.cfg".to_string(),
            }],
        };
        let err = ScoopError::CascadeIncomplete { failed_count: 1 };
        let json: serde_json::Value = serde_json::from_str(&incomplete_json(&err, &data)).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["command"], "uninstall");
        assert_eq!(json["error"]["code"], "UNINSTALL_CASCADE_INCOMPLETE");
        assert_eq!(json["data"]["removed_envs"][0], "web");
        assert_eq!(json["data"]["failed_envs"][0]["name"], "1bad");
        assert_eq!(json["data"]["unverified_envs"][0]["name"], "nocfg");
    }

    /// `unverified_envs` is left out when empty, so a success without one
    /// keeps the earlier shape. Fails if it is always written.
    #[test]
    fn uninstall_data_json_omits_empty_unverified_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(Vec::new()),
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };
        let json = serde_json::to_value(&data).unwrap();
        assert!(json.get("unverified_envs").is_none(), "{json}");
    }

    /// The reason JSON carries is English whatever the user's language;
    /// the warning's is translated. Fails if either side switches.
    #[test]
    #[serial_test::serial]
    fn unverified_reason_is_english_for_json_and_localized_for_the_warning() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("ko");
        let (env, localized) = HomeUnknown::NoConfig.into_unverified("x".to_string());
        assert_eq!(env.reason, "no pyvenv.cfg");
        assert_eq!(localized, "pyvenv.cfg 없음");
        assert_eq!(HomeUnknown::NoConfig.reason("ko"), "pyvenv.cfg 없음");
        assert_eq!(
            HomeUnknown::DirUnreadable("e".to_string()).reason("en"),
            "cannot read the environment directory: e"
        );
    }

    /// The snapshot reads the env directory's identity before its `home`,
    /// and an env whose directory cannot be read is not planned. Fails if
    /// an unreadable directory yields a snapshot.
    #[cfg(unix)]
    #[test]
    fn snapshot_reports_an_unreadable_env_directory() {
        let root = tempfile::tempdir().unwrap();
        let blocker = root.path().join("blocker");
        std::fs::write(&blocker, "").unwrap();
        assert!(matches!(
            snapshot(&blocker.join("web")),
            Err(HomeUnknown::DirUnreadable(_))
        ));
        let env = planned_env(root.path(), "web", &root.path().join("py"));
        assert_eq!(
            snapshot(&env.path).unwrap(),
            (env.identity, env.home.clone())
        );
    }

    #[test]
    fn uninstall_data_json_has_correct_field_name() {
        let data = UninstallData {
            version: "3.12.0".to_string(),
            removed_envs: None,
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&data).unwrap();

        // Verify exact JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("version").is_some());
        assert_eq!(parsed["version"].as_str().unwrap(), "3.12.0");
        // removed_envs should be absent when None (skip_serializing_if)
        assert!(parsed.get("removed_envs").is_none());
    }

    /// True roundtrip test: serialize -> deserialize -> compare
    #[test]
    fn uninstall_data_json_roundtrip() {
        let original = UninstallData {
            version: "3.11.5".to_string(),
            removed_envs: None,
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn uninstall_data_json_with_removed_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["env1".to_string(), "env2".to_string()]),
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "3.12");
        let envs = parsed["removed_envs"].as_array().unwrap();
        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0], "env1");
        assert_eq!(envs[1], "env2");
    }

    #[test]
    fn uninstall_data_json_roundtrip_with_removed_envs() {
        let original = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec!["web".to_string(), "api".to_string()]),
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn uninstall_data_json_empty_removed_envs() {
        let data = UninstallData {
            version: "3.12".to_string(),
            removed_envs: Some(vec![]),
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let envs = parsed["removed_envs"].as_array().unwrap();
        assert!(envs.is_empty());
    }

    // =========================================================================
    // Security & Edge Case Tests
    // =========================================================================

    #[test]
    fn uninstall_data_roundtrip_empty_version() {
        let original = UninstallData {
            version: String::new(),
            removed_envs: None,
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert!(restored.version.is_empty());
    }

    #[test]
    fn uninstall_data_roundtrip_json_special_chars() {
        let test_cases = [
            ("quote", r#"3.12.0"injected"#),
            ("backslash", r"3.12.0\path"),
            ("newline", "3.12.0\nmalicious"),
            ("tab", "3.12.0\tvalue"),
            ("null_char", "3.12.0\0null"),
            ("control_chars", "3.12.0\r\n\x08"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
                removed_envs: None,
                failed_envs: Vec::new(),
                unverified_envs: Vec::new(),
            };

            let json = serde_json::to_string(&original).unwrap();
            let restored: UninstallData = serde_json::from_str(&json).unwrap();

            assert_eq!(
                original, restored,
                "Roundtrip failed for '{}': {:?}",
                name, version
            );
        }
    }

    #[test]
    fn uninstall_data_roundtrip_unicode() {
        let test_cases = [
            ("korean", "3.12.0-\u{d55c}\u{ae00}\u{bc84}\u{c804}"),
            ("emoji", "3.12.0-\u{1f40d}python"),
            ("chinese", "3.12.0-\u{4e2d}\u{6587}\u{7248}"),
            (
                "mixed",
                "v3.12.0-\u{03b1}\u{03b2}\u{03b3}-\u{65e5}\u{672c}\u{8a9e}",
            ),
            ("rtl", "3.12.0-\u{05e2}\u{05d1}\u{05e8}\u{05d9}\u{05ea}"),
        ];

        for (name, version) in test_cases {
            let original = UninstallData {
                version: version.to_string(),
                removed_envs: None,
                failed_envs: Vec::new(),
                unverified_envs: Vec::new(),
            };

            let json = serde_json::to_string(&original).unwrap();
            let restored: UninstallData = serde_json::from_str(&json).unwrap();

            assert_eq!(
                original, restored,
                "Unicode roundtrip failed for '{}'",
                name
            );
        }
    }

    #[test]
    fn uninstall_data_roundtrip_long_string() {
        let long_version = "3.12.0-".to_string() + &"x".repeat(10_000);
        let original = UninstallData {
            version: long_version,
            removed_envs: None,
            failed_envs: Vec::new(),
            unverified_envs: Vec::new(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: UninstallData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
        assert_eq!(restored.version.len(), 10_007);
    }

    // =========================================================================
    // execute Error Handling Tests
    // =========================================================================

    #[test]
    fn execute_returns_uv_not_found_when_uv_missing() {
        if UvClient::new().is_ok() {
            return;
        }

        let output = Output::new(0, true, crate::output::Colors::NONE, false);
        let result = execute(&output, "3.12.0", false, false);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ScoopError::UvNotFound),
            "Expected UvNotFound, got {:?}",
            err
        );
    }

    // =========================================================================
    // CascadeAborted Error Tests
    // =========================================================================

    /// uv's minor-version link name for an install key. Fails if the patch
    /// is not dropped, the variant or platform is lost, or a key without a
    /// patch yields a name.
    #[rstest::rstest]
    #[case(
        "cpython-3.12.14-macos-aarch64-none",
        Some("cpython-3.12-macos-aarch64-none")
    )]
    #[case(
        "cpython-3.13.1+freethreaded-linux-x86_64-gnu",
        Some("cpython-3.13+freethreaded-linux-x86_64-gnu")
    )]
    #[case("cpython-3-macos-aarch64-none", None)]
    #[case("garbage", None)]
    fn minor_link_name_cases(#[case] key: &str, #[case] link: Option<&str>) {
        assert_eq!(minor_link_name(key).as_deref(), link);
    }

    /// A plain request removes default builds whose numbers match; a
    /// variant (free-threaded) needs its own request, and a patch request
    /// leaves that patch's pre-releases. Fails if a variant or a pre-release
    /// of an exact patch is counted as removed, or the numbers stop deciding.
    #[rstest::rstest]
    #[case("3.12.14", "cpython-3.12.14-macos-aarch64-none", true)]
    #[case("3.12", "cpython-3.12.14-macos-aarch64-none", true)]
    #[case("3.12.14", "cpython-3.12.13-macos-aarch64-none", false)]
    #[case("3.12.14", "cpython-3.12.14+freethreaded-macos-aarch64-none", false)]
    #[case("3.12", "pypy-3.12.9-macos-aarch64-none", true)]
    #[case("3.14.0", "cpython-3.14.0rc1-macos-aarch64-none", false)]
    #[case("3.14", "cpython-3.14.0rc1-macos-aarch64-none", true)]
    #[case("3.14.0", "cpython-3.14.0-macos-aarch64-none", true)]
    fn removed_by_cases(#[case] request: &str, #[case] key: &str, #[case] removed: bool) {
        assert_eq!(
            removed_by(key, &PythonVersion::parse(request).unwrap(), true),
            removed
        );
    }

    /// Before uv 0.9.1 a patch request takes that patch's pre-releases too.
    /// Fails if the pre-release is left out of the plan regardless of uv.
    #[test]
    fn removed_by_counts_prereleases_for_an_older_uv() {
        let filter = PythonVersion::parse("3.14.0").unwrap();
        assert!(removed_by(
            "cpython-3.14.0rc1-macos-aarch64-none",
            &filter,
            false
        ));
    }

    /// Fails if the cut-over moves off 0.9.1, or an unknown version is
    /// taken to leave pre-releases.
    #[rstest::rstest]
    #[case(Some("uv 0.9.1"), true)]
    #[case(Some("uv 0.12.22 (Homebrew 2026-05-21 aarch64-apple-darwin)"), true)]
    #[case(Some("uv 0.9.0"), false)]
    #[case(Some("uv 0.5.19"), false)]
    #[case(Some("garbage"), false)]
    #[case(None, false)]
    fn patch_request_skips_prereleases_cases(#[case] raw: Option<&str>, #[case] skips: bool) {
        assert_eq!(patch_request_skips_prereleases(raw), skips);
    }

    /// Only real install directories with a patch version count: not the
    /// minor-version links, not uv's own files. Fails if a link or `.lock`
    /// is taken for an install.
    #[cfg(unix)]
    #[test]
    fn installed_keys_lists_patch_installs_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(P14)).unwrap();
        std::fs::create_dir_all(dir.path().join(FT13)).unwrap();
        std::fs::create_dir_all(dir.path().join(".temp")).unwrap();
        std::fs::write(dir.path().join(".lock"), "").unwrap();
        std::os::unix::fs::symlink(dir.path().join(P14), dir.path().join(LINK)).unwrap();
        // A real directory named like a link (a Windows junction is not a
        // symlink) is still not an install.
        std::fs::create_dir_all(dir.path().join("cpython-3.13-macos-aarch64-none")).unwrap();
        let mut keys = installed_keys(dir.path());
        keys.sort();
        assert_eq!(keys, [FT13, P14]);
    }

    const LINK: &str = "cpython-3.12-macos-aarch64-none";
    const P14: &str = "cpython-3.12.14-macos-aarch64-none";
    const P13: &str = "cpython-3.12.13-macos-aarch64-none";
    const FT13: &str = "cpython-3.12.13+freethreaded-macos-aarch64-none";
    const FT_LINK: &str = "cpython-3.12+freethreaded-macos-aarch64-none";

    /// Which envs lose their Python, by what their interpreter links to.
    /// Fails if a pinned removed install is missed, if a minor link is kept
    /// with no compatible install left (or taken with one left), if another
    /// build (free-threaded) is counted as taking over, or if an env outside
    /// uv's installs, or on a variant's own link, is touched.
    #[rstest::rstest]
    #[case::pinned_to_removed(Some(P14), &[P14], &[P13], true)]
    #[case::pinned_to_kept(Some(P13), &[P14], &[P13], false)]
    #[case::minor_link_nothing_left(Some(LINK), &[P14], &[], true)]
    #[case::minor_link_other_patch_left(Some(LINK), &[P14], &[P13], false)]
    #[case::minor_link_only_freethreaded_left(Some(LINK), &[P14], &[FT13], true)]
    #[case::freethreaded_link_untouched(Some(FT_LINK), &[P14], &[FT13], false)]
    #[case::outside_uv(None, &[P14, P13], &[], false)]
    fn affected_envs_cases(
        #[case] linked: Option<&str>,
        #[case] removed: &[&str],
        #[case] remaining: &[&str],
        #[case] affected: bool,
    ) {
        let removed: Vec<String> = removed.iter().map(|k| k.to_string()).collect();
        let remaining: Vec<String> = remaining.iter().map(|k| k.to_string()).collect();
        let got = affected_envs(
            [("web".to_string(), linked.map(str::to_string))],
            &removed,
            &remaining,
        );
        assert_eq!(got == ["web"], affected, "{got:?}");
    }

    /// The install an env's `home` lies in: through a symlinked minor link,
    /// a Windows `home` without `bin`, and an older `<key>/install/bin`
    /// layout. Fails if a `home` outside the install dir (Homebrew, a
    /// --python-path interpreter) yields a name, or one of the layouts is
    /// not recognised.
    #[cfg(unix)]
    #[test]
    fn linked_install_resolves_each_layout() {
        let root = tempfile::tempdir().unwrap();
        let installs = root.path().join("py");
        std::fs::create_dir_all(installs.join(P14).join("bin")).unwrap();
        std::fs::create_dir_all(installs.join(P13).join("install").join("bin")).unwrap();
        std::os::unix::fs::symlink(installs.join(P14), installs.join(LINK)).unwrap();
        let root = std::fs::canonicalize(&installs).unwrap();
        let at = |p: &Path| linked_install(p, &root);
        assert_eq!(at(&installs.join(LINK).join("bin")).as_deref(), Some(LINK));
        assert_eq!(at(&installs.join(P14).join("bin")).as_deref(), Some(P14));
        assert_eq!(at(&installs.join(P14)).as_deref(), Some(P14));
        assert_eq!(
            at(&installs.join(P13).join("install").join("bin")).as_deref(),
            Some(P13)
        );
        assert_eq!(at(Path::new("/opt/homebrew/opt/python@3.12/bin")), None);
    }

    /// `pyvenv.cfg`'s `home` line, or why it cannot be read. Fails if a
    /// missing file, an unreadable one (here a directory) and a file without
    /// the line stop being told apart, or the line stops being found.
    #[test]
    fn env_home_reads_the_home_line() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(env_home(dir.path()), Err(HomeUnknown::NoConfig));
        let cfg = dir.path().join("pyvenv.cfg");
        std::fs::create_dir(&cfg).unwrap();
        assert!(matches!(
            env_home(dir.path()),
            Err(HomeUnknown::Unreadable(_))
        ));
        std::fs::remove_dir(&cfg).unwrap();
        std::fs::write(&cfg, "version_info = 3.12\n").unwrap();
        assert_eq!(env_home(dir.path()), Err(HomeUnknown::NoHomeLine));
        std::fs::write(&cfg, "version_info = 3.12\nhome = /x/bin\n").unwrap();
        assert_eq!(env_home(dir.path()), Ok(PathBuf::from("/x/bin")));
    }

    /// A planned env `envs/<name>`, linked to a `home` under `py` that
    /// exists until the test removes it.
    #[cfg(unix)]
    fn planned_env(envs: &Path, name: &str, py: &Path) -> Planned {
        let path = envs.join(name);
        let home = py.join(P14).join("bin");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            path.join("pyvenv.cfg"),
            format!("home = {}\n", home.display()),
        )
        .unwrap();
        Planned {
            name: name.to_string(),
            identity: dir_identity(&path).unwrap(),
            path,
            home,
        }
    }

    /// Each answer of the recheck. Fails if a gone interpreter is not
    /// removed, a working one is, or a replaced or unreadable env is
    /// removed instead of kept.
    #[cfg(unix)]
    #[test]
    fn recheck_removes_only_the_same_env_whose_interpreter_is_gone() {
        let root = tempfile::tempdir().unwrap();
        let env = planned_env(root.path(), "web", &root.path().join("py"));
        assert_eq!(recheck(&env), Recheck::Works);
        std::fs::remove_dir_all(root.path().join("py")).unwrap();
        assert_eq!(recheck(&env), Recheck::Remove);

        // Another env under the name, pointing at the same home.
        let cfg = std::fs::read_to_string(env.path.join("pyvenv.cfg")).unwrap();
        std::fs::rename(&env.path, root.path().join("old")).unwrap();
        std::fs::create_dir_all(&env.path).unwrap();
        std::fs::write(env.path.join("pyvenv.cfg"), &cfg).unwrap();
        assert_eq!(recheck(&env), Recheck::Replaced);

        // Same directory, another home.
        let env = Planned {
            identity: dir_identity(&env.path).unwrap(),
            ..env
        };
        std::fs::write(env.path.join("pyvenv.cfg"), "home = /elsewhere/bin\n").unwrap();
        assert_eq!(recheck(&env), Recheck::Replaced);

        // pyvenv.cfg unreadable (a directory): cannot tell.
        std::fs::remove_file(env.path.join("pyvenv.cfg")).unwrap();
        std::fs::create_dir(env.path.join("pyvenv.cfg")).unwrap();
        assert!(matches!(recheck(&env), Recheck::Unknown(_)));

        std::fs::remove_dir_all(&env.path).unwrap();
        assert_eq!(recheck(&env), Recheck::Gone);
    }

    /// What `remove_envs` does with each recheck answer: an env replaced
    /// meanwhile and one it cannot judge both stay; only the latter is a
    /// failure. Fails if either is removed, or the undecided one is not
    /// reported.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn remove_envs_keeps_replaced_and_undecided_envs() {
        use crate::test_utils::env_guard;
        let home = tempfile::tempdir().unwrap();
        let _env = env_guard(&[(
            crate::paths::SCUV_HOME_ENV,
            Some(home.path().to_str().unwrap()),
        )]);
        let envs = home.path().join("virtualenvs");
        let py = home.path().join("py");
        let swap = planned_env(&envs, "swap", &py);
        let odd = planned_env(&envs, "odd", &py);
        std::fs::remove_dir_all(&py).unwrap();
        // "swap" is replaced: another directory under the name, same home.
        std::fs::rename(&swap.path, home.path().join("old")).unwrap();
        std::fs::create_dir_all(&swap.path).unwrap();
        std::fs::copy(
            home.path().join("old").join("pyvenv.cfg"),
            swap.path.join("pyvenv.cfg"),
        )
        .unwrap();
        // "odd" cannot be read: its pyvenv.cfg is a directory.
        std::fs::remove_file(odd.path.join("pyvenv.cfg")).unwrap();
        std::fs::create_dir(odd.path.join("pyvenv.cfg")).unwrap();

        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        let (removed, failed) = remove_envs(&out, &[swap, odd]);
        assert!(removed.is_empty(), "{removed:?}");
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].name, "odd");
        assert!(envs.join("swap").exists() && envs.join("odd").exists());
    }

    /// A pyvenv.cfg that turns unreadable during the recheck is reported in
    /// the user's language, like the other removal failures. Fails if the
    /// recheck falls back to the English reason.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn recheck_reports_an_unreadable_cfg_in_the_users_language() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("ko");
        let root = tempfile::tempdir().unwrap();
        let env = planned_env(root.path(), "web", &root.path().join("py"));
        std::fs::remove_file(env.path.join("pyvenv.cfg")).unwrap();
        assert_eq!(
            recheck(&env),
            Recheck::Unknown("pyvenv.cfg 없음".to_string())
        );
    }

    /// A `home` (or env directory) that cannot be checked is not taken as
    /// gone: `Path::exists` says false for it, `try_exists` reports the
    /// error. Fails if the recheck reads an error as absence and removes the
    /// env.
    #[cfg(unix)]
    #[test]
    fn recheck_keeps_an_env_whose_home_cannot_be_checked() {
        let root = tempfile::tempdir().unwrap();
        let mut env = planned_env(root.path(), "web", &root.path().join("py"));
        // A regular file where a directory of the path should be: stat
        // fails with ENOTDIR, not "not found".
        let blocker = root.path().join("blocker");
        std::fs::write(&blocker, "").unwrap();
        env.home = blocker.join("bin");
        std::fs::write(
            env.path.join("pyvenv.cfg"),
            format!("home = {}\n", env.home.display()),
        )
        .unwrap();
        assert!(!env.home.exists(), "exists() would read it as gone");
        assert!(matches!(recheck(&env), Recheck::Unknown(_)));
        // The same for the env directory itself: an error is not "gone".
        env.path = blocker.join("web");
        assert!(matches!(recheck(&env), Recheck::Unknown(_)));
    }

    /// Only a request that names the patch needs the uv version. Fails if a
    /// minor request asks uv for it, or a patch request stops asking.
    #[cfg(unix)]
    #[rstest::rstest]
    #[case("3.12", 0)]
    #[case("3.12.14", 1)]
    #[serial_test::serial]
    fn cascade_asks_the_uv_version_only_for_a_patch_request(
        #[case] request: &str,
        #[case] calls: usize,
    ) {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(&[]);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
        ]);
        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        execute(&out, request, true, true).unwrap();
        assert_eq!(uv.version_calls(), calls);
    }

    /// How an env's `home` is laid out under the fake uv's install dir.
    #[cfg(unix)]
    enum Home {
        /// uv's minor-version link, a symlink to this install.
        LinkTo(&'static str),
        /// This install's own directory, not the link.
        Install(&'static str),
        /// A minor-version link that is a real directory: it survives the
        /// fake uninstall, as when uv relinks it to another patch.
        LinkThatSurvives,
        /// Outside uv's installs (Homebrew).
        Outside,
    }

    /// A cascade run against a fake uv with `installs` as install
    /// directories and envs `(name, home)`. The fake uninstall removes the
    /// requested default build's directory. Returns which envs still exist,
    /// what uv was asked to uninstall, and the command's result.
    #[cfg(unix)]
    fn cascade_run(
        request: &str,
        installs: &[&str],
        envs: &[(&str, Home)],
        uninstall_fails: bool,
    ) -> (Vec<String>, Vec<String>, Result<()>) {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(&[]);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
            ("FAKE_UV_UNINSTALL_FAILS", uninstall_fails.then_some("1")),
        ]);
        let py = uv.python_dir();
        for key in installs {
            std::fs::create_dir_all(py.join(key).join("bin")).unwrap();
        }
        for (name, layout) in envs {
            let target = match layout {
                Home::LinkTo(key) => {
                    let link = py.join(LINK);
                    if !link.exists() {
                        std::os::unix::fs::symlink(py.join(key), &link).unwrap();
                    }
                    link.join("bin")
                }
                Home::Install(key) => py.join(key).join("bin"),
                Home::LinkThatSurvives => {
                    std::fs::create_dir_all(py.join(LINK).join("bin")).unwrap();
                    py.join(LINK).join("bin")
                }
                Home::Outside => PathBuf::from("/opt/homebrew/opt/python@3.12/bin"),
            };
            let env = home.path().join("virtualenvs").join(name);
            std::fs::create_dir_all(env.join("bin")).unwrap();
            std::fs::write(
                env.join("pyvenv.cfg"),
                format!("home = {}\nversion_info = 3.12\n", target.display()),
            )
            .unwrap();
        }

        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        let result = execute(&out, request, true, true);
        let left = envs
            .iter()
            .map(|(n, _)| n.to_string())
            .filter(|n| home.path().join("virtualenvs").join(n).exists())
            .collect();
        (left, uv.uninstalled(), result)
    }

    /// A minor request takes every patch of that minor: the env on the minor
    /// link and the env on a patch install both lose their Python, although
    /// two patches were installed. Fails if a remaining patch is counted as
    /// keeping the link for a minor request, or the minor request path stops
    /// removing envs (or the fake stops removing every patch it names).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_on_a_minor_request_removes_the_env_every_patch_served() {
        let (left, uninstalled, result) = cascade_run(
            "3.12",
            &[P14, P13],
            &[("web", Home::LinkTo(P14)), ("pinned", Home::Install(P13))],
            false,
        );
        result.unwrap();
        assert!(left.is_empty(), "no 3.12 is left for either env: {left:?}");
        assert_eq!(uninstalled, ["3.12"]);
    }

    /// An env on uv's 3.12 minor link, with 3.12.14 the only 3.12 install:
    /// uninstalling 3.12.14 breaks it, so --cascade removes it. Fails if the
    /// cascade matches by recorded version (`3.12` vs `3.12.14`: the bug —
    /// the Python went and the broken env stayed).
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_a_minor_link_env_when_no_install_takes_over() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &[P14], &[("web", Home::LinkTo(P14))], false);
        result.unwrap();
        assert!(left.is_empty(), "the env lost its only Python and must go");
        assert_eq!(uninstalled, ["3.12.14"]);
    }

    /// The same env with 3.12.13 also installed: it is not even planned for
    /// removal. Fails if remaining installs are not considered.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_a_minor_link_env_another_patch_takes_over() {
        let (left, _, result) =
            cascade_run("3.12.14", &[P13, P14], &[("web", Home::LinkTo(P14))], false);
        result.unwrap();
        assert_eq!(left, ["web"]);
    }

    /// Planned for removal (3.12.14 was the only install when the plan was
    /// made), but its interpreter is still there after the uninstall, as when
    /// a patch installed meanwhile took the link over: the env is kept. Fails
    /// if the plan is not checked again after uv runs.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_a_planned_env_whose_interpreter_survived() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &[P14], &[("web", Home::LinkThatSurvives)], false);
        result.unwrap();
        assert_eq!(uninstalled, ["3.12.14"]);
        assert_eq!(left, ["web"]);
    }

    /// An env on a Python uv does not manage (Homebrew) is never removed by
    /// uninstalling a uv Python. Fails if the cascade goes by recorded
    /// version instead of the env's link.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_keeps_an_env_on_a_python_uv_does_not_manage() {
        let (left, _, result) = cascade_run("3.12", &[P14], &[("brew", Home::Outside)], false);
        result.unwrap();
        assert_eq!(left, ["brew"]);
    }

    /// When uv fails to uninstall, no env has been removed. Fails if envs
    /// are deleted before the Python uninstall succeeds.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_removes_nothing_when_the_uninstall_fails() {
        let (left, uninstalled, result) =
            cascade_run("3.12.14", &[P14], &[("web", Home::LinkTo(P14))], true);
        assert!(result.is_err());
        assert_eq!(left, ["web"]);
        assert!(uninstalled.is_empty());
    }

    /// One env that cannot be removed (a name `delete` refuses) does not stop
    /// the others, and the command still completes. Fails if the first
    /// failure aborts the rest.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn cascade_goes_on_past_an_env_it_cannot_remove() {
        let (left, _, result) = cascade_run(
            "3.12.14",
            &[P14],
            &[("1bad", Home::LinkTo(P14)), ("web", Home::LinkTo(P14))],
            false,
        );
        assert!(
            matches!(
                result,
                Err(ScoopError::CascadeIncomplete { failed_count: 1 })
            ),
            "a failed removal must not pass as a success: {result:?}"
        );
        assert_eq!(left, ["1bad"], "web goes; 1bad fails and is reported");
    }

    /// One to three numbers are a plain request. Fails if a suffix (`rc1`,
    /// `t`), an implementation prefix, an empty part or a fourth part passes.
    #[rstest::rstest]
    #[case("3.12.14", true)]
    #[case("3.12", true)]
    #[case("3.12.0rc1", false)]
    #[case("3.13t", false)]
    #[case("cpython@3.12", false)]
    #[case("3..12", false)]
    #[case("3.12.14.1", false)]
    #[case("3", true)]
    fn is_plain_version_cases(#[case] version: &str, #[case] plain: bool) {
        assert_eq!(is_plain_version(version), plain);
    }

    /// A request that names more than numbers is refused before anything
    /// is removed: `3.13t` would otherwise match a GIL `3.13` env, and
    /// `3.12.0rc1` a stable `3.12.0` one. Fails if --cascade proceeds for a
    /// non-plain request, or refuses only after looking for uv or removing
    /// an env.
    #[cfg(unix)]
    #[rstest::rstest]
    #[case("3.13t")]
    #[case("3.12.0rc1")]
    #[case("cpython@3.12")]
    #[serial_test::serial]
    fn cascade_refuses_a_request_that_is_not_a_plain_version(#[case] request: &str) {
        use crate::test_utils::env_guard;
        // No uv on PATH and an env the numbers would match: the refusal must
        // come before the uv lookup and before any env is removed.
        let home = tempfile::tempdir().unwrap();
        let no_uv = tempfile::tempdir().unwrap();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(no_uv.path().to_str().unwrap())),
        ]);
        let env = home.path().join("virtualenvs").join("gil");
        std::fs::create_dir_all(env.join("bin")).unwrap();
        std::fs::write(
            env.join(crate::core::Metadata::FILE_NAME),
            r#"{"name":"gil","python_version":"3.13","created_at":"2026-01-01T00:00:00Z","created_by":"test","uv_version":null,"python_path":null}"#,
        )
        .unwrap();
        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        let err = execute(&out, request, true, true).unwrap_err();
        assert!(matches!(err, ScoopError::InvalidArgument { .. }), "{err:?}");
        assert!(env.exists(), "nothing may be removed");
    }

    /// The refusal is about --cascade only: a plain uninstall of the same
    /// request still goes to uv. Fails if the check runs without --cascade.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn uninstall_without_cascade_passes_any_request_to_uv() {
        use crate::test_utils::{FakeUv, env_guard};
        let home = tempfile::tempdir().unwrap();
        let uv = FakeUv::new(&["3.12.0"]);
        let path = uv.path_var();
        let _env = env_guard(&[
            (
                crate::paths::SCUV_HOME_ENV,
                Some(home.path().to_str().unwrap()),
            ),
            ("PATH", Some(path.as_str())),
        ]);
        let out = Output::new(0, true, crate::output::Colors::NONE, false);
        execute(&out, "cpython@3.12", false, false).unwrap();
        assert_eq!(uv.uninstalled(), ["cpython@3.12"]);
    }

    #[test]
    fn cascade_aborted_error_code() {
        let err = ScoopError::CascadeAborted;
        assert_eq!(err.code(), "UNINSTALL_CASCADE_ABORTED");
    }

    #[test]
    fn cascade_aborted_has_no_suggestion() {
        let err = ScoopError::CascadeAborted;
        assert!(err.suggestion().is_none());
    }
}
