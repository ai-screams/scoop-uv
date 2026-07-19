//! Check for shell configuration (scuv init).

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
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => {
                return vec![CheckResult::error(
                    self.id(),
                    self.name(),
                    "could not determine home directory",
                )];
            }
        };

        // Detect current shell from $SHELL environment variable
        let shell = std::env::var("SHELL").unwrap_or_default();
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Determine config files to check based on shell
        let config_files: Vec<(&str, std::path::PathBuf)> = match shell_name.as_str() {
            "zsh" => vec![("zsh", home.join(".zshrc"))],
            "bash" => {
                // macOS uses .bash_profile, Linux uses .bashrc
                if cfg!(target_os = "macos") {
                    vec![
                        ("bash", home.join(".bash_profile")),
                        ("bash", home.join(".bashrc")),
                    ]
                } else {
                    vec![("bash", home.join(".bashrc"))]
                }
            }
            _ => {
                // Unknown shell - check both common configs
                return vec![
                    CheckResult::warn(
                        self.id(),
                        self.name(),
                        format!("unsupported shell: {}", shell_name),
                    )
                    .with_details("Supported shells: bash, zsh")
                    .with_suggestion("Manual setup may be required"),
                ];
            }
        };

        let shell_type = if shell_name == "zsh" { "zsh" } else { "bash" };

        // Check if any config file contains scuv init. A legacy-only
        // `scoop init` line does NOT count as configured: the deprecated
        // `scoop` shell function is defined *inside* `scuv init`'s own
        // output (see shell/bash.rs, shell/zsh.rs), so it only exists once
        // that init line has already run successfully in the session. An rc
        // file that still invokes `eval "$(scoop init ...)"` calls the
        // `scoop` *binary* directly — which no longer ships after upgrade —
        // so the eval fails at shell startup and integration never loads.
        // That must be flagged as a warning, not treated as configured.
        //
        // DEPRECATION(0.16.0): drop the legacy branch once the shim window
        // closes.
        for (_shell_type, config_path) in &config_files {
            if config_path.exists() {
                match std::fs::read_to_string(config_path) {
                    Ok(content) => {
                        if content.contains("scuv init") {
                            return vec![
                                CheckResult::ok(self.id(), self.name())
                                    .with_details(format!("found in {}", config_path.display())),
                            ];
                        }
                        if content.contains("scoop init") {
                            return vec![
                                CheckResult::warn(
                                    self.id(),
                                    self.name(),
                                    "shell config still references the removed `scoop` command (init line fails at startup)",
                                )
                                .with_details(format!("found in {}", config_path.display()))
                                .with_suggestion(format!(
                                    "Replace with: eval \"$(scuv init {})\"",
                                    shell_type
                                )),
                            ];
                        }
                    }
                    Err(_) => {
                        return vec![CheckResult::warn(
                            self.id(),
                            self.name(),
                            format!("could not read {}", config_path.display()),
                        )];
                    }
                }
            }
        }

        // No scuv init (or legacy scoop init) found
        let config_file = if shell_name == "zsh" {
            "~/.zshrc"
        } else if cfg!(target_os = "macos") {
            "~/.bash_profile"
        } else {
            "~/.bashrc"
        };

        vec![
            CheckResult::error(
                self.id(),
                self.name(),
                "scuv init not found in shell config",
            )
            .with_suggestion(format!(
                "Add to {}: eval \"$(scuv init {})\"",
                config_file, shell_type
            )),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serial_test::serial;

    // ==========================================================================
    // ShellCheck: scuv init vs. legacy scoop init in rc files
    //
    // A legacy-only `scoop init` line must warn, not pass — the deprecated
    // `scoop` shell function is defined by `scuv init`'s own output (see
    // shell/bash.rs), so it doesn't exist yet when an rc file's
    // `eval "$(scoop init ...)"` line runs at shell startup; that line
    // invokes the (now-removed) `scoop` binary directly and fails.
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
}
