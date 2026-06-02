//! Human-readable relative time formatting.
//!
//! Used by `status` / `info` / `list` to render `last_used` as
//! "3 hours ago" instead of a raw RFC 3339 timestamp. English-only on
//! purpose: the per-bucket plural/singular rules don't generalise across
//! the four locales we support (en/ko/ja/pt-BR) without per-locale
//! grammar work, and the existing decision (see plan v2) is that
//! short-form age labels are not worth four-way translation overhead.
//!
//! JSON output sticks with RFC 3339 — machine consumers parse the real
//! timestamp themselves. This module is for human eyes only.
//!
//! Bucket selection is deliberately coarse: we want "2 weeks ago" not
//! "13 days, 4 hours ago". Sub-second precision and DST/timezone
//! arithmetic are intentionally ignored — the input is UTC, the output
//! is fuzzy.

use chrono::{DateTime, Utc};

/// Format `then` as a relative age compared to `now`, in English.
///
/// Returns one of:
/// - `"just now"` — under one minute (also returned when `then > now`,
///   e.g. clock skew between machines that share the metadata file).
/// - `"N minute(s) ago"` — under one hour.
/// - `"N hour(s) ago"` — under one day.
/// - `"N day(s) ago"` — under one week.
/// - `"N week(s) ago"` — under 30 days.
/// - `"N month(s) ago"` — under 365 days. A "month" is 30 days here:
///   calendar months would require timezone-aware arithmetic for a
///   marginal gain in accuracy on a label already labelled "fuzzy".
/// - `"N year(s) ago"` — 365 days and beyond.
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Utc};
/// use scoop_uv::output::format_age;
///
/// let now = Utc::now();
/// let two_hours_ago = now - Duration::hours(2);
/// assert_eq!(format_age(two_hours_ago, now), "2 hours ago");
/// ```
pub fn format_age(then: DateTime<Utc>, now: DateTime<Utc>) -> String {
    // Future timestamp (clock skew, NFS, two machines disagreeing): we
    // can't honestly say "in 5 minutes", so flatten it to "just now".
    let secs = match (now - then).num_seconds() {
        n if n < 0 => return "just now".to_string(),
        n => n,
    };

    if secs < 60 {
        return "just now".to_string();
    }

    let minutes = secs / 60;
    if minutes < 60 {
        return pluralize(minutes, "minute");
    }

    let hours = minutes / 60;
    if hours < 24 {
        return pluralize(hours, "hour");
    }

    let days = hours / 24;
    if days < 7 {
        return pluralize(days, "day");
    }
    if days < 30 {
        return pluralize(days / 7, "week");
    }
    if days < 365 {
        return pluralize(days / 30, "month");
    }
    pluralize(days / 365, "year")
}

fn pluralize(n: i64, unit: &str) -> String {
    if n == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{n} {unit}s ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn anchor() -> DateTime<Utc> {
        "2026-06-02T12:00:00Z".parse().unwrap()
    }

    #[test]
    fn just_now_for_under_a_minute() {
        let now = anchor();
        assert_eq!(format_age(now, now), "just now");
        assert_eq!(format_age(now - Duration::seconds(59), now), "just now");
    }

    #[test]
    fn minutes_singular_at_one() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::seconds(60), now), "1 minute ago");
    }

    #[test]
    fn minutes_plural_above_one() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::minutes(2), now), "2 minutes ago");
        assert_eq!(
            format_age(now - Duration::minutes(59), now),
            "59 minutes ago"
        );
    }

    #[test]
    fn hours_bucket_with_boundary() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::minutes(60), now), "1 hour ago");
        assert_eq!(format_age(now - Duration::hours(23), now), "23 hours ago");
    }

    #[test]
    fn days_bucket_with_boundary() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::hours(24), now), "1 day ago");
        assert_eq!(format_age(now - Duration::days(6), now), "6 days ago");
    }

    #[test]
    fn weeks_bucket_with_boundary() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::days(7), now), "1 week ago");
        assert_eq!(format_age(now - Duration::days(29), now), "4 weeks ago");
    }

    #[test]
    fn months_bucket_with_boundary() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::days(30), now), "1 month ago");
        assert_eq!(format_age(now - Duration::days(364), now), "12 months ago");
    }

    #[test]
    fn years_bucket() {
        let now = anchor();
        assert_eq!(format_age(now - Duration::days(365), now), "1 year ago");
        assert_eq!(format_age(now - Duration::days(730), now), "2 years ago");
    }

    #[test]
    fn future_timestamps_flatten_to_just_now() {
        // Clock skew across machines or NFS clients can produce a
        // last_used in our future. We must not render "in 5 minutes".
        let now = anchor();
        assert_eq!(format_age(now + Duration::minutes(5), now), "just now");
        assert_eq!(format_age(now + Duration::days(1), now), "just now");
    }

    #[test]
    fn pluralization_at_every_singular_boundary() {
        // Pins the "singular at exactly 1" contract for each bucket so
        // a mutation flipping `== 1` to `<= 1` or `< 1` gets caught.
        let now = anchor();
        for s in [
            "1 minute ago",
            "1 hour ago",
            "1 day ago",
            "1 week ago",
            "1 month ago",
            "1 year ago",
        ] {
            let d = match s {
                "1 minute ago" => Duration::minutes(1),
                "1 hour ago" => Duration::hours(1),
                "1 day ago" => Duration::days(1),
                "1 week ago" => Duration::days(7),
                "1 month ago" => Duration::days(30),
                "1 year ago" => Duration::days(365),
                _ => unreachable!(),
            };
            assert_eq!(format_age(now - d, now), s, "bucket mismatch for {s}");
        }
    }
}
