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
