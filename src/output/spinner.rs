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
    fn spinner_new_sets_initial_message() {
        let spinner = Spinner::new("Loading...");
        assert_eq!(spinner.bar.message(), "Loading...");
        assert!(
            !spinner.bar.is_finished(),
            "New spinner should not be finished"
        );
    }

    #[test]
    fn spinner_new_with_empty_message_is_valid() {
        let spinner = Spinner::new("");
        assert_eq!(spinner.bar.message(), "");
        assert!(!spinner.bar.is_finished());
    }

    #[test]
    fn spinner_new_preserves_unicode_message() {
        let spinner = Spinner::new("로딩 중...");
        assert_eq!(spinner.bar.message(), "로딩 중...");
    }

    // =========================================================================
    // Message Update Tests
    // =========================================================================

    #[test]
    fn set_message_updates_current_message() {
        let spinner = Spinner::new("Initial");
        assert_eq!(spinner.bar.message(), "Initial");

        spinner.set_message("Updated");
        assert_eq!(spinner.bar.message(), "Updated");
    }

    #[test]
    fn set_message_allows_empty_string() {
        let spinner = Spinner::new("Initial");
        spinner.set_message("");
        assert_eq!(spinner.bar.message(), "");
    }

    #[test]
    fn set_message_multiple_times_keeps_last() {
        let spinner = Spinner::new("Step 1");
        spinner.set_message("Step 2");
        spinner.set_message("Step 3");
        spinner.set_message("Final");

        assert_eq!(spinner.bar.message(), "Final");
    }

    // =========================================================================
    // Finish Tests
    // =========================================================================

    #[test]
    fn finish_with_message_marks_as_finished() {
        let spinner = Spinner::new("Working...");
        assert!(!spinner.bar.is_finished());

        spinner.finish_with_message("Done!");
        assert!(spinner.bar.is_finished(), "Spinner should be finished");
        assert_eq!(spinner.bar.message(), "Done!");
    }

    #[test]
    fn finish_with_empty_message_still_finishes() {
        let spinner = Spinner::new("Working...");
        spinner.finish_with_message("");

        assert!(spinner.bar.is_finished());
        assert_eq!(spinner.bar.message(), "");
    }

    #[test]
    fn finish_and_clear_marks_as_finished() {
        let spinner = Spinner::new("Working...");
        spinner.finish_and_clear();

        assert!(
            spinner.bar.is_finished(),
            "Spinner should be finished after clear"
        );
    }

    // =========================================================================
    // Lifecycle Tests
    // =========================================================================

    #[test]
    fn spinner_full_lifecycle_tracks_state() {
        let spinner = Spinner::new("Starting...");
        assert_eq!(spinner.bar.message(), "Starting...");
        assert!(!spinner.bar.is_finished());

        spinner.set_message("Processing...");
        assert_eq!(spinner.bar.message(), "Processing...");

        spinner.set_message("Finalizing...");
        assert_eq!(spinner.bar.message(), "Finalizing...");

        spinner.finish_with_message("Complete!");
        assert!(spinner.bar.is_finished());
        assert_eq!(spinner.bar.message(), "Complete!");
    }

    #[test]
    fn spinner_lifecycle_with_clear_finishes() {
        let spinner = Spinner::new("Background task");
        spinner.set_message("Still running...");
        assert_eq!(spinner.bar.message(), "Still running...");

        spinner.finish_and_clear();
        assert!(spinner.bar.is_finished());
    }

    #[test]
    fn multiple_spinners_independent_state() {
        let spinner1 = Spinner::new("First");
        let spinner2 = Spinner::new("Second");

        // Both start unfinished
        assert!(!spinner1.bar.is_finished());
        assert!(!spinner2.bar.is_finished());

        // Finish first, second still running
        spinner1.finish_with_message("Done 1");
        assert!(spinner1.bar.is_finished());
        assert!(
            !spinner2.bar.is_finished(),
            "Second spinner should be independent"
        );

        // Finish second
        spinner2.finish_with_message("Done 2");
        assert!(spinner2.bar.is_finished());
        assert_eq!(spinner1.bar.message(), "Done 1");
        assert_eq!(spinner2.bar.message(), "Done 2");
    }
}
