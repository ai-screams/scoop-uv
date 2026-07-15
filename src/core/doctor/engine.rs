use super::*;

// ============================================================================
// Doctor Engine
// ============================================================================

/// Doctor diagnostic engine.
///
/// Runs all registered checks and collects results.
pub struct Doctor {
    pub(super) checks: Vec<Box<dyn Check>>,
}

impl Doctor {
    /// Creates a new Doctor with default checks.
    pub fn new() -> Self {
        Self {
            checks: vec![
                Box::new(UvCheck),
                Box::new(HomeCheck),
                Box::new(VirtualenvCheck),
                Box::new(SymlinkCheck),
                Box::new(ShellCheck),
                Box::new(VersionCheck),
                Box::new(LegacyCheck),
            ],
        }
    }

    /// Runs all checks and returns results.
    pub fn run_all(&self) -> Vec<CheckResult> {
        self.checks.iter().flat_map(|c| c.run()).collect()
    }

    /// Runs all checks and attempts to fix issues where possible.
    ///
    /// Returns the results after attempting fixes.
    pub fn run_and_fix(&self, output: &crate::output::Output) -> Vec<CheckResult> {
        let mut all_results = Vec::new();

        for check in &self.checks {
            let results = check.run();

            for result in results {
                // Attempt auto-fix for specific error types
                if result.is_error() {
                    if let Some(fixed_result) = check.fix(&result, output) {
                        output.doctor_check(&fixed_result);
                        all_results.push(fixed_result);
                        continue;
                    }
                }

                output.doctor_check(&result);
                all_results.push(result);
            }
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

    fn quiet_output() -> crate::output::Output {
        crate::output::Output::new(0, true, true, false)
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
}
