//! Tiny age-style duration parser shared by gc.
//!
//! Accepts `<n><suffix>` where `<n>` is a positive integer and
//! `<suffix>` is `d` (days), `w` (weeks), or `y` (years, 365 d). Built
//! hand-rolled instead of pulling in the `humantime` crate: we need
//! literally one form, predictable rejection of month-style inputs
//! (`m` is ambiguous between minute and month), and zero new
//! dependencies. The output is a `chrono::Duration` so callers can
//! subtract directly from `Utc::now()`.
//!
//! Not exported from the crate — `gc::execute` is the only consumer.
//! If a second consumer shows up, lift this to its own module.

use chrono::Duration;
use rust_i18n::t;

use crate::error::{Result, ScoopError};

/// Maximum value we'll accept for the numeric prefix. Picked so that
/// `<n> * 86_400 * 365` fits in `i64` with room for arithmetic later.
/// In practice "older-than 200 years" is nonsense for an env, so the
/// limit isn't even close to the meaningful range; this is just
/// belt-and-suspenders against `i64` overflow during multiplication.
const MAX_VALUE: u64 = 1_000_000;

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
/// ```
/// # use chrono::Duration;
/// # use scoop_uv::cli::commands::parse_duration;
/// assert_eq!(parse_duration("30d").unwrap(), Duration::days(30));
/// assert_eq!(parse_duration("2w").unwrap(), Duration::days(14));
/// assert_eq!(parse_duration("1y").unwrap(), Duration::days(365));
/// assert!(parse_duration("6m").is_err()); // month: not accepted
/// assert!(parse_duration("0d").is_err()); // zero: not accepted
/// ```
pub fn parse_duration(s: &str) -> Result<Duration> {
    if s.is_empty() {
        return Err(invalid("duration cannot be empty"));
    }

    // Split into numeric prefix + 1-char suffix. ASCII-only by design:
    // the suffix table is `d`/`w`/`y`, no need to walk Unicode boundaries.
    let bytes = s.as_bytes();
    let last = *bytes.last().unwrap(); // safe: empty case checked above

    if !last.is_ascii_alphabetic() {
        return Err(invalid(
            "duration must end with a unit suffix (d/w/y), e.g. 30d",
        ));
    }

    let (num_part, suffix) = s.split_at(s.len() - 1);
    if num_part.is_empty() {
        return Err(invalid("duration must start with a number, e.g. 30d"));
    }

    // Explicitly reject signed prefixes — `parse::<u64>` already does
    // this for `-30d`, but `+30d` would parse, which we don't want
    // (suggests a "since now" form we don't support).
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

    if value > MAX_VALUE {
        return Err(invalid(&format!(
            "duration too large (max {MAX_VALUE}{suffix})"
        )));
    }

    let days_per_unit: i64 = match suffix {
        "d" => 1,
        "w" => 7,
        // 365 d — calendar years dropped on purpose (see module doc).
        "y" => 365,
        // Explicit "no months" error rather than the generic "unknown
        // suffix" message: it's the most likely user mistake (Codex M5
        // call-out) and deserves a specific hint.
        "m" => {
            return Err(invalid(
                "month suffix 'm' is ambiguous between minutes and months; \
                 use 'd' (days), 'w' (weeks), or 'y' (years=365d)",
            ));
        }
        _ => {
            return Err(invalid(&format!(
                "unknown duration suffix '{suffix}'; expected 'd', 'w', or 'y'"
            )));
        }
    };

    // Safe by construction: value <= MAX_VALUE (1_000_000), days_per_unit <= 365.
    // i64::MAX / 365 ≈ 2.5e16, so the cast and multiply fit comfortably.
    // Still go through checked_mul to pin the contract — if MAX_VALUE
    // is ever raised, this catches the overflow rather than wrapping.
    let total_days = (value as i64)
        .checked_mul(days_per_unit)
        .ok_or_else(|| invalid("duration arithmetic overflowed"))?;

    Ok(Duration::days(total_days))
}

/// Build an `InvalidArgument` error so the caller's `--older-than`
/// invocation surfaces the parse reason instead of a generic "bad
/// flag". Reuses the existing `error.invalid_argument` i18n key —
/// the parse reasons themselves are English-only engineering hints
/// (matching how other parse errors thread through in this crate).
fn invalid(reason: &str) -> ScoopError {
    ScoopError::InvalidArgument {
        message: t!("error.invalid_argument", message = reason).to_string(),
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
    fn rejects_overflow_value() {
        // MAX_VALUE + 1 must reject before the multiply step.
        let too_big = format!("{}d", MAX_VALUE + 1);
        assert!(parse_duration(&too_big).is_err());
    }

    #[test]
    fn rejects_garbage() {
        for s in ["abc", "30dx", "d30", "1.5d", "1 d", "  30d  "] {
            assert!(parse_duration(s).is_err(), "should reject {s:?}");
        }
    }

    #[test]
    fn max_value_boundary_accepted() {
        // MAX_VALUE exactly must still parse, so the bound is
        // inclusive — pinning it prevents a `> MAX_VALUE` → `>= MAX_VALUE`
        // mutant from sneaking through.
        let exact = format!("{MAX_VALUE}d");
        assert!(parse_duration(&exact).is_ok());
    }
}
