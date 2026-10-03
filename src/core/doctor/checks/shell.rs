//! Check for shell configuration (scuv init).

use std::path::{Path, PathBuf};

use super::super::types::{Check, CheckResult};

/// Check for shell configuration (scuv init).
pub(super) struct ShellCheck;

impl Check for ShellCheck {
    fn id(&self) -> &'static str {
        "shell"
    }

    fn name(&self) -> &'static str {
        "shell configuration"
    }

    fn run(&self) -> Vec<CheckResult> {
        let Some(home) = dirs::home_dir() else {
            return vec![CheckResult::error(
                self.id(),
                self.name(),
                "could not determine home directory",
            )];
        };

        let shell_name = current_shell_name();
        let Some(rc_files) = rc_files(&shell_name, &home) else {
            return vec![
                CheckResult::warn(
                    self.id(),
                    self.name(),
                    format!("unsupported shell: {}", shell_name),
                )
                .with_details("Supported shells: bash, zsh")
                .with_suggestion("Manual setup may be required"),
            ];
        };
        let shell_type = if shell_name == "zsh" { "zsh" } else { "bash" };

        if let Some(found) = rc_files
            .iter()
            .find_map(|rc| self.inspect_rc_file(rc, shell_type))
        {
            return vec![found];
        }

        vec![
            CheckResult::error(
                self.id(),
                self.name(),
                "scuv init not found in shell config",
            )
            .with_suggestion(format!(
                "Add to {}: eval \"$(scuv init {})\"",
                default_rc_file(&shell_name),
                shell_type
            )),
        ]
    }
}

impl ShellCheck {
    /// The verdict for one rc file, or `None` when it is absent or has no
    /// init line (keep looking).
    ///
    /// A stale `scoop init` line does NOT count as configured: `scoop` is
    /// not a command any more (renamed in 0.15.0, the transitional shell
    /// forwarder went in 0.16.0), so `eval "$(scoop init ...)"` fails at
    /// shell startup and integration never loads. That must be flagged as a
    /// warning, not treated as configured. This is a diagnostic for an old
    /// rc line, not a compatibility shim, so it stays.
    fn inspect_rc_file(&self, rc: &Path, shell_type: &str) -> Option<CheckResult> {
        if !rc.exists() {
            return None;
        }
        let Ok(content) = std::fs::read_to_string(rc) else {
            return Some(CheckResult::warn(
                self.id(),
                self.name(),
                format!("could not read {}", rc.display()),
            ));
        };
        if content.contains("scuv init") {
            return Some(
                CheckResult::ok(self.id(), self.name())
                    .with_details(format!("found in {}", rc.display())),
            );
        }
        if content.contains("scoop init") {
            return Some(
                CheckResult::warn(
                    self.id(),
                    self.name(),
                    "shell config still references the removed `scoop` command (init line fails at startup)",
                )
                .with_details(format!("found in {}", rc.display()))
                .with_suggestion(format!(
                    "Replace with: eval \"$(scuv init {})\"",
                    shell_type
                )),
            );
        }
        None
    }
}

