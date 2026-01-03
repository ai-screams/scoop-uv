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
