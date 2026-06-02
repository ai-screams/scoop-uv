//! Tiny age-style duration parser shared by gc.
//!
//! Accepts `<n><suffix>` where `<n>` is a positive integer and
//! `<suffix>` is `d` (days), `w` (weeks), or `y` (years, 365 d). Built
//! hand-rolled instead of pulling in the `humantime` crate: we need
//! literally one form, predictable rejection of month-style inputs
//! (`m` is ambiguous between minute and month), and zero new
//! dependencies. The output is a [`chrono::Duration`] so callers can
//! subtract from `Utc::now()` via [`chrono::DateTime::checked_sub_signed`]
//! to produce a cutoff instant (bare `-` would panic on overflow).
//!
//! `pub(crate)` only: `gc::execute` is the lone consumer. If a second
//! consumer shows up, lift this to its own module.

use chrono::Duration;

use crate::error::{Result, ScoopError};

/// Hard cap on the final day count.
///
/// 200 calendar years (`365 * 200`) is well outside any realistic
/// `--older-than` window and well inside chrono's representable range
/// (~262k years). Picking the cap on *days* (not the unit-prefixed
/// value) means `200y`, `~10400w`, and `73000d` all hit the same
/// ceiling — no funny "1000000d slipped through because the multiplier
/// was 1" surprises. Codex review on the previous commit caught a
/// pre-cap `MAX_VALUE = 1_000_000` that would have let `1000000y`
/// panic on the eventual `Utc::now() - duration` in Step 5.
const MAX_DAYS: i64 = 365 * 200;

/// Parse an age string like `"30d"` / `"2w"` / `"1y"` into a
/// [`chrono::Duration`].
///
/// # Suffix table
///
/// | Suffix | Meaning  | Multiplier      |
/// |--------|----------|-----------------|
/// | `d`    | days     | 1 day           |
/// | `w`    | weeks    | 7 days          |
/// | `y`    | years    | 365 days        |
///
/// `m` is deliberately **not** accepted — between "minute" and "month"
/// it has no unambiguous reading at this granularity, and calendar
/// months would require timezone-aware arithmetic for a marginal gain
/// in accuracy on a stale-env heuristic. Users who want a one-month
/// window can write `30d`; those who want a year can write `1y`.
///
/// # Errors
///
/// - Empty input.
/// - Missing or unknown suffix (e.g. `"30"`, `"5h"`, `"6m"`).
/// - Numeric prefix that doesn't parse as `u64` or exceeds [`MAX_VALUE`].
/// - Zero value (`"0d"`) — `--older-than 0d` would match every env
///   including the one created this second, which is almost never what
///   the user means; the explicit error nudges them toward `gc` without
///   the flag.
/// - Multiplication overflow when computing the final [`Duration`]
///   (defensive — `MAX_VALUE` already prevents this for valid suffixes,
///   but the check pins the contract).
///
/// # Examples
///
/// See unit tests below for the full matrix; the parser is `pub(crate)`
/// because Step 5 (`gc --older-than`) is the only consumer.
pub(crate) fn parse_duration(s: &str) -> Result<Duration> {
    if s.is_empty() {
        return Err(invalid("duration cannot be empty"));
    }

    // Locate the suffix as the first non-digit run, so multi-char
    // suffixes (`mo`, `min`, ...) are detected as a unit instead of
    // collapsing into the numeric prefix. Without this, `1mo` would
    // parse as num="1m" + suffix="o" and surface a confusing
    // "could not parse '1m' as a positive integer" error.
    let suffix_start = s
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| invalid("duration must end with a unit suffix (d/w/y), e.g. 30d"))?;

    let (num_part, suffix) = s.split_at(suffix_start);

    if num_part.is_empty() {
        return Err(invalid("duration must start with a number, e.g. 30d"));
    }

    // `parse::<u64>` already rejects `-30d`; `+30d` would slip through
    // because `+` is not a digit (so split_at sees the `+` in the
    // suffix). Belt-and-suspenders: refuse both signs explicitly.
    if num_part.starts_with('+') || num_part.starts_with('-') {
        return Err(invalid("duration must be a positive number, e.g. 30d"));
    }

    let value: u64 = num_part.parse().map_err(|_| {
        invalid(&format!(
            "could not parse '{num_part}' as a positive integer"
        ))
    })?;

    if value == 0 {
        return Err(invalid(
            "duration must be greater than zero — '0d' would match every env",
        ));
    }

    // A clean suffix must be alphabetic-only (no digits, no whitespace,
    // no punctuation). Inputs like `200y2d` (combined durations),
    // `30 d` (whitespace), or `30d!` (junk) produce confusing "unknown
    // suffix '<stuff>'" messages when the real problem is structural —
    // the parser doesn't compose multiple units or tolerate separators.
    // Surface that explicitly so the hint points at the actual mistake.
    if !suffix.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(invalid(&format!(
            "duration must be a single integer followed by one unit suffix \
             (e.g. 30d). '{suffix}' is not a recognized suffix — combined \
             durations (like '1y6m') and whitespace-separated forms are not \
             supported; pick one unit"
        )));
    }

    let days_per_unit: u64 = match suffix {
        "d" => 1,
        "w" => 7,
        // 365 d — calendar years dropped on purpose (see module doc).
        "y" => 365,
        // Month variants get the most-likely-mistake error so users
        // don't get bounced by a generic "unknown suffix" hint. `m` is
        // also ambiguous with "minute"; the message calls out both
        // readings instead of guessing.
        "m" | "mo" | "mon" | "month" | "months" => {
            return Err(invalid(
                "month suffixes (m/mo/month) are ambiguous between minutes \
                 and months; use 'd' (days), 'w' (weeks), or 'y' (years=365d)",
            ));
        }
        _ => {
            return Err(invalid(&format!(
                "unknown duration suffix '{suffix}'; expected 'd', 'w', or 'y'"
            )));
        }
    };

    // Multiply in u64 to avoid the `(u64 as i64)` silent-truncation
    // trap Codex flagged on the previous commit: `18446744073709551615d`
    // would have cast to `-1`, producing a "tomorrow" cutoff that
    // matched every env. Stay in u64 until we've capped the day count;
    // only then convert to the i64 chrono needs.
    let total_days_u64 = value
        .checked_mul(days_per_unit)
        .ok_or_else(|| invalid("duration arithmetic overflowed"))?;

    if total_days_u64 > MAX_DAYS as u64 {
        return Err(invalid(&format!(
            "duration too large (max {MAX_DAYS} days ≈ 200 years)"
        )));
    }

    let total_days =
        i64::try_from(total_days_u64).map_err(|_| invalid("duration arithmetic overflowed"))?;

    // `Duration::days` panics on internal overflow; `try_days` returns
    // None instead. Within the MAX_DAYS cap we'll never trip this, but
    // keeping the checked path means a future cap bump can't silently
    // re-introduce a panic.
    Duration::try_days(total_days).ok_or_else(|| invalid("duration arithmetic overflowed"))
}

