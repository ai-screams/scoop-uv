//! uv version policy.
//!
//! Single source of truth for the minimum supported uv release. The doctor
//! check, Docker image pin, and user-facing docs should all be aligned with
//! [`MIN_VERSION`]; bumping the floor is a deliberate, one-line change here.

/// Minimum supported uv version (major, minor, patch).
///
/// `0.5.14` is the first release that stabilizes
/// `uv python list --output-format=json`, which we plan to migrate to.
/// Earlier releases would still work against today's text parser, but
/// pinning the floor at the same version we test against keeps every layer
/// (Docker image, doctor check, docs) in agreement.
pub const MIN_VERSION: (u32, u32, u32) = (0, 5, 14);

/// Parse `uv --version` stdout into a `(major, minor, patch)` tuple.
///
/// Accepted formats (uv prints any of these depending on install source):
///
/// * `"uv 0.11.16"`
/// * `"uv 0.11.16 (Homebrew 2026-05-21 aarch64-apple-darwin)"`
/// * `"uv 0.5.14 (build 1234)"`
///
/// Returns `None` if the second whitespace-separated token is not a
/// dotted triple of unsigned integers.
pub fn parse(raw: &str) -> Option<(u32, u32, u32)> {
    let version_token = raw.split_whitespace().nth(1)?;
    let mut parts = version_token.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// Returns true if `version` meets the [`MIN_VERSION`] floor.
pub fn meets_minimum(version: (u32, u32, u32)) -> bool {
    version >= MIN_VERSION
}

/// Format a `(major, minor, patch)` tuple as `MAJOR.MINOR.PATCH`.
pub fn format_version(version: (u32, u32, u32)) -> String {
    let (major, minor, patch) = version;
    format!("{major}.{minor}.{patch}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::plain("uv 0.5.14", Some((0, 5, 14)))]
    #[case::plain_higher("uv 0.11.16", Some((0, 11, 16)))]
    #[case::homebrew_suffix("uv 0.11.16 (Homebrew 2026-05-21 aarch64-apple-darwin)", Some((0, 11, 16)))]
    #[case::build_suffix("uv 0.5.14 (build 1234)", Some((0, 5, 14)))]
    #[case::extra_segments_ignored("uv 0.5.14.1", Some((0, 5, 14)))]
    #[case::empty("", None)]
    #[case::name_only("uv", None)]
    #[case::non_numeric("uv abc", None)]
    #[case::missing_patch("uv 0.5", None)]
    #[case::non_numeric_patch("uv 0.5.abc", None)]
    fn parse_cases(#[case] input: &str, #[case] expected: Option<(u32, u32, u32)>) {
        assert_eq!(parse(input), expected);
    }

    #[test]
    fn meets_minimum_boundary() {
        assert!(meets_minimum(MIN_VERSION));
    }

    #[test]
    fn meets_minimum_above() {
        assert!(meets_minimum((0, 11, 16)));
        assert!(meets_minimum((1, 0, 0)));
    }

    #[test]
    fn meets_minimum_below() {
        let (major, minor, patch) = MIN_VERSION;
        assert!(!meets_minimum((major, minor, patch.saturating_sub(1))));
        if minor > 0 {
            assert!(!meets_minimum((major, minor - 1, 99)));
        }
    }

    #[test]
    fn format_version_renders_dotted() {
        assert_eq!(format_version((0, 5, 14)), "0.5.14");
        assert_eq!(format_version((1, 12, 0)), "1.12.0");
    }
}
