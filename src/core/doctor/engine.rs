use super::types::{Check, CheckResult};

// ============================================================================
// Doctor Engine
// ============================================================================

/// Doctor diagnostic engine.
///
/// Runs all registered checks and collects results.
pub struct Doctor {
    checks: Vec<Box<dyn Check>>,
}

impl Doctor {
    /// Creates a new Doctor with default checks.
    pub fn new() -> Self {
        Self {
            checks: super::checks::default_checks(),
        }
    }

    /// Runs all checks and returns results.
    pub fn run_all(&self) -> Vec<CheckResult> {
        self.checks.iter().flat_map(|c| c.run()).collect()
    }

    /// Runs all checks and attempts to fix issues where possible.
    ///
    /// Returns the results after attempting fixes. A fix can also clear an
    /// error another check reported: relinking an env's interpreter mends the
    /// "broken virtualenv" the virtualenvs check found before the symlink
    /// check fixed it. So once anything is fixed, every check that still had
    /// an unfixed error runs again and its fresh results replace the old.
    pub fn run_and_fix(&self, output: &crate::output::Output) -> Vec<CheckResult> {
        let mut per_check: Vec<Vec<CheckResult>> = Vec::with_capacity(self.checks.len());
        let mut fixed_any = false;

        for check in &self.checks {
            let mut results = Vec::new();
            for result in check.run() {
                if result.is_error()
                    && let Some(fixed_result) = check.fix(&result, output)
                {
                    fixed_any = true;
                    results.push(fixed_result);
                } else {
                    results.push(result);
                }
            }
            per_check.push(results);
        }

        if fixed_any {
            for (check, results) in self.checks.iter().zip(per_check.iter_mut()) {
                if results.iter().any(CheckResult::is_error) {
                    *results = check.run();
                }
            }
        }

        let all_results: Vec<CheckResult> = per_check.into_iter().flatten().collect();
        for result in &all_results {
            output.doctor_check(result);
        }
        all_results
    }
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A check whose error IS fixable — its `fix` override returns `Some(ok)`.
    struct FixableCheck;
    impl Check for FixableCheck {
        fn id(&self) -> &'static str {
            "fixable"
        }
        fn name(&self) -> &'static str {
            "fixable check"
        }
        fn run(&self) -> Vec<CheckResult> {
            vec![CheckResult::error("fixable", "fixable check", "boom")]
        }
        fn fix(
            &self,
            result: &CheckResult,
            _output: &crate::output::Output,
        ) -> Option<CheckResult> {
            // Only fixes an Error status (mirrors the real checks' guard).
            if result.is_error() {
                Some(CheckResult::ok("fixable", "fixable check"))
            } else {
                None
            }
        }
    }

    /// A check whose error is NOT fixable — relies on the default `fix` (returns `None`).
    struct UnfixableCheck;
    impl Check for UnfixableCheck {
        fn id(&self) -> &'static str {
            "unfixable"
        }
        fn name(&self) -> &'static str {
            "unfixable check"
        }
        fn run(&self) -> Vec<CheckResult> {
            vec![CheckResult::error(
                "unfixable",
                "unfixable check",
                "still broken",
            )]
        }
    }

    /// Reports an error while `broken` is set; never fixes it itself.
    struct DependentCheck(std::sync::Arc<std::sync::atomic::AtomicBool>);
    impl Check for DependentCheck {
        fn id(&self) -> &'static str {
            "dependent"
        }
        fn name(&self) -> &'static str {
            "dependent check"
        }
        fn run(&self) -> Vec<CheckResult> {
            if self.0.load(std::sync::atomic::Ordering::SeqCst) {
                vec![CheckResult::error("dependent", "dependent check", "broken")]
            } else {
                vec![CheckResult::ok("dependent", "dependent check")]
            }
        }
    }

    /// Fixes the shared `broken` state, as relinking mends a virtualenv.
    struct RepairingCheck(std::sync::Arc<std::sync::atomic::AtomicBool>);
    impl Check for RepairingCheck {
        fn id(&self) -> &'static str {
            "repairing"
        }
        fn name(&self) -> &'static str {
            "repairing check"
        }
        fn run(&self) -> Vec<CheckResult> {
            DependentCheck(self.0.clone()).run()
        }
        fn fix(
            &self,
            _result: &CheckResult,
            _output: &crate::output::Output,
        ) -> Option<CheckResult> {
            self.0.store(false, std::sync::atomic::Ordering::SeqCst);
            Some(CheckResult::ok("repairing", "repairing check"))
        }
    }

    /// An error reported by an earlier check and mended by a later check's
    /// fix is not left in the result: `doctor --fix` said "Fixed symlink"
    /// and still counted the env's "broken virtualenv" error, exit 2.
    /// Fails if the re-run after a fix is dropped.
    #[test]
    fn run_and_fix_reruns_checks_a_later_fix_mended() {
        let broken = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let doctor = Doctor {
            checks: vec![
                Box::new(DependentCheck(broken.clone())),
                Box::new(RepairingCheck(broken.clone())),
                Box::new(UnfixableCheck),
            ],
        };
        let results = doctor.run_and_fix(&quiet_output());
        let errors: Vec<_> = results
            .iter()
            .filter(|r| r.is_error())
            .map(|r| r.id)
            .collect();
        // The unrelated unfixable error stays.
        assert_eq!(errors, ["unfixable"]);
        assert_eq!(results.len(), 3);
    }

    fn quiet_output() -> crate::output::Output {
        crate::output::Output::new(0, true, crate::output::Colors::NONE, false)
    }

    #[test]
    fn run_all_collects_every_check_result_without_fixing() {
        // run_all must return the raw run() results (no fix pass). A `-> vec![]`
        // mutant would drop them all.
        let doctor = Doctor {
            checks: vec![Box::new(FixableCheck), Box::new(UnfixableCheck)],
        };
        let results = doctor.run_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.id == "fixable" && r.is_error()));
        assert!(results.iter().any(|r| r.id == "unfixable" && r.is_error()));
    }

    #[test]
    fn run_and_fix_replaces_error_when_fix_returns_some() {
        let doctor = Doctor {
            checks: vec![Box::new(FixableCheck)],
        };
        let results = doctor.run_and_fix(&quiet_output());
        assert_eq!(results.len(), 1);
        assert!(
            results[0].is_ok(),
            "a fixable error must be replaced by the fixed ok result: {:#?}",
            results[0]
        );
    }

    #[test]
    fn run_and_fix_keeps_raw_error_when_fix_returns_none() {
        let doctor = Doctor {
            checks: vec![Box::new(UnfixableCheck)],
        };
        let results = doctor.run_and_fix(&quiet_output());
        assert_eq!(results.len(), 1);
        assert!(
            results[0].is_error(),
            "an unfixable error must pass through unchanged"
        );
    }

    #[test]
    fn run_and_fix_dispatches_fix_per_check() {
        // Both checks run; only the fixable one's error is replaced — proving the
        // fix is dispatched on the producing check, not applied globally.
        let doctor = Doctor {
            checks: vec![Box::new(FixableCheck), Box::new(UnfixableCheck)],
        };
        let results = doctor.run_and_fix(&quiet_output());
        assert_eq!(results.len(), 2);
        assert!(
            results.iter().any(|r| r.id == "fixable" && r.is_ok()),
            "fixable check's error should be fixed"
        );
        assert!(
            results.iter().any(|r| r.id == "unfixable" && r.is_error()),
            "unfixable check's error should remain"
        );
    }

    #[test]
    fn test_doctor_has_default_checks() {
        let doctor = Doctor::new();
        assert!(!doctor.checks.is_empty());
    }

    /// The warn-only remnant check must stay registered: it is what makes an
    /// incomplete 0.15 → 0.16 upgrade visible. Fails if it is dropped from
    /// `default_checks`.
    #[test]
    fn doctor_registers_legacy_check() {
        let doctor = Doctor::new();
        assert!(
            doctor.checks.iter().any(|c| c.id() == "legacy"),
            "Doctor::new() must register the legacy check"
        );
    }
}
