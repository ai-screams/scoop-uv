//! Uninstall command

use std::io::IsTerminal;

use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::{Result, ScoopError};
use crate::output::{CascadeFailure, Output, UninstallData, UnverifiedEnv};
use crate::paths::{DirIdentity, dir_identity};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::python_version::PythonVersion;
use crate::uv::UvClient;

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
    // remove them only once uv has run, and only those whose interpreter is
    // really gone (see `recheck`).
    let plan = if cascade {
        Some(plan_cascade(output, &uv, version, force)?)
    } else {
        None
    };

    output.info(&t!("uninstall.uninstalling", version = version));

    if let Err(e) = uv.uninstall_python(version) {
        // uv may have removed some installs before failing on another: the
        // envs those served are broken now, so they still go. Only envs
        // that had their interpreter when planned count: one already broken
        // before uv ran was not broken by it.
        if let Some(plan) = plan {
            let served: Vec<Planned> = plan
                .planned
                .into_iter()
                .filter(|env| env.home_existed)
                .collect();
            let (removed, failed) = remove_envs(output, &served);
            if output.is_json() && !(removed.is_empty() && failed.is_empty()) {
                let data = UninstallData {
                    version: version.to_string(),
                    removed_envs: Some(removed),
                    failed_envs: failed,
                    unverified_envs: plan.unverified,
                };
                output.print_json(&incomplete_json(&e, &data));
            }
        }
        return Err(e);
    }

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
            output.print_json(&incomplete_json(&err, &data));
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

/// The JSON for a cascade that ended in `err` (envs left behind, or uv
/// failing after envs were removed): the error, plus the same `data` a
/// success carries, so scripts see which envs went and which did not.
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
    /// Whether `home` was there when planned (an env can be broken already).
    home_existed: bool,
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
            let (removed, remaining): (Vec<String>, Vec<String>) = installed_keys(&install_dir)?
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
                            // Unknown counts as there: the recheck then
                            // decides, and reports what it cannot check.
                            home_existed: home.try_exists().unwrap_or(true),
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
        // After an "unverified" warning, "no environments" would claim more
        // than is known.
        if !output.is_json() && unverified.is_empty() {
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

        // Anything but a yes (stderr not a terminal included) aborts.
        let confirmed =
            crate::cli::confirm::confirm(&t!("uninstall.cascade_confirm", version = version))
                == crate::cli::confirm::Confirmation::Yes;

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
    crate::core::pyvenv_home(&cfg)
        .map(PathBuf::from)
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
///
/// A directory uv has not created yet holds nothing. Any other read error
/// stops the cascade before anything is removed: an install missing from
/// the list would leave the envs it serves out of the plan.
fn installed_keys(install_dir: &Path) -> Result<Vec<String>> {
    let entries = match std::fs::read_dir(install_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut keys = Vec::new();
    for entry in entries {
        if let Ok(key) = entry?.file_name().into_string()
            && version_part(&key).is_some_and(|(numbers, _)| numbers.split('.').count() >= 3)
        {
            keys.push(key);
        }
    }
    Ok(keys)
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
mod tests;
