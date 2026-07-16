//! Check for uv installation.

use std::process::Command;

use crate::uv::version as uv_version;

use super::super::types::{Check, CheckResult};

/// Check for uv installation.
pub(super) struct UvCheck;

impl UvCheck {
    /// Platform-appropriate install or upgrade command for uv.
    fn install_hint() -> &'static str {
        if cfg!(target_os = "macos") {
            "brew install uv  OR  curl -LsSf https://astral.sh/uv/install.sh | sh"
        } else if cfg!(target_os = "windows") {
            "powershell -ExecutionPolicy ByPass -c \"irm https://astral.sh/uv/install.ps1 | iex\""
        } else {
            "curl -LsSf https://astral.sh/uv/install.sh | sh"
        }
    }
}

impl Check for UvCheck {
    fn id(&self) -> &'static str {
        "uv"
    }

    fn name(&self) -> &'static str {
        "uv installation"
    }

    fn run(&self) -> Vec<CheckResult> {
        match Command::new("uv").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let raw = String::from_utf8_lossy(&output.stdout);
                let raw = raw.trim();

                // Enforce the minimum supported uv version when parseable.
                // Unparseable output (custom build, unknown format) is treated
                // as a soft pass so we don't block users on a banner change.
                if let Some(version) = uv_version::parse(raw) {
                    if !uv_version::meets_minimum(version) {
                        return vec![
                            CheckResult::error(
                                self.id(),
                                self.name(),
                                format!(
                                    "uv {} is older than the supported minimum ({})",
                                    uv_version::format_version(version),
                                    uv_version::format_version(uv_version::MIN_VERSION),
                                ),
                            )
                            .with_details(raw.to_string())
                            .with_suggestion(format!("Upgrade uv: {}", Self::install_hint())),
                        ];
                    }
                }

                vec![CheckResult::ok(self.id(), self.name()).with_details(raw.to_string())]
            }
            _ => {
                vec![
                    CheckResult::error(self.id(), self.name(), "uv not found in PATH")
                        .with_details("scuv requires uv to manage Python environments")
                        .with_suggestion(format!("Install uv: {}", Self::install_hint())),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_check_trait_dispatch_always_reports_under_uv_id() {
        // Environment-tolerant: passes whether or not a usable `uv` is on
        // PATH — what it pins is that run() actually runs and every result
        // carries this check's identity.
        let check: &dyn Check = &UvCheck;
        assert_eq!(check.id(), "uv");
        assert_eq!(check.name(), "uv installation");
        let results = check.run();
        assert!(!results.is_empty(), "run() must produce results");
        assert!(
            results.iter().all(|r| r.id == "uv"),
            "all results must carry the uv id, got {results:#?}"
        );
    }

    #[test]
    fn install_hint_names_uv() {
        // A `-> ""` / `-> "xyzzy"` mutant would drop the real install command.
        let hint = UvCheck::install_hint();
        assert!(
            hint.contains("uv"),
            "install hint must mention uv, got {hint:?}"
        );
        assert!(
            hint.contains("astral") || hint.contains("brew"),
            "install hint must reference a real installer, got {hint:?}"
        );
    }
}
