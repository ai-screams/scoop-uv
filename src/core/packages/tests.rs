//! Unit tests for `core::packages`.

use super::*;
use rstest::rstest;

fn origin(json: &str) -> std::result::Result<Origin, String> {
    serde_json::from_str::<Origin>(json).map_err(|e| e.to_string())
}

fn pkg(name: &str, version: &str, origin: Option<Origin>) -> InstalledPackage {
    InstalledPackage {
        name: name.to_string(),
        version: version.to_string(),
        origin,
        installer: None,
    }
}

fn listed(name: &str, version: &str, editable: Option<&str>) -> UvPipListEntry {
    UvPipListEntry {
        name: name.to_string(),
        version: version.to_string(),
        editable_project_location: editable.map(PathBuf::from),
    }
}

fn row(name: &str, version: &str, direct_url: Option<&str>) -> OriginRow {
    OriginRow(
        Some(name.to_string()),
        Some(version.to_string()),
        direct_url.map(str::to_string),
        None,
    )
}

fn row_by(
    name: &str,
    version: &str,
    direct_url: Option<&str>,
    installer: Option<&str>,
) -> OriginRow {
    OriginRow(
        Some(name.to_string()),
        Some(version.to_string()),
        direct_url.map(str::to_string),
        installer.map(str::to_string),
    )
}

const EDITABLE: &str = r#"{"url":"file:///work/proj","dir_info":{"editable":true}}"#;

// ============================================================================
// canonical_name
// ============================================================================

#[rstest]
#[case("Requests", "requests")]
#[case("foo_bar", "foo-bar")]
#[case("foo..bar", "foo-bar")]
#[case("Foo-_.Bar[socks]", "foo-bar")]
fn canonical_name_folds_case_separators_and_extras(#[case] raw: &str, #[case] want: &str) {
    assert_eq!(canonical_name(raw), want);
}

// ============================================================================
// Origin (PEP 610)
// ============================================================================

#[test]
fn origin_reads_each_kind() {
    assert_eq!(
        origin(EDITABLE).unwrap(),
        Origin::Dir {
            url: "file:///work/proj".into(),
            editable: true,
            subdirectory: None
        }
    );
    assert_eq!(
        origin(r#"{"url":"file:///work/proj","dir_info":{}}"#).unwrap(),
        Origin::Dir {
            url: "file:///work/proj".into(),
            editable: false,
            subdirectory: None
        }
    );
    assert_eq!(
        origin(r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"abc","requested_revision":"main"},"subdirectory":"pkg"}"#).unwrap(),
        Origin::Vcs {
            vcs: "git".into(),
            url: "https://h/r.git".into(),
            commit_id: "abc".into(),
            requested_revision: Some("main".into()),
            subdirectory: Some("pkg".into())
        }
    );
    assert_eq!(
        origin(r#"{"url":"https://h/x.whl","archive_info":{"hashes":{"SHA256":"AB"}}}"#).unwrap(),
        Origin::Archive {
            url: "https://h/x.whl".into(),
            subdirectory: None,
            hashes: BTreeMap::from([("sha256".into(), "ab".into())])
        }
    );
}

/// The legacy `hash` is folded into `hashes`; a disagreement is refused.
/// What fails this: dropping the legacy field, or the conflict check.
#[test]
fn origin_folds_the_legacy_hash_and_refuses_a_conflict() {
    let o = origin(r#"{"url":"https://h/x.whl","archive_info":{"hash":"sha256=aa"}}"#).unwrap();
    assert!(
        matches!(&o, Origin::Archive { hashes, .. } if hashes.get("sha256").map(String::as_str) == Some("aa"))
    );
    assert!(origin(r#"{"url":"https://h/x.whl","archive_info":{"hash":"sha256=aa","hashes":{"sha256":"aa"}}}"#).is_ok());
    let err = origin(
        r#"{"url":"https://h/x.whl","archive_info":{"hash":"sha256=aa","hashes":{"sha256":"bb"}}}"#,
    )
    .unwrap_err();
    assert!(err.contains("disagree"), "{err}");
    assert!(origin(r#"{"url":"https://h/x.whl","archive_info":{"hash":"nohash"}}"#).is_err());
}

#[rstest]
#[case::no_kind(r#"{"url":"file:///p"}"#)]
#[case::two_kinds(r#"{"url":"file:///p","dir_info":{},"archive_info":{}}"#)]
#[case::empty_url(r#"{"url":" ","dir_info":{}}"#)]
#[case::dir_not_file(r#"{"url":"https://h/p","dir_info":{}}"#)]
#[case::vcs_without_commit(r#"{"url":"https://h/r","vcs_info":{"vcs":"git","commit_id":""}}"#)]
#[case::vcs_without_name(r#"{"url":"https://h/r","vcs_info":{"vcs":" ","commit_id":"abc"}}"#)]
// uv's selectors scuv does not reproduce: refused, not dropped.
#[case::uv_path(
    r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"c"},"path":"dist/p.whl"}"#
)]
#[case::git_lfs(
    r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"c","git_lfs":true}}"#
)]
fn origin_refuses_an_unusable_record(#[case] json: &str) {
    assert!(origin(json).is_err(), "{json}");
}

