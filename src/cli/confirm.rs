//! Yes/no confirmation before a destructive step.
//!
//! dialoguer draws the prompt on stderr and refuses when stderr is not a
//! terminal (`2>log`); it reads the answer from the terminal even when stdin
//! is redirected. So the only thing checked here is stderr. Each caller
//! decides what an unavailable prompt means (an error naming the flag that
//! skips it, or "no").

use std::io::IsTerminal;

use dialoguer::Confirm;

/// The outcome of asking.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Confirmation {
    Yes,
    No,
    /// The question could not be asked or answered: stderr is not a
    /// terminal, or the prompt failed while drawing or reading.
    Unavailable,
}

/// Asks `prompt`, defaulting to no.
pub(crate) fn confirm(prompt: &str) -> Confirmation {
    #[cfg(test)]
    if let Some(answer) = TEST_ANSWER.with(std::cell::Cell::get) {
        return confirm_with(true, || answer);
    }
    confirm_with(std::io::stderr().is_terminal(), || {
        Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()
            .ok()
    })
}

#[cfg(test)]
thread_local! {
    /// The answer [`confirm`] gives on this thread instead of asking, set by
    /// [`answering`]: `Some(Some(yes))` answers, `Some(None)` is a failed
    /// prompt, `None` asks for real.
    static TEST_ANSWER: std::cell::Cell<Option<Option<bool>>> = const { std::cell::Cell::new(None) };
}

/// Runs `f` with [`confirm`] answering `answer` (see [`TEST_ANSWER`]), so
/// the callers' yes/no paths can be tested without a terminal.
#[cfg(test)]
pub(crate) fn answering<T>(answer: Option<bool>, f: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_ANSWER.with(|a| a.set(None));
        }
    }
    TEST_ANSWER.with(|a| a.set(Some(answer)));
    let _reset = Reset;
    f()
}

/// [`confirm`] with the terminal check and the prompt passed in: `ask` runs
/// only when stderr is a terminal and returns `None` when the prompt failed.
fn confirm_with(stderr_is_terminal: bool, ask: impl FnOnce() -> Option<bool>) -> Confirmation {
    if !stderr_is_terminal {
        return Confirmation::Unavailable;
    }
    match ask() {
        Some(true) => Confirmation::Yes,
        Some(false) => Confirmation::No,
        None => Confirmation::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Without a terminal on stderr the prompt is never drawn. What fails
    /// this: asking first and checking after.
    #[test]
    fn no_terminal_never_asks() {
        let asked = std::cell::Cell::new(false);
        let outcome = confirm_with(false, || {
            asked.set(true);
            Some(true)
        });
        assert_eq!(outcome, Confirmation::Unavailable);
        assert!(!asked.get());
    }

    #[test]
    fn answers_and_failures_map_to_outcomes() {
        assert_eq!(confirm_with(true, || Some(true)), Confirmation::Yes);
        assert_eq!(confirm_with(true, || Some(false)), Confirmation::No);
        assert_eq!(confirm_with(true, || None), Confirmation::Unavailable);
    }
}