/// The current shell's name from `$SHELL` (`/bin/zsh` → `zsh`), lowercased.
fn current_shell_name() -> String {
    let shell = std::env::var("SHELL").unwrap_or_default();
    Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// The rc files to look in for `shell_name`, in order; `None` for a shell
/// this check does not know. macOS bash reads `.bash_profile`, Linux bash
/// `.bashrc`.
fn rc_files(shell_name: &str, home: &Path) -> Option<Vec<PathBuf>> {
    match shell_name {
        "zsh" => Some(vec![home.join(".zshrc")]),
        "bash" if cfg!(target_os = "macos") => {
            Some(vec![home.join(".bash_profile"), home.join(".bashrc")])
        }
        "bash" => Some(vec![home.join(".bashrc")]),
        _ => None,
    }
}

/// Where the suggestion tells the user to add the init line.
fn default_rc_file(shell_name: &str) -> &'static str {
    if shell_name == "zsh" {
        "~/.zshrc"
    } else if cfg!(target_os = "macos") {
        "~/.bash_profile"
    } else {
        "~/.bashrc"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serial_test::serial;

    // ==========================================================================
    // ShellCheck: scuv init vs. a stale scoop init line in rc files
    //
    // A stale `scoop init` line must warn, not pass — `scoop` is not a
    // command any more, so an rc file's `eval "$(scoop init ...)"` fails at
    // shell startup.
    // ==========================================================================

    /// Write `content` to both `.bash_profile` and `.bashrc` so the test is
    /// deterministic regardless of which one `ShellCheck` consults on the
    /// host OS (macOS checks `.bash_profile` first, Linux only `.bashrc`).
    fn write_bash_rc(home: &std::path::Path, content: &str) {
        std::fs::write(home.join(".bash_profile"), content).unwrap();
        std::fs::write(home.join(".bashrc"), content).unwrap();
    }

    #[test]
    #[serial]
    fn shell_check_is_ok_when_current_scuv_init_line_present() {
        let home_tmp = tempfile::tempdir().unwrap();
        write_bash_rc(home_tmp.path(), "eval \"$(scuv init bash)\"\n");
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/bash")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_ok(), "expected Ok, got {:#?}", results[0]);
        assert_eq!(results[0].id, "shell");
        assert_eq!(results[0].name, "shell configuration");
    }

    /// Pins the exact branch order in `ShellCheck::run`: a `scuv init` match
    /// must short-circuit before the legacy-only warning branch is even
    /// reached, so an rc file that still has an old, now-inert comment or
    /// leftover `scoop init` reference alongside a working `scuv init` line
    /// is not incorrectly flagged.
    #[test]
    #[serial]
    fn shell_check_ok_when_both_scuv_and_legacy_scoop_init_present() {
        let home_tmp = tempfile::tempdir().unwrap();
        write_bash_rc(
            home_tmp.path(),
            "# old: eval \"$(scoop init bash)\"\neval \"$(scuv init bash)\"\n",
        );
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/bash")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_ok(), "expected Ok, got {:#?}", results[0]);
    }

    #[test]
    #[serial]
    fn shell_check_warns_on_legacy_only_scoop_init_line() {
        let home_tmp = tempfile::tempdir().unwrap();
        write_bash_rc(home_tmp.path(), "eval \"$(scoop init bash)\"\n");
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/bash")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(
            results[0].is_warning(),
            "expected Warning, got {:#?}",
            results[0]
        );
        let suggestion = results[0].suggestion.as_deref().unwrap_or_default();
        assert!(
            suggestion.contains("scuv init"),
            "suggestion should point at scuv init, got: {suggestion}"
        );
    }

    // ==========================================================================
    // ShellCheck zsh branch (the `shell_name == "zsh"` boundary): the
    // suggestion must name `scuv init zsh`, not bash, and read .zshrc.
    // ==========================================================================

    #[test]
    #[serial]
    fn shell_check_zsh_warns_on_legacy_only_line_with_zsh_suggestion() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            home_tmp.path().join(".zshrc"),
            "eval \"$(scoop init zsh)\"\n",
        )
        .unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/zsh")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_warning(), "got {results:#?}");
        let suggestion = results[0].suggestion.as_deref().unwrap_or_default();
        assert!(
            suggestion.contains("scuv init zsh"),
            "zsh shell must get a zsh suggestion, got {suggestion:?}"
        );
    }

    #[test]
    #[serial]
    fn shell_check_zsh_no_init_errors_with_zshrc_suggestion() {
        // Empty `.zshrc` (no scuv/scoop init) reaches the "no init found" tail,
        // which selects the config-file hint by shell name. A `== -> !=` mutant
        // on the `shell_name == "zsh"` boundary would pick the bash file here.
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::write(home_tmp.path().join(".zshrc"), "").unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/zsh")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        // no init line present -> error path selects config file by shell name.
        // A `== -> !=` mutant would pick the bash file for zsh.
        assert!(results[0].is_error());
        assert!(
            results[0]
                .suggestion
                .as_deref()
                .unwrap_or("")
                .contains(".zshrc"),
            "zsh no-init suggestion must name ~/.zshrc, got {:?}",
            results[0].suggestion
        );
    }

    #[test]
    #[serial]
    fn shell_check_zsh_is_ok_with_current_scuv_init_line() {
        let home_tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            home_tmp.path().join(".zshrc"),
            "eval \"$(scuv init zsh)\"\n",
        )
        .unwrap();
        let _g = crate::test_utils::env_guard(&[
            ("SHELL", Some("/bin/zsh")),
            ("HOME", Some(home_tmp.path().to_str().unwrap())),
        ]);

        let results = ShellCheck.run();
        assert_eq!(results.len(), 1, "got {results:#?}");
        assert!(results[0].is_ok(), "got {results:#?}");
    }

    /// Which rc files are read per shell and platform. Fails if a shell
    /// arm or the macOS split is dropped (the macOS-only variants are only
    /// distinguishable on the other platform, which is where CI runs).
    #[test]
    fn rc_files_per_shell_and_platform() {
        let home = Path::new("/h");
        assert_eq!(rc_files("zsh", home), Some(vec![home.join(".zshrc")]));
        let bash = rc_files("bash", home).expect("bash is supported");
        if cfg!(target_os = "macos") {
            assert_eq!(bash, vec![home.join(".bash_profile"), home.join(".bashrc")]);
        } else {
            assert_eq!(bash, vec![home.join(".bashrc")]);
        }
        assert_eq!(rc_files("fish", home), None);
    }
}