#[test]
fn origin_treats_an_empty_or_dot_subdirectory_as_the_root() {
    for sub in ["", "."] {
        let json = format!(r#"{{"url":"file:///p","dir_info":{{}},"subdirectory":"{sub}"}}"#);
        assert!(matches!(
            origin(&json).unwrap(),
            Origin::Dir {
                subdirectory: None,
                ..
            }
        ));
    }
}

/// An origin writes back the PEP 610 object it was read from.
#[test]
fn origin_serialises_as_pep_610() {
    for json in [
        EDITABLE,
        r#"{"url":"file:///work/proj","dir_info":{}}"#,
        r#"{"url":"https://h/r.git","subdirectory":"pkg","vcs_info":{"vcs":"git","commit_id":"abc","requested_revision":"main"}}"#,
        r#"{"url":"https://h/x.whl","archive_info":{"hashes":{"sha256":"ab"}}}"#,
        r#"{"url":"https://h/x.whl","archive_info":{}}"#,
    ] {
        let back = serde_json::to_string(&origin(json).unwrap()).unwrap();
        assert_eq!(back, json);
    }
}

// ============================================================================
// file: URLs
// ============================================================================

#[cfg(unix)]
#[rstest]
#[case("file:///home/u/proj", "/home/u/proj")]
#[case("file://localhost/home/u/proj", "/home/u/proj")]
#[case("file://LOCALHOST/p", "/p")]
#[case("file:/home/u/proj", "/home/u/proj")]
#[case("file:///a/sp%20ace", "/a/sp ace")]
#[case("file:///a/hash%23mark", "/a/hash#mark")]
#[case("file:///a/p%252", "/a/p%2")]
#[case("file:///a/caf%C3%A9", "/a/café")]
fn file_url_decodes_to_a_path(#[case] url: &str, #[case] want: &str) {
    assert_eq!(file_url_to_path(url).unwrap(), PathBuf::from(want));
}

#[rstest]
#[case::not_file("https://h/p")]
#[case::remote_host("file://server/share/p")]
#[case::query("file:///p?x=1")]
#[case::fragment("file:///p#frag")]
#[case::bad_escape("file:///p%zz")]
#[case::short_escape("file:///p%2")]
#[case::relative("file:p")]
fn file_url_refuses(#[case] url: &str) {
    assert!(file_url_to_path(url).is_err(), "{url}");
}

// ============================================================================
// Identity
// ============================================================================

/// `/repo` + `pkg` is `/repo/pkg`, but `jump/..` is not folded away: it
/// may resolve somewhere else. What fails this: comparing the URL and
/// subdirectory as written, or folding `..` lexically.
#[test]
fn dir_identity_joins_the_subdirectory_and_keeps_parent_components() {
    let split =
        origin(r#"{"url":"file:///repo","dir_info":{"editable":true},"subdirectory":"pkg"}"#)
            .unwrap();
    let joined = origin(r#"{"url":"file:///repo/pkg","dir_info":{"editable":true}}"#).unwrap();
    assert_eq!(split.identity(), joined.identity());

    let dotted = origin(r#"{"url":"file:///outside/jump/../project","dir_info":{}}"#).unwrap();
    let plain = origin(r#"{"url":"file:///outside/project","dir_info":{}}"#).unwrap();
    assert_ne!(dotted.identity(), plain.identity());

    let not_editable = origin(r#"{"url":"file:///repo/pkg","dir_info":{}}"#).unwrap();
    assert_ne!(joined.identity(), not_editable.identity());
}

/// The VCS name counts; the requested revision does not (the commit does).
#[test]
fn vcs_identity_compares_vcs_and_commit_not_the_requested_revision() {
    let a = origin(r#"{"url":"https://h/r","vcs_info":{"vcs":"git","commit_id":"abc","requested_revision":"main"}}"#).unwrap();
    let b = origin(r#"{"url":"https://h/r","vcs_info":{"vcs":"git","commit_id":"abc"}}"#).unwrap();
    let hg = origin(r#"{"url":"https://h/r","vcs_info":{"vcs":"hg","commit_id":"abc"}}"#).unwrap();
    let other =
        origin(r#"{"url":"https://h/r","vcs_info":{"vcs":"git","commit_id":"def"}}"#).unwrap();
    assert_eq!(a.identity(), b.identity());
    assert_ne!(a.identity(), hg.identity());
    assert_ne!(a.identity(), other.identity());
}

/// uv's record of a hashed archive (`#sha256=` in the URL, empty
/// `archive_info`) and pip's (hash in `archive_info`) are one origin, so a
/// clone does not read as a change. What fails this: comparing the URL as
/// written.
#[test]
fn archive_identity_merges_a_url_fragment_hash() {
    let uv = origin(r#"{"url":"file:///w/p.whl#sha256=aa","archive_info":{}}"#).unwrap();
    let pip =
        origin(r#"{"url":"file:///w/p.whl","archive_info":{"hashes":{"sha256":"aa"}}}"#).unwrap();
    let other = origin(r#"{"url":"file:///w/p.whl#sha256=bb","archive_info":{}}"#).unwrap();
    assert_eq!(uv.identity(), pip.identity());
    let upper = origin(r#"{"url":"file:///w/p.whl#sha256=AA","archive_info":{}}"#).unwrap();
    assert_eq!(upper.identity(), pip.identity());
    assert_ne!(other.identity(), pip.identity());
}

#[test]
fn archive_identity_compares_hashes() {
    let a =
        origin(r#"{"url":"https://h/x.whl","archive_info":{"hashes":{"sha256":"aa"}}}"#).unwrap();
    let b =
        origin(r#"{"url":"https://h/x.whl","archive_info":{"hashes":{"sha256":"bb"}}}"#).unwrap();
    assert_ne!(a.identity(), b.identity());
}

#[test]
fn describe_names_the_kind() {
    assert_eq!(origin(EDITABLE).unwrap().describe(), "editable /work/proj");
    let vcs =
        origin(r#"{"url":"https://h/r","vcs_info":{"vcs":"git","commit_id":"abc"}}"#).unwrap();
    assert_eq!(vcs.describe(), "git https://h/r@abc");
}

// ============================================================================
// InstallPlan
// ============================================================================

/// Each kind becomes the argument uv reproduces it from.
#[cfg(unix)]
#[test]
fn install_plan_builds_one_argument_per_kind() {
    let plan = InstallPlan::build(
        &[
            pkg("six", "1.16.0", None),
            pkg("ed", "0.1", Some(origin(r#"{"url":"file:///repo","dir_info":{"editable":true},"subdirectory":"pkg"}"#).unwrap())),
            pkg("dir", "0.1", Some(origin(r#"{"url":"file:///work/dir","dir_info":{}}"#).unwrap())),
            pkg("whl", "0.1", Some(origin(r#"{"url":"https://h/x.whl","archive_info":{"hashes":{"sha256":"aa","sha512":"bb"}},"subdirectory":"s"}"#).unwrap())),
            pkg("plain", "0.1", Some(origin(r#"{"url":"https://h/y.whl","archive_info":{}}"#).unwrap())),
            pkg("vcs", "0.1", Some(origin(r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"abc"}}"#).unwrap())),
        ],
        "src",
    )
    .unwrap();
    assert_eq!(
        plan.args(),
        [
            "six==1.16.0",
            "--editable",
            "/repo/pkg",
            "dir @ file:///work/dir",
            "whl @ https://h/x.whl#subdirectory=s&sha512=bb",
            "plain @ https://h/y.whl",
            "vcs @ git+https://h/r.git@abc",
        ]
    );
    assert_eq!(plan.packages(), 6);
    assert_eq!(plan.hashed_archive(), Some("whl"));
    let sources: Vec<_> = plan
        .local_sources()
        .iter()
        .map(|s| (s.package.as_str(), s.path.clone()))
        .collect();
    assert_eq!(
        sources,
        [
            ("ed", PathBuf::from("/repo/pkg")),
            ("dir", PathBuf::from("/work/dir"))
        ]
    );
}

/// A local directory installs from a URL with its subdirectory joined on,
/// since uv ignores `#subdirectory=` there and would build the root
/// project; the URL reads back to the same path. What fails this: passing
/// the recorded URL and fragment through, or an encoding that does not
/// round-trip.
#[cfg(unix)]
#[test]
fn install_plan_joins_a_local_directory_subdirectory_into_its_url() {
    let plan = InstallPlan::build(
        &[pkg(
            "inner",
            "1",
            Some(
                origin(
                    r#"{"url":"file:///work/my%20repo","dir_info":{},"subdirectory":"py #?%&"}"#,
                )
                .unwrap(),
            ),
        )],
        "src",
    )
    .unwrap();
    assert_eq!(
        plan.args(),
        ["inner @ file:///work/my%20repo/py%20%23%3F%25%26"]
    );
    let url = plan.args()[0].strip_prefix("inner @ ").unwrap();
    assert_eq!(
        file_url_to_path(url).unwrap(),
        PathBuf::from("/work/my repo/py #?%&")
    );
}

/// Every byte outside the kept set is escaped and decodes back.
#[cfg(unix)]
#[test]
fn path_to_file_url_round_trips_every_byte() {
    use std::os::unix::ffi::OsStrExt;
    let bytes: Vec<u8> = (1..=255u8).filter(|b| *b != b'/').collect();
    let path = Path::new("/d").join(std::ffi::OsStr::from_bytes(&bytes));
    let url = path_to_file_url(&path);
    assert!(url.starts_with("file:///d/"), "{url}");
    assert_eq!(file_url_to_path(&url).unwrap(), path);
    assert_eq!(
        path_to_file_url(Path::new("/a-b_c.d~e:f/G9")),
        "file:///a-b_c.d~e:f/G9"
    );
}

/// A URL scheme matches in any case, so `FILE:` sources are checked like
/// `file:` ones, and the argument uv gets is lower case (uv reads
/// `FILE:///p` as a relative path). What fails this: a case-sensitive
/// scheme test, which skips the existence and containment checks.
#[cfg(unix)]
#[test]
fn install_plan_reads_an_upper_case_file_scheme() {
    let plan = InstallPlan::build(
        &[
            pkg(
                "a",
                "1",
                Some(origin(r#"{"url":"FILE:///w/a.whl","archive_info":{}}"#).unwrap()),
            ),
            pkg(
                "r",
                "1",
                Some(
                    origin(r#"{"url":"File:///w/repo","vcs_info":{"vcs":"git","commit_id":"c"}}"#)
                        .unwrap(),
                ),
            ),
            pkg(
                "d",
                "1",
                Some(origin(r#"{"url":"FILE:///w/dir","dir_info":{}}"#).unwrap()),
            ),
        ],
        "src",
    )
    .unwrap();
    assert_eq!(
        plan.args(),
        [
            "a @ file:///w/a.whl",
            "r @ git+file:///w/repo@c",
            "d @ file:///w/dir"
        ]
    );
    let paths: Vec<_> = plan
        .local_sources()
        .iter()
        .map(|s| s.path.clone())
        .collect();
    assert_eq!(
        paths,
        [
            PathBuf::from("/w/a.whl"),
            PathBuf::from("/w/repo"),
            PathBuf::from("/w/dir")
        ]
    );
}

/// The commit goes on the repository path, before a query. What fails
/// this: appending it to the whole URL, where it becomes part of the query.
#[test]
fn install_plan_pins_a_vcs_url_before_its_query() {
    let plan = InstallPlan::build(
        &[pkg("r", "1", Some(origin(r#"{"url":"https://h/r.git?t=1&u=2","vcs_info":{"vcs":"git","commit_id":"abc"}}"#).unwrap()))],
        "src",
    )
    .unwrap();
    assert_eq!(plan.args(), ["r @ git+https://h/r.git@abc?t=1&u=2"]);
}

/// Recorded hashes that are none of sha256, sha384 and sha512 are refused,
/// not dropped: dropping them installs the archive unchecked. One strong
/// hash among weak ones is used. What fails this: treating a weak-only
/// map like an empty one.
#[test]
fn install_plan_refuses_an_archive_with_only_weak_hashes() {
    let weak = pkg(
        "w",
        "1",
        Some(
            origin(
                r#"{"url":"https://h/w.whl","archive_info":{"hashes":{"md5":"aa","sha1":"bb"}}}"#,
            )
            .unwrap(),
        ),
    );
    let err = InstallPlan::build(&[weak], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains("md5, sha1")),
        "{err:?}"
    );
    let mixed = pkg(
        "m",
        "1",
        Some(
            origin(
                r#"{"url":"https://h/m.whl","archive_info":{"hashes":{"md5":"aa","sha256":"cc"}}}"#,
            )
            .unwrap(),
        ),
    );
    let plan = InstallPlan::build(&[mixed], "src").unwrap();
    assert_eq!(plan.args(), ["m @ https://h/m.whl#sha256=cc"]);
}

/// uv ignores `#subdirectory=` in a local archive and would build its root,
/// so that is refused; a remote archive keeps the fragment. What fails
/// this: dropping either condition.
#[test]
fn install_plan_refuses_a_local_archive_subdirectory() {
    let local = pkg(
        "l",
        "1",
        Some(
            origin(r#"{"url":"file:///w/repo.tar.gz","archive_info":{},"subdirectory":"py"}"#)
                .unwrap(),
        ),
    );
    let err = InstallPlan::build(&[local], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains("subdirectory")),
        "{err:?}"
    );
    let root = pkg(
        "l",
        "1",
        Some(origin(r#"{"url":"file:///w/repo.tar.gz","archive_info":{}}"#).unwrap()),
    );
    InstallPlan::build(&[root], "src").unwrap();
}

/// A fragment on an archive URL is split off: its hash counts as recorded
/// (uv records `#sha256=` there with an empty `archive_info`), and the URL
/// uv gets has one fragment. What fails this: passing the fragment through,
/// where uv reads `#download#sha256=..` as no hash at all.
#[cfg(unix)]
#[test]
fn install_plan_merges_an_archive_url_fragment() {
    let archive = |json: &str| pkg("w", "1", Some(origin(json).unwrap()));
    let uv_recorded = InstallPlan::build(
        &[archive(
            r#"{"url":"file:///d/w.whl#sha256=aa","archive_info":{}}"#,
        )],
        "src",
    )
    .unwrap();
    assert_eq!(uv_recorded.args(), ["w @ file:///d/w.whl#sha256=aa"]);
    assert_eq!(uv_recorded.hashed_archive(), Some("w"));
    assert_eq!(
        uv_recorded.local_sources()[0].path,
        PathBuf::from("/d/w.whl")
    );

    let noisy = InstallPlan::build(
        &[archive(r#"{"url":"https://h/w.whl#download&egg=w&subdirectory=s","archive_info":{"hashes":{"sha256":"aa"}}}"#)],
        "src",
    )
    .unwrap();
    assert_eq!(
        noisy.args(),
        ["w @ https://h/w.whl#subdirectory=s&sha256=aa"]
    );
    // A digest is hex: case does not make it disagree, and it is passed on
    // in lower case. What fails this: comparing the fragment's as written.
    let upper = InstallPlan::build(
        &[archive(
            r#"{"url":"https://h/w.whl#SHA256=AA","archive_info":{"hashes":{"sha256":"aa"}}}"#,
        )],
        "src",
    )
    .unwrap();
    assert_eq!(upper.args(), ["w @ https://h/w.whl#sha256=aa"]);
    let fragment_only = InstallPlan::build(
        &[archive(
            r#"{"url":"https://h/w.whl#sha256=AA","archive_info":{}}"#,
        )],
        "src",
    )
    .unwrap();
    assert_eq!(fragment_only.args(), ["w @ https://h/w.whl#sha256=aa"]);
    // `egg=` alone is not a hash: no hash, nothing to refuse.
    let egg = InstallPlan::build(
        &[archive(
            r#"{"url":"https://h/w.whl#egg=w","archive_info":{}}"#,
        )],
        "src",
    )
    .unwrap();
    assert_eq!(egg.args(), ["w @ https://h/w.whl"]);

    for (json, why) in [
        (
            r#"{"url":"https://h/w.whl#sha256=bb","archive_info":{"hashes":{"sha256":"aa"}}}"#,
            "disagree",
        ),
        (
            r#"{"url":"https://h/w.whl#subdirectory=x","archive_info":{},"subdirectory":"y"}"#,
            "disagree",
        ),
        (
            r#"{"url":"https://h/w.whl#a#sha256=aa","archive_info":{}}"#,
            "more than one",
        ),
        (
            r#"{"url":"https://h/w.whl#blake9=aa","archive_info":{}}"#,
            "blake9",
        ),
    ] {
        let err = InstallPlan::build(&[archive(json)], "src").unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains(why)),
            "{json}: {err:?}"
        );
    }
}

/// A local source with `..` is refused: preflight follows symlinks the way
/// the kernel does, uv folds `..` away first, so the two could name
/// different directories. What fails this: dropping the check.
#[cfg(unix)]
#[rstest]
#[case(r#"{"url":"file:///s/jump/../dst/pkg","dir_info":{}}"#)]
#[case(r#"{"url":"file:///s/jump/../w.whl","archive_info":{}}"#)]
#[case(r#"{"url":"file:///s/repo","dir_info":{"editable":true},"subdirectory":"../x"}"#)]
fn install_plan_refuses_a_local_source_with_dot_dot(#[case] json: &str) {
    let err = InstallPlan::build(&[pkg("p", "1", Some(origin(json).unwrap()))], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains("`..`")),
        "{json}: {err:?}"
    );
}

/// uv reads `#` in an editable path as a fragment and cannot install it
/// (measured with a raw path and with `%23`), so it is refused before
/// anything is deleted; a non-editable directory's `%23` URL installs.
/// What fails this: dropping the check.
#[cfg(unix)]
#[test]
fn install_plan_refuses_an_editable_path_with_a_hash_sign() {
    let ed = pkg(
        "p",
        "1",
        Some(origin(r#"{"url":"file:///w/pk%23dev","dir_info":{"editable":true}}"#).unwrap()),
    );
    let err = InstallPlan::build(&[ed], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains("editable")),
        "{err:?}"
    );
    let dir = pkg(
        "p",
        "1",
        Some(origin(r#"{"url":"file:///w/pk%23dev","dir_info":{}}"#).unwrap()),
    );
    assert_eq!(
        InstallPlan::build(&[dir], "src").unwrap().args(),
        ["p @ file:///w/pk%23dev"]
    );
}

/// A VCS URL with a fragment is refused: the commit would land after it.
#[test]
fn install_plan_refuses_a_vcs_url_with_a_fragment() {
    let vcs = pkg(
        "r",
        "1",
        Some(
            origin(r#"{"url":"https://h/r.git#egg=r","vcs_info":{"vcs":"git","commit_id":"c"}}"#)
                .unwrap(),
        ),
    );
    let err = InstallPlan::build(&[vcs], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains("fragment")),
        "{err:?}"
    );
}

/// uv reads `[..]` after an editable path as extras: `--editable
/// /w/pkg[dev]` installs `/w/pkg` (measured). Spaces, `;` and `@` install
/// as written (measured), so they pass. What fails this: checking `#` only.
#[cfg(unix)]
#[rstest]
#[case("file:///w/pkg%5Bdev%5D", false)]
#[case("file:///w/pkg%5D", false)]
// uv expands `${PART}` in a raw path (measured).
#[case("file:///w/%24%7BPART%7D/pkg", false)]
#[case("file:///w/my%20pkg;x@y", true)]
fn install_plan_refuses_editable_paths_uv_reads_as_extras(#[case] url: &str, #[case] ok: bool) {
    let json = format!(r#"{{"url":"{url}","dir_info":{{"editable":true}}}}"#);
    let result = InstallPlan::build(&[pkg("p", "1", Some(origin(&json).unwrap()))], "src");
    assert_eq!(result.is_ok(), ok, "{url}: {result:?}");
}

/// uv splits a fragment on `&` and ends it at `#`, so a subdirectory with
/// either would select another directory: refused for archives and VCS
/// alike. What fails this: dropping the check on either path.
#[rstest]
#[case(r#"{"url":"https://h/w.tar.gz","archive_info":{},"subdirectory":"pkg&other"}"#)]
// uv looks `café` up as `caf%C3%A9` (measured), and a space as `%20`.
#[case(
    r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"c"},"subdirectory":"café"}"#
)]
#[case(r#"{"url":"https://h/w.tar.gz","archive_info":{},"subdirectory":"my pkg"}"#)]
#[case(
    r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"c"},"subdirectory":"pkg#x"}"#
)]
fn install_plan_refuses_a_subdirectory_a_fragment_cannot_carry(#[case] json: &str) {
    let err = InstallPlan::build(&[pkg("p", "1", Some(origin(json).unwrap()))], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains("subdirectory")),
        "{json}: {err:?}"
    );
}

/// The characters a fragment carries as written pass; `git_lfs: false` is
/// no LFS. What fails this: an allowlist that drops one of them, or
/// refusing `git_lfs` whatever its value.
#[test]
fn install_plan_keeps_a_plain_subdirectory_and_no_lfs() {
    let json = r#"{"url":"https://h/r.git","vcs_info":{"vcs":"git","commit_id":"c","git_lfs":false},"subdirectory":"a-b.c_d~e/f+g@h,i=j/K9"}"#;
    let plan = InstallPlan::build(&[pkg("p", "1", Some(origin(json).unwrap()))], "src").unwrap();
    assert_eq!(
        plan.args(),
        ["p @ git+https://h/r.git@c#subdirectory=a-b.c_d~e/f+g@h,i=j/K9"]
    );
}

/// uv installs only git checkouts, so another VCS is refused before
/// anything is deleted. What fails this: dropping the check.
#[rstest]
#[case("hg")]
#[case("svn")]
#[case("bzr")]
fn install_plan_refuses_a_vcs_uv_cannot_install(#[case] vcs: &str) {
    let json = format!(r#"{{"url":"https://h/r","vcs_info":{{"vcs":"{vcs}","commit_id":"c"}}}}"#);
    let err =
        InstallPlan::build(&[pkg("p", "1", Some(origin(&json).unwrap()))], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref reason, .. } if reason.contains(vcs)),
        "{err:?}"
    );
}

/// uv reads a directory URL ending in `.git` as a git checkout and
/// installs its committed HEAD (measured), so a non-editable one is
/// refused; an editable one stays a directory (measured) and passes. What
/// fails this: dropping the check, or applying it to editable installs.
#[cfg(unix)]
#[rstest]
#[case(r#"{"url":"file:///w/demo.git","dir_info":{}}"#, false)]
#[case(r#"{"url":"file:///w/Demo.GIT","dir_info":{}}"#, false)]
#[case(
    r#"{"url":"file:///w/repo","dir_info":{},"subdirectory":"x.git"}"#,
    false
)]
#[case(r#"{"url":"file:///w/demo.git","dir_info":{"editable":true}}"#, true)]
#[case(r#"{"url":"file:///w/demo.gitx","dir_info":{}}"#, true)]
fn install_plan_refuses_a_directory_uv_reads_as_git(#[case] json: &str, #[case] ok: bool) {
    let result = InstallPlan::build(&[pkg("p", "1", Some(origin(json).unwrap()))], "src");
    assert_eq!(result.is_ok(), ok, "{json}: {result:?}");
}

#[test]
fn install_plan_without_hashed_archives_needs_no_hash_check() {
    let plan = InstallPlan::build(&[pkg("six", "1.16.0", None)], "src").unwrap();
    assert_eq!(plan.hashed_archive(), None);
    assert!(plan.local_sources().is_empty());
}

/// A local VCS checkout and a local archive are local sources too.
#[cfg(unix)]
#[test]
fn install_plan_lists_local_vcs_and_archive_sources() {
    let plan = InstallPlan::build(
        &[
            pkg(
                "g",
                "1",
                Some(
                    origin(
                        r#"{"url":"file:///work/repo","vcs_info":{"vcs":"git","commit_id":"c"}}"#,
                    )
                    .unwrap(),
                ),
            ),
            pkg(
                "w",
                "1",
                Some(origin(r#"{"url":"file:///work/w.whl","archive_info":{}}"#).unwrap()),
            ),
        ],
        "src",
    )
    .unwrap();
    assert_eq!(
        plan.args(),
        ["g @ git+file:///work/repo@c", "w @ file:///work/w.whl"]
    );
    assert_eq!(plan.local_sources().len(), 2);
}

#[test]
fn install_plan_refuses_an_unusable_local_url() {
    let bad = pkg(
        "ed",
        "1",
        Some(Origin::Dir {
            url: "file://server/share".into(),
            editable: true,
            subdirectory: None,
        }),
    );
    let err = InstallPlan::build(&[bad], "src").unwrap_err();
    assert!(
        matches!(err, ScoopError::PackageOriginsUnreadable { ref env, .. } if env == "src"),
        "{err:?}"
    );
}

/// The gate asks uv only for a hashed archive, and passes from the
/// first release that checks hashes. What fails this: comparing against
/// the wrong constant, or asking (or passing) without a hashed archive.
#[test]
fn check_hash_support_gates_on_the_first_enforcing_uv() {
    let hashed = InstallPlan::build(
        &[pkg(
            "w",
            "1",
            Some(
                origin(r#"{"url":"https://h/w.whl","archive_info":{"hashes":{"sha256":"aa"}}}"#)
                    .unwrap(),
            ),
        )],
        "src",
    )
    .unwrap();
    let plain = InstallPlan::build(&[pkg("six", "1.16.0", None)], "src").unwrap();
    check_hash_support(&plain, || panic!("asked uv without a hashed archive")).unwrap();
    check_hash_support(&hashed, || Ok("uv 0.11.29".into())).unwrap();
    check_hash_support(&hashed, || Ok("uv 0.12.13 (abc 2026-09-10)".into())).unwrap();
    for old in ["uv 0.11.28", "uv 0.5.19", "garbage"] {
        let err = check_hash_support(&hashed, || Ok(old.into())).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageHashUnenforced { ref package, .. } if package == "w"),
            "{old}: {err:?}"
        );
    }
}

/// The first *hashed* archive is the one reported, not the first archive.
/// What fails this: `||` in place of `&&` when recording it.
#[test]
fn hashed_archive_names_the_first_hashed_one() {
    let plan = InstallPlan::build(
        &[
            pkg(
                "plain",
                "1",
                Some(origin(r#"{"url":"https://h/p.whl","archive_info":{}}"#).unwrap()),
            ),
            pkg(
                "hashed",
                "1",
                Some(
                    origin(
                        r#"{"url":"https://h/h.whl","archive_info":{"hashes":{"sha256":"aa"}}}"#,
                    )
                    .unwrap(),
                ),
            ),
        ],
        "src",
    )
    .unwrap();
    assert_eq!(plan.hashed_archive(), Some("hashed"));

    let archive = |name: &str, json: &str| pkg(name, "1", Some(origin(json).unwrap()));
    let plain_only = InstallPlan::build(
        &[archive(
            "plain",
            r#"{"url":"https://h/p.whl","archive_info":{}}"#,
        )],
        "src",
    )
    .unwrap();
    assert_eq!(plain_only.hashed_archive(), None);
    let two = InstallPlan::build(
        &[
            archive(
                "first",
                r#"{"url":"https://h/1.whl","archive_info":{"hashes":{"sha256":"aa"}}}"#,
            ),
            archive(
                "second",
                r#"{"url":"https://h/2.whl","archive_info":{"hashes":{"sha256":"bb"}}}"#,
            ),
        ],
        "src",
    )
    .unwrap();
    assert_eq!(two.hashed_archive(), Some("first"));
}

#[rstest]
#[case("/C:/x", "C:/x")]
#[case("/c:", "c:")]
#[case("/1:/x", "/1:/x")]
#[case("/home/u", "/home/u")]
#[case("C:/x", "C:/x")]
fn strip_drive_slash_only_strips_before_a_drive_letter(#[case] path: &str, #[case] want: &str) {
    assert_eq!(strip_drive_slash(path), want);
}

/// A large listing (well under the cap) is read whole. What fails this:
/// a cap far smaller than 8 MiB.
#[test]
fn parse_origin_output_reads_a_large_listing() {
    // Over 1 MiB: a cap of `8 + 1024 * 1024` would refuse it.
    let rows: Vec<String> = (0..60_000)
        .map(|i| format!(r#"["p{i}","1.0.{i}",null]"#))
        .collect();
    let out = format!("SCUV_ORIGINS:[{}]\n", rows.join(","));
    assert!(out.len() > 1024 * 1024 + 8, "{}", out.len());
    assert_eq!(parse_origin_output(out.as_bytes()).unwrap().len(), 60_000);
}

#[test]
fn strongest_hash_prefers_sha512_then_sha384_then_sha256() {
    let h = |pairs: &[(&str, &str)]| {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(
        strongest_hash(&h(&[("sha256", "a"), ("sha384", "b")])),
        Some(("sha384", "b"))
    );
    assert_eq!(
        strongest_hash(&h(&[("sha256", "a")])),
        Some(("sha256", "a"))
    );
    assert_eq!(strongest_hash(&h(&[("md5", "a")])), None);
}

// ============================================================================
// reconcile
// ============================================================================

#[test]
fn reconcile_attaches_origins_by_name_and_version() {
    let got = reconcile(
        vec![
            listed("six", "1.16.0", None),
            listed("My_Proj", "0.1", Some("/work/proj")),
        ],
        vec![
            row("six", "1.16.0", None),
            row("my-proj", "0.1", Some(EDITABLE)),
            row("pip", "24", None),
        ],
    )
    .unwrap();
    assert_eq!(got[0], pkg("six", "1.16.0", None));
    assert_eq!(got[1].name, "My_Proj");
    assert!(matches!(
        got[1].origin,
        Some(Origin::Dir { editable: true, .. })
    ));
}

/// Every way of not knowing a package's origin is an error, never a
/// registry `name==version`. What fails each case: the fallback to `None`.
#[rstest]
#[case::not_seen(vec![listed("six", "1.16.0", None)], vec![row("six", "1.15.0", None)], "Python")]
#[case::ambiguous(vec![listed("six", "1.16.0", None)], vec![row("six", "1.16.0", None), row("six", "1.16.0", Some(EDITABLE))], "origins")]
#[case::editable_mismatch(vec![listed("p", "1", Some("/work/proj"))], vec![row("p", "1", Some(r#"{"url":"file:///work/proj","dir_info":{}}"#))], "editable")]
#[case::editable_relative(vec![listed("p", "1", Some("proj"))], vec![row("p", "1", None)], "editable")]
#[case::installer_ambiguous(vec![listed("p", "1", None)], vec![row_by("p", "1", None, Some("pip")), row_by("p", "1", None, Some("conda"))], "installers")]
#[case::installer_none_vs_conda(vec![listed("p", "1", None)], vec![row_by("p", "1", None, None), row_by("p", "1", None, Some("conda"))], "installers")]
#[case::bad_record(vec![listed("p", "1", None)], vec![row("p", "1", Some("{not json"))], "direct_url")]
fn reconcile_refuses_what_it_cannot_match(
    #[case] uv: Vec<UvPipListEntry>,
    #[case] rows: Vec<OriginRow>,
    #[case] says: &str,
) {
    let _locale = crate::test_utils::LocaleGuard::capture();
    rust_i18n::set_locale("en");
    let err = reconcile(uv, rows).unwrap_err();
    assert!(err.contains(says), "{err}");
}

/// An `.egg-link` editable (old pip, `setup.py develop`) records no
/// direct_url.json; its origin is the absolute project location uv
/// reports. What fails this: requiring a recorded origin.
#[cfg(unix)]
#[test]
fn reconcile_reads_an_egg_link_editable_from_uvs_location() {
    let got = reconcile(
        vec![listed("p", "1", Some("/work/my proj"))],
        vec![row("p", "1", None)],
    )
    .unwrap();
    assert_eq!(
        got[0].origin,
        Some(Origin::Dir {
            url: "file:///work/my%20proj".to_string(),
            editable: true,
            subdirectory: None,
        })
    );
}

/// The installer travels with the package; an empty one is none. What
/// fails this: dropping the fourth field.
#[test]
fn reconcile_keeps_the_installer() {
    let got = reconcile(
        vec![
            listed("a", "1", None),
            listed("b", "1", None),
            listed("c", "1", None),
        ],
        vec![
            row_by("a", "1", None, Some("conda")),
            row_by("b", "1", None, Some("")),
            row("c", "1", None),
        ],
    )
    .unwrap();
    let installers: Vec<_> = got.iter().map(|p| p.installer.as_deref()).collect();
    assert_eq!(installers, [Some("conda"), None, None]);
}

#[test]
fn reconcile_accepts_identical_duplicates_and_skips_unnamed_rows() {
    let got = reconcile(
        vec![listed("six", "1.16.0", None)],
        vec![
            row("six", "1.16.0", None),
            row("six", "1.16.0", None),
            OriginRow(None, Some("1".into()), None, None),
        ],
    )
    .unwrap();
    assert_eq!(got, [pkg("six", "1.16.0", None)]);
}

// ============================================================================
// parse_origin_output
// ============================================================================

#[test]
fn parse_origin_output_reads_the_marker_line_among_noise() {
    let out = b"hello from sitecustomize\nSCUV_ORIGINS:[[\"six\",\"1.16.0\",null]]\nmore\n";
    assert_eq!(
        parse_origin_output(out).unwrap(),
        [row("six", "1.16.0", None)]
    );
}

/// The script writes four fields (the installer last); three is the shape
/// before it did, and is read as no installer. Any other count is refused.
/// What fails this: a fixed-length tuple, or accepting any length.
#[test]
fn parse_origin_output_reads_rows_of_three_or_four() {
    let out = b"SCUV_ORIGINS:[[\"a\",\"1\",null,\"conda\"],[\"b\",\"1\",null]]\n";
    assert_eq!(
        parse_origin_output(out).unwrap(),
        [row_by("a", "1", None, Some("conda")), row("b", "1", None)]
    );
    for bad in [
        &b"SCUV_ORIGINS:[[\"a\",\"1\"]]\n"[..],
        b"SCUV_ORIGINS:[[\"a\",\"1\",null,null,null]]\n",
    ] {
        assert!(
            parse_origin_output(bad).is_err(),
            "{}",
            String::from_utf8_lossy(bad)
        );
    }
}

#[rstest]
#[case::too_old(b"SCUV_ORIGINS_PYTHON:3.7\n".to_vec(), "3.7")]
#[case::missing(b"nothing here\n".to_vec(), "no package listing")]
#[case::cut_off(b"SCUV_ORIGINS:[[\"six\",\"1\",null]]".to_vec(), "no package listing")]
#[case::bad_json(b"SCUV_ORIGINS:[[\n".to_vec(), "not valid JSON")]
#[case::too_large(vec![b'x'; STDOUT_CAP as usize], "MiB")]
fn parse_origin_output_refuses(#[case] out: Vec<u8>, #[case] says: &str) {
    let _locale = crate::test_utils::LocaleGuard::capture();
    rust_i18n::set_locale("en");
    let err = parse_origin_output(&out).unwrap_err();
    assert!(err.contains(says), "{err}");
}

// ============================================================================
// run_origin_script (fake interpreters: shell scripts)
// ============================================================================

#[cfg(unix)]
mod interpreter {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn script(dir: &Path, body: &str) -> PathBuf {
        let p = dir.join("python");
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    #[test]
    #[serial_test::serial]
    fn returns_the_rows() {
        let d = tempfile::tempdir().unwrap();
        let py = script(d.path(), r#"echo 'SCUV_ORIGINS:[["six","1.16.0",null]]'"#);
        assert_eq!(
            run_origin_script(&py, Duration::from_secs(5)).unwrap(),
            [row("six", "1.16.0", None)]
        );
    }

    #[test]
    #[serial_test::serial]
    fn reports_a_failing_interpreter_with_its_stderr() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");
        let d = tempfile::tempdir().unwrap();
        let py = script(d.path(), "echo boom >&2; exit 3");
        let err = run_origin_script(&py, Duration::from_secs(5)).unwrap_err();
        assert!(err.contains("boom"), "{err}");
    }

    #[test]
    fn reports_a_missing_interpreter() {
        let d = tempfile::tempdir().unwrap();
        assert!(run_origin_script(&d.path().join("nope"), Duration::from_secs(5)).is_err());
    }

    /// A process the hooks start that keeps stdout open (8 s) does not hold
    /// the read up. What fails this: reading through a pipe until EOF.
    #[test]
    #[serial_test::serial]
    fn a_descendant_holding_stdout_open_does_not_block() {
        let d = tempfile::tempdir().unwrap();
        let py = script(d.path(), r#"sleep 8 & echo 'SCUV_ORIGINS:[]'"#);
        let start = Instant::now();
        assert!(
            run_origin_script(&py, Duration::from_secs(10))
                .unwrap()
                .is_empty()
        );
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "{:?}",
            start.elapsed()
        );
    }

    /// A descendant that keeps writing (about 8 s) after the interpreter
    /// exits does not hold the call up. What fails this: reading stdout
    /// through a pipe until EOF. The writer stops by itself, so nothing
    /// outlives the test run; the cap is pinned by `read_capped_*`.
    #[test]
    #[serial_test::serial]
    fn a_descendant_still_writing_does_not_extend_the_read() {
        let d = tempfile::tempdir().unwrap();
        let py = script(
            d.path(),
            r#"echo 'SCUV_ORIGINS:[]'; (for i in $(seq 800); do echo spam; sleep 0.01; done) & sleep 0.2"#,
        );
        let start = Instant::now();
        assert!(
            run_origin_script(&py, Duration::from_secs(10))
                .unwrap()
                .is_empty()
        );
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "{:?}",
            start.elapsed()
        );
    }

    /// A few KiB of stderr reach the error whole. What fails this: a stderr
    /// cap far smaller than 64 KiB.
    #[test]
    #[serial_test::serial]
    fn keeps_a_few_kib_of_stderr_whole() {
        let d = tempfile::tempdir().unwrap();
        let py = script(
            d.path(),
            "i=0; while [ $i -lt 100 ]; do echo 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' >&2; i=$((i+1)); done; echo end >&2; exit 1",
        );
        let err = run_origin_script(&py, Duration::from_secs(5)).unwrap_err();
        assert!(err.contains("end"), "{err}");
        assert!(!err.contains("[...]"), "{err}");
    }

    /// Reads stop at the cap and say there was more. What fails this:
    /// dropping the `take`.
    #[test]
    fn read_capped_stops_at_the_cap() {
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(f.path(), b"0123456789").unwrap();
        assert_eq!(read_capped(f.path(), 4).unwrap(), (b"0123".to_vec(), true));
        assert_eq!(
            read_capped(f.path(), 10).unwrap(),
            (b"0123456789".to_vec(), false)
        );
    }

    /// Past the deadline the interpreter is killed and the call fails.
    /// What fails this: an unbounded wait.
    #[test]
    #[serial_test::serial]
    fn a_hung_interpreter_times_out() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");
        let d = tempfile::tempdir().unwrap();
        // Sleeps past the deadline, but not past the mutation-test
        // timeout: without the deadline the call returns "no listing".
        let py = script(d.path(), "sleep 3");
        let start = Instant::now();
        let err = run_origin_script(&py, Duration::from_millis(300)).unwrap_err();
        assert!(err.contains("did not finish"), "{err}");
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "{:?}",
            start.elapsed()
        );
    }
}

// ============================================================================
// resolve_visiting / preflight
// ============================================================================

#[cfg(unix)]
mod deletion_safety {
    use super::*;
    use std::os::unix::fs::symlink;

    /// A tree under a canonical temp root: `envs/dst` (the target) and
    /// `work/project`, `work/subdir/project`.
    struct Tree {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    impl Tree {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(dir.path()).unwrap();
            for d in ["envs/dst", "work/project", "work/subdir/project", "outside"] {
                std::fs::create_dir_all(root.join(d)).unwrap();
            }
            Self { _dir: dir, root }
        }
        fn p(&self, rel: &str) -> PathBuf {
            self.root.join(rel)
        }
        fn target(&self) -> PathBuf {
            self.p("envs/dst")
        }
    }

    /// Built by hand: `build` already refuses a `..`, and preflight must
    /// still resolve one the way the kernel does.
    fn plan(path: &Path) -> InstallPlan {
        InstallPlan {
            args: Vec::new(),
            packages: 1,
            local_sources: vec![LocalSource {
                package: "proj".to_string(),
                path: path.to_path_buf(),
                // These tests are about traversal; the project check has
                // its own below.
                project: false,
            }],
            hashed_archive: None,
        }
    }

    fn check(tree: &Tree, source: &Path) -> Result<()> {
        preflight(&plan(source), &[&tree.target()])
    }

    #[test]
    fn a_source_outside_the_target_passes() {
        let t = Tree::new();
        check(&t, &t.p("work/project")).unwrap();
    }

    #[test]
    fn a_source_inside_the_target_is_refused() {
        let t = Tree::new();
        std::fs::create_dir_all(t.p("envs/dst/src/project")).unwrap();
        let err = check(&t, &t.p("envs/dst/src/project")).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceInsideTarget { .. }),
            "{err:?}"
        );
    }

    /// On a case-insensitive file system `envs/DST/...` is inside
    /// `envs/dst` though no path prefix says so. What fails this: comparing
    /// paths instead of the file system's own identity. A case-sensitive
    /// file system has no such path, so there the source is missing.
    #[test]
    fn a_differently_cased_path_inside_the_target_is_refused() {
        let t = Tree::new();
        std::fs::create_dir_all(t.p("envs/dst/src/project")).unwrap();
        let err = check(&t, &t.p("envs/DST/src/project")).unwrap_err();
        if t.p("envs/DST").exists() {
            assert!(
                matches!(err, ScoopError::PackageSourceInsideTarget { .. }),
                "{err:?}"
            );
        } else {
            assert!(
                matches!(err, ScoopError::PackageSourceMissing { .. }),
                "{err:?}"
            );
        }
    }

    /// `outside/link -> envs/dst/real`: the path is outside, the source
    /// is not.
    #[test]
    fn an_inward_symlink_is_refused() {
        let t = Tree::new();
        std::fs::create_dir_all(t.p("envs/dst/real")).unwrap();
        symlink(t.p("envs/dst/real"), t.p("outside/link")).unwrap();
        let err = check(&t, &t.p("outside/link")).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceInsideTarget { .. }),
            "{err:?}"
        );
    }

    /// `envs/dst/link -> work/project`: the source survives, the way to it
    /// does not.
    #[test]
    fn an_outward_symlink_inside_the_target_is_refused() {
        let t = Tree::new();
        symlink(t.p("work/project"), t.p("envs/dst/link")).unwrap();
        let err = check(&t, &t.p("envs/dst/link")).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceInsideTarget { .. }),
            "{err:?}"
        );
    }

    /// `outside/jump -> envs/dst/hop -> work/subdir`; the recorded
    /// `outside/jump/../project` resolves to `work/project` through `hop`.
    /// What fails this: folding `..` before resolving.
    #[test]
    fn a_chained_symlink_before_parent_dir_is_refused() {
        let t = Tree::new();
        symlink(t.p("work/subdir"), t.p("envs/dst/hop")).unwrap();
        symlink(t.p("envs/dst/hop"), t.p("outside/jump")).unwrap();
        let source = t.p("outside/jump").join("..").join("project");
        let visited = resolve_visiting(&source).unwrap();
        assert_eq!(visited.last().unwrap(), &t.p("work/project"));
        let err = check(&t, &source).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceInsideTarget { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn a_relative_symlink_resolves_from_its_directory() {
        let t = Tree::new();
        symlink("../work/project", t.p("outside/rel")).unwrap();
        let visited = resolve_visiting(&t.p("outside/rel")).unwrap();
        assert_eq!(visited.last().unwrap(), &t.p("work/project"));
    }

    /// Exactly the limit of symlink expansions resolves; one more does not.
    /// What fails this: an off-by-one in the limit.
    #[test]
    fn the_symlink_limit_is_exact() {
        let t = Tree::new();
        let chain = |n: usize| {
            let base = t.p(&format!("outside/chain{n}"));
            std::fs::create_dir_all(&base).unwrap();
            for i in 1..=n {
                let target = if i == 1 {
                    t.p("work/project")
                } else {
                    base.join(format!("l{}", i - 1))
                };
                symlink(target, base.join(format!("l{i}"))).unwrap();
            }
            base.join(format!("l{n}"))
        };
        assert_eq!(
            resolve_visiting(&chain(MAX_SYMLINK_EXPANSIONS))
                .unwrap()
                .last()
                .unwrap(),
            &t.p("work/project")
        );
        assert!(resolve_visiting(&chain(MAX_SYMLINK_EXPANSIONS + 1)).is_err());
    }

    #[test]
    fn a_symlink_loop_is_an_error() {
        let t = Tree::new();
        symlink(t.p("outside/b"), t.p("outside/a")).unwrap();
        symlink(t.p("outside/a"), t.p("outside/b")).unwrap();
        assert!(resolve_visiting(&t.p("outside/a")).is_err());
    }

    #[test]
    fn a_missing_source_is_refused() {
        let t = Tree::new();
        let err = check(&t, &t.p("work/gone")).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceMissing { .. }),
            "{err:?}"
        );
    }

    /// With no target there is nothing to delete; only existence counts.
    #[test]
    fn an_absent_target_only_checks_existence() {
        let t = Tree::new();
        let absent = t.p("envs/new");
        preflight(&plan(&t.p("work/project")), &[&absent]).unwrap();
        let err = preflight(&plan(&t.p("work/gone")), &[&absent]).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceMissing { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn a_relative_path_is_refused() {
        assert!(resolve_visiting(Path::new("work/project")).is_err());
    }
}

// ============================================================================
// Project check
// ============================================================================

#[cfg(unix)]
mod project_check {
    use super::*;

    fn dir_plan(name: &str, dir: &Path) -> InstallPlan {
        let o = Origin::Dir {
            url: format!("file://{}", dir.display()),
            editable: true,
            subdirectory: None,
        };
        InstallPlan::build(&[pkg(name, "1", Some(o))], "src").unwrap()
    }

    fn reason(err: ScoopError) -> String {
        match err {
            ScoopError::PackageSourceNotAProject { reason, .. } => reason,
            other => panic!("{other:?}"),
        }
    }

    /// A directory uv would refuse, or build as another project, is
    /// refused before anything is deleted. What fails each case: dropping
    /// the file check, the name check, or the parse check.
    #[test]
    fn a_directory_that_is_not_the_package_is_refused() {
        let _locale = crate::test_utils::LocaleGuard::capture();
        rust_i18n::set_locale("en");
        let work = tempfile::TempDir::new().unwrap();
        let dir = std::fs::canonicalize(work.path()).unwrap();

        // An old src-layout .egg-link points at src/, which has neither file.
        let err = preflight(&dir_plan("alpha", &dir), &[]).unwrap_err();
        assert!(reason(err).contains("pyproject.toml or setup.py"));

        std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"beta\"\n").unwrap();
        let err = preflight(&dir_plan("alpha", &dir), &[]).unwrap_err();
        assert!(reason(err).contains("beta"));

        std::fs::write(dir.join("pyproject.toml"), "[project\n").unwrap();
        let err = preflight(&dir_plan("alpha", &dir), &[]).unwrap_err();
        assert!(reason(err).contains("does not parse"));
    }

    /// The package's own project passes: a name that differs only in case
    /// or separators, a pyproject without `[project]` (setuptools with
    /// setup.py), or setup.py alone. An archive is not a directory to
    /// check. What fails this: comparing names as written, or requiring
    /// `[project]`.
    #[test]
    fn the_packages_own_project_passes() {
        let work = tempfile::TempDir::new().unwrap();
        let dir = std::fs::canonicalize(work.path()).unwrap();
        std::fs::write(dir.join("setup.py"), "").unwrap();
        preflight(&dir_plan("alpha", &dir), &[]).unwrap();
        std::fs::write(
            dir.join("pyproject.toml"),
            "[build-system]\nrequires = []\n",
        )
        .unwrap();
        preflight(&dir_plan("alpha", &dir), &[]).unwrap();
        std::fs::write(
            dir.join("pyproject.toml"),
            "[project]\nname = \"My.Proj\"\n",
        )
        .unwrap();
        preflight(&dir_plan("my_proj", &dir), &[]).unwrap();

        let wheel = dir.join("w.whl");
        std::fs::write(&wheel, "").unwrap();
        let archive = Origin::Archive {
            url: format!("file://{}", wheel.display()),
            subdirectory: None,
            hashes: BTreeMap::new(),
        };
        let plan = InstallPlan::build(&[pkg("w", "1", Some(archive))], "src").unwrap();
        assert!(!plan.local_sources()[0].project);
        preflight(&plan, &[]).unwrap();
    }

    /// With no root to remove, every source is still checked. What fails
    /// this: returning early when `roots` is empty.
    #[test]
    fn sources_are_checked_with_no_roots() {
        let err = preflight(
            &dir_plan("alpha", Path::new("/nonexistent/scuv/alpha")),
            &[],
        )
        .unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceMissing { .. }),
            "{err:?}"
        );
    }

    /// Every root is checked, not only the first. What fails this: one
    /// root.
    #[test]
    fn every_root_is_checked() {
        let work = tempfile::TempDir::new().unwrap();
        let base = std::fs::canonicalize(work.path()).unwrap();
        let (a, b) = (base.join("a"), base.join("b"));
        let proj = b.join("proj");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("setup.py"), "").unwrap();
        let err = preflight(&dir_plan("proj", &proj), &[&a, &b]).unwrap_err();
        assert!(
            matches!(err, ScoopError::PackageSourceInsideTarget { ref target, .. } if *target == b),
            "{err:?}"
        );
    }
}