/// Build an `InvalidArgument` error. The error's `Display` impl
/// already routes through the `error.invalid_argument` i18n key, so
/// we just pass the raw reason string — wrapping it in another
/// `t!("error.invalid_argument", ...)` call (as an earlier draft did)
/// would double-apply the template.
fn invalid(reason: &str) -> ScoopError {
    ScoopError::InvalidArgument {
        message: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_days() {
        assert_eq!(parse_duration("1d").unwrap(), Duration::days(1));
        assert_eq!(parse_duration("30d").unwrap(), Duration::days(30));
        assert_eq!(parse_duration("365d").unwrap(), Duration::days(365));
    }

    #[test]
    fn parses_weeks_as_seven_days() {
        // Pins the 7-day-per-week conversion — a mutation flipping it
        // to 6 or 8 trips this assertion.
        assert_eq!(parse_duration("1w").unwrap(), Duration::days(7));
        assert_eq!(parse_duration("2w").unwrap(), Duration::days(14));
        assert_eq!(parse_duration("52w").unwrap(), Duration::days(364));
    }

    #[test]
    fn parses_years_as_365_days_no_leap() {
        // 365 is the contract — calendar years are deliberately dropped
        // (see module doc). Pin it so a "more accurate" refactor that
        // switches to 365.25 / 366 trips this test.
        assert_eq!(parse_duration("1y").unwrap(), Duration::days(365));
        assert_eq!(parse_duration("2y").unwrap(), Duration::days(730));
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn rejects_zero() {
        // The "0d would match everything" guard. Every suffix matters
        // here because the value=0 check runs before the suffix match.
        for s in ["0d", "0w", "0y"] {
            assert!(parse_duration(s).is_err(), "should reject {s}");
        }
    }

    #[test]
    fn rejects_unknown_suffix() {
        for s in ["30h", "5s", "1mi", "30x"] {
            assert!(parse_duration(s).is_err(), "should reject {s}");
        }
    }

    #[test]
    fn rejects_months_with_specific_hint() {
        // `m` is the most likely mistake — make sure the error message
        // calls out month vs minute ambiguity, not just "unknown suffix".
        let err = parse_duration("6m").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("month") || msg.contains("ambiguous"),
            "month rejection should explain why: {msg}"
        );
    }

    #[test]
    fn rejects_no_suffix() {
        // Bare numbers (`30`) are common typos — must error, not be
        // interpreted as days or seconds.
        assert!(parse_duration("30").is_err());
        assert!(parse_duration("1").is_err());
    }

    #[test]
    fn rejects_no_number() {
        assert!(parse_duration("d").is_err());
        assert!(parse_duration("w").is_err());
    }

    #[test]
    fn rejects_signed_values() {
        // u64 already rejects `-`, but `+30d` is the more interesting
        // case — guard against an over-friendly future "always positive"
        // refactor accidentally accepting it.
        assert!(parse_duration("-30d").is_err());
        assert!(parse_duration("+30d").is_err());
    }

    #[test]
    fn rejects_value_past_max_days_in_each_unit() {
        // Cap is on the *day count*, so passing the limit in any unit
        // must reject. Pinning all three keeps a future "raise the cap
        // for years only" tweak from re-introducing the panic surface
        // Codex flagged on the previous commit.
        let past_in_days = format!("{}d", MAX_DAYS + 1);
        let past_in_weeks = format!("{}w", (MAX_DAYS / 7) + 1);
        let past_in_years = format!("{}y", (MAX_DAYS / 365) + 1);
        for s in [&past_in_days, &past_in_weeks, &past_in_years] {
            assert!(parse_duration(s).is_err(), "should reject {s}");
        }
    }

    #[test]
    fn max_days_in_years_does_not_panic_on_cutoff() {
        // The whole reason for MAX_DAYS = 365*200 is that 200y leaves
        // plenty of headroom inside chrono's representable range, so
        // a caller doing `Utc::now().checked_sub_signed(d)` returns
        // `Some(_)`. If a future refactor raises the cap and breaks
        // this, callers start panicking on a CLI argument.
        let d = parse_duration("200y").expect("200y must parse");
        let now = chrono::Utc::now();
        assert!(
            now.checked_sub_signed(d).is_some(),
            "cutoff math must not overflow for MAX_DAYS",
        );
    }

    #[test]
    fn rejects_garbage() {
        for s in ["abc", "30dx", "d30", "1.5d", "1 d", "  30d  "] {
            assert!(parse_duration(s).is_err(), "should reject {s:?}");
        }
    }

    #[test]
    fn combined_or_separated_durations_surface_specific_hint() {
        // Inputs like "200y2d" / "30 d" / "30d!" used to produce a
        // confusing "unknown duration suffix '<stuff>'" message; the
        // alphabetic-only guard now calls out the structural mistake
        // (combined units / whitespace / junk) so the user sees the
        // real problem.
        for s in ["200y2d", "1y6m", "30 d", "30d!"] {
            let err = parse_duration(s).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("single integer") || msg.contains("combined"),
                "{s} should surface structural-mistake hint, got: {msg}"
            );
        }
    }

    #[test]
    fn max_days_boundary_accepted() {
        // MAX_DAYS exactly must parse — the bound is inclusive. Pins
        // the contract so a `> MAX_DAYS` → `>= MAX_DAYS` mutation gets
        // caught.
        let exact_days = format!("{MAX_DAYS}d");
        assert!(parse_duration(&exact_days).is_ok());
        // Same value, expressed in years — equivalent to MAX_DAYS, so
        // it must also be accepted.
        assert!(parse_duration("200y").is_ok());
    }

    #[test]
    fn rejects_month_variants_with_specific_hint() {
        // Codex MEDIUM-2 on this commit's predecessor: `1mo` used to
        // parse as num="1m" + suffix="o" and surface "could not parse"
        // instead of the month-ambiguity hint. Walk every variant to
        // make sure all month-shaped inputs hit the dedicated arm.
        for s in ["6m", "6mo", "6mon", "6month", "6months"] {
            let err = parse_duration(s).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("month") || msg.contains("ambiguous"),
                "{s} must surface month-ambiguity hint, got: {msg}"
            );
        }
    }
}
