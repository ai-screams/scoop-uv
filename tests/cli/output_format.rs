//! Output format assertions (real output, not snapshots).

use crate::support::*;

#[test]
fn test_version_output_format() {
    let output = Command::cargo_bin("scuv")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Version format should be "scuv X.Y.Z"
    assert!(
        stdout.starts_with("scuv "),
        "Version should start with 'scuv '"
    );

    // Verify semver format (X.Y.Z)
    let version_part = stdout.trim().strip_prefix("scuv ").unwrap();
    let parts: Vec<&str> = version_part.split('.').collect();
    assert_eq!(parts.len(), 3, "Version should be semver format X.Y.Z");
    for (i, part) in parts.iter().enumerate() {
        assert!(
            part.chars().all(|c| c.is_ascii_digit()),
            "Version part {} ('{}') should be numeric",
            i,
            part
        );
    }
}

#[test]
fn test_help_has_required_sections() {
    let output = Command::cargo_bin("scuv")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Help must have these sections
    assert!(stdout.contains("Usage:"), "Help missing 'Usage:' section");
    assert!(
        stdout.contains("Commands:"),
        "Help missing 'Commands:' section"
    );
    assert!(
        stdout.contains("Options:"),
        "Help missing 'Options:' section"
    );
}

#[test]
fn test_help_lists_all_subcommands() {
    let output = Command::cargo_bin("scuv")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // User-facing subcommands that should be visible in help
    // Note: activate/deactivate are hidden (shell wrapper handles them)
    let visible_subcommands = [
        "list", "create", "remove", "use", "install", "init", "shell",
    ];

    for cmd in visible_subcommands {
        assert!(
            stdout.contains(cmd),
            "Help missing required subcommand: {}",
            cmd
        );
    }
}

#[test]
fn test_error_message_is_helpful() {
    let fixture = TestFixture::new();

    let output = scoop_cmd(&fixture.scoop_home)
        .args(["activate", "nonexistent"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Error messages should be helpful
    assert!(
        stderr.to_lowercase().contains("error") || !output.status.success(),
        "Failed command should indicate error"
    );
    assert!(
        stderr.contains("nonexistent"),
        "Error should mention the problematic env name"
    );
    assert!(
        stderr.contains("Can't find"),
        "Error should explain the problem"
    );
}
