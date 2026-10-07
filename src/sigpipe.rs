//! SIGPIPE disposition: default for the process, ignored where a sudden
//! stop would skip cleanup.
//!
//! The Rust runtime ignores SIGPIPE, so a write to a pipe whose reader has
//! gone fails with EPIPE and `println!` panics (exit 101, a crash report on
//! stderr). `main` restores the default action, which ends the process at
//! that write, silently, as it ends `cat`. Every shell wrapper reads scuv's
//! output to EOF (`$(...)`, fish `| source`, PowerShell `| Out-String`), so
//! none of them closes the pipe early.
//!
//! A signal stops every thread at once, though. Where parallel workers hold
//! cleanup guards (`migrate all` rolling back half-built envs), one worker's
//! progress line into a closed pipe would stop the others mid-migration;
//! [`ignore`] puts back the runtime's behaviour for that stretch, so the
//! failure is a panic that unwinds through those guards.

/// Restores the default SIGPIPE action. Call first thing in `main`.
#[cfg(unix)]
pub fn restore_default() {
    // SAFETY: only sets a signal disposition to its default.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

/// Restores the default SIGPIPE action. Windows has no SIGPIPE.
#[cfg(not(unix))]
pub fn restore_default() {}

/// Ignores SIGPIPE until the returned guard drops, then puts back the
/// previous action.
#[must_use = "SIGPIPE is ignored only while the guard lives"]
pub fn ignore() -> IgnoreGuard {
    #[cfg(unix)]
    {
        // SAFETY: only sets a signal disposition; the previous one is kept
        // for the guard to put back.
        let previous = unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
        IgnoreGuard { previous }
    }
    #[cfg(not(unix))]
    IgnoreGuard {}
}

/// Puts back the SIGPIPE action [`ignore`] replaced.
pub struct IgnoreGuard {
    #[cfg(unix)]
    previous: libc::sighandler_t,
}

impl Drop for IgnoreGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        // SAFETY: restores the disposition `ignore` read from the kernel.
        unsafe {
            libc::signal(libc::SIGPIPE, self.previous);
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    /// The current SIGPIPE action, read without changing it.
    fn current() -> libc::sighandler_t {
        // SAFETY: a zeroed sigaction is valid output space; a null new
        // action makes sigaction(2) a pure read.
        unsafe {
            let mut old: libc::sigaction = std::mem::zeroed();
            assert_eq!(
                libc::sigaction(libc::SIGPIPE, std::ptr::null(), &mut old),
                0
            );
            old.sa_sigaction
        }
    }

    /// Fails if `ignore` stops ignoring, or its guard stops restoring.
    #[test]
    #[serial_test::serial]
    fn ignore_guard_ignores_then_restores_the_default() {
        restore_default();
        assert_eq!(current(), libc::SIG_DFL);
        {
            let _guard = ignore();
            assert_eq!(current(), libc::SIG_IGN);
        }
        assert_eq!(current(), libc::SIG_DFL);
        // Leave the test process as the test harness had it.
        // SAFETY: as in `ignore`.
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_IGN);
        }
    }
}
