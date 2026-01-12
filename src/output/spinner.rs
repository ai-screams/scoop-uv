//! Spinner utilities

use indicatif::{ProgressBar, ProgressStyle};

/// A simple spinner for long-running operations
pub struct Spinner {
    bar: ProgressBar,
}

impl Spinner {
    /// Create a new spinner with a message
    pub fn new(msg: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .expect("Invalid spinner template"),
        );
        bar.set_message(msg.to_string());
        bar.enable_steady_tick(std::time::Duration::from_millis(100));

        Self { bar }
    }

    /// Update the spinner message
    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    /// Finish the spinner with a success message
    pub fn finish_with_message(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    /// Finish and clear the spinner
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Spinner Creation Tests
    // =========================================================================

    #[test]
    fn spinner_new_does_not_panic() {
        let _spinner = Spinner::new("Loading...");
    }

    #[test]
    fn spinner_new_with_empty_message() {
        let _spinner = Spinner::new("");
    }

    #[test]
    fn spinner_new_with_unicode_message() {
        let _spinner = Spinner::new("로딩 중...");
    }

    // =========================================================================
    // Message Update Tests
    // =========================================================================

    #[test]
    fn set_message_does_not_panic() {
        let spinner = Spinner::new("Initial");
        spinner.set_message("Updated");
    }

    #[test]
    fn set_message_with_empty_string() {
        let spinner = Spinner::new("Initial");
        spinner.set_message("");
    }

    #[test]
    fn set_message_multiple_times() {
        let spinner = Spinner::new("Step 1");
        spinner.set_message("Step 2");
        spinner.set_message("Step 3");
        spinner.set_message("Final");
    }

    // =========================================================================
    // Finish Tests
    // =========================================================================

    #[test]
    fn finish_with_message_does_not_panic() {
        let spinner = Spinner::new("Working...");
        spinner.finish_with_message("Done!");
    }

    #[test]
    fn finish_with_message_empty_string() {
        let spinner = Spinner::new("Working...");
        spinner.finish_with_message("");
    }

    #[test]
    fn finish_and_clear_does_not_panic() {
        let spinner = Spinner::new("Working...");
        spinner.finish_and_clear();
    }

    // =========================================================================
    // Lifecycle Tests
    // =========================================================================

    #[test]
    fn spinner_full_lifecycle() {
        let spinner = Spinner::new("Starting...");
        spinner.set_message("Processing...");
        spinner.set_message("Finalizing...");
        spinner.finish_with_message("Complete!");
    }

    #[test]
    fn spinner_lifecycle_with_clear() {
        let spinner = Spinner::new("Background task");
        spinner.set_message("Still running...");
        spinner.finish_and_clear();
    }

    #[test]
    fn multiple_spinners_sequential() {
        let spinner1 = Spinner::new("First");
        spinner1.finish_with_message("Done 1");

        let spinner2 = Spinner::new("Second");
        spinner2.finish_with_message("Done 2");
    }
}
