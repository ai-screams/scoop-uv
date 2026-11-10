//! Unit tests for `validate`.

use super::*;
use rstest::rstest;

#[rstest]
#[case::simple("myenv", true)]
#[case::mixed_case("MyEnv", true)]
#[case::hyphen("my-env", true)]
#[case::underscore("my_env", true)]
#[case::trailing_digits("env123", true)]
#[case::single_char("a", true)]
#[case::empty("", false)]
#[case::digit_start("123", false)]
#[case::version_like("3.12", false)]
#[case::hyphen_start("-env", false)]
#[case::underscore_start("_env", false)]
#[case::space("my env", false)]
#[case::dot("my.env", false)]
#[case::reserved_activate("activate", false)]
#[case::reserved_activate_upper("ACTIVATE", false)]
#[case::reserved_list("list", false)]
#[case::reserved_version("version", false)]
fn is_valid_env_name_cases(#[case] input: &str, #[case] expected: bool) {
    assert_eq!(is_valid_env_name(input), expected);
}

#[rstest]
#[case::major_only("3", true)]
#[case::major_minor("3.12", true)]
#[case::major_minor_patch("3.12.0", true)]
#[case::alpha_suffix("3.12.0a1", true)]
#[case::empty("", false)]
#[case::non_numeric("abc", false)]
#[case::v_prefix("v3.12", false)]
#[case::hyphen_beta("3.12-beta", false)]
fn is_valid_python_version_cases(#[case] input: &str, #[case] expected: bool) {
    assert_eq!(is_valid_python_version(input), expected);
}

#[test]
fn test_validate_python_version() {
    assert!(validate_python_version("3.12").is_ok());
    assert!(validate_python_version("3.12.0").is_ok());
    assert!(validate_python_version("latest").is_ok());
    assert!(validate_python_version("LATEST").is_ok());
    assert!(validate_python_version("stable").is_ok());
    assert!(validate_python_version("STABLE").is_ok());
    assert!(validate_python_version("").is_err());
}

/// Every top-level command and alias is reserved, so `scuv <word>` can
/// never be read as an env name. Hidden commands count too: the shell
/// wrappers call them. What fails this: a new subcommand or alias that
/// is not added to RESERVED_NAMES.
#[test]
fn reserved_names_cover_every_top_level_command() {
    use clap::CommandFactory;

    let mut cmd = crate::cli::Cli::command();
    cmd.build();
    let mut seen = Vec::new();
    for sub in cmd.get_subcommands() {
        seen.push(sub.get_name().to_string());
        seen.extend(sub.get_all_aliases().map(str::to_string));
    }
    for expected in ["help", "activate", "ls", "rm", "delete", "self"] {
        assert!(seen.iter().any(|s| s == expected), "{expected} not seen");
    }
    let missing: Vec<_> = seen
        .iter()
        .filter(|s| !RESERVED_NAMES.contains(&s.as_str()))
        .collect();
    assert!(missing.is_empty(), "not in RESERVED_NAMES: {missing:?}");
}

/// A reserved word only stops a new env from taking the name; an env
/// that already has it can still be found, used and removed.
#[test]
fn reserved_words_pass_the_existing_name_check() {
    for name in RESERVED_NAMES {
        assert!(validate_existing_env_name(name).is_ok(), "{name}");
        assert!(is_valid_existing_env_name(&name.to_uppercase()), "{name}");
        assert!(validate_env_name(name).is_err(), "{name}");
    }
}

/// The existing-name check keeps every rule but the reserved list, so
/// it still refuses anything that could leave the virtualenvs dir.
#[rstest]
#[case("")]
#[case("../x")]
#[case("a/b")]
#[case("a\\b")]
#[case(".")]
#[case("..")]
#[case("3.12")]
#[case("-x")]
#[case("my env")]
fn existing_name_check_rejects_unsafe_names(#[case] name: &str) {
    assert!(validate_existing_env_name(name).is_err(), "{name:?}");
    assert!(!is_valid_existing_env_name(name), "{name:?}");
}

#[test]
fn existing_name_check_enforces_the_length_limit() {
    assert!(is_valid_existing_env_name(&"a".repeat(MAX_ENV_NAME_LENGTH)));
    assert!(!is_valid_existing_env_name(
        &"a".repeat(MAX_ENV_NAME_LENGTH + 1)
    ));
}

#[test]
fn test_validate_env_name_errors() {
    let err = validate_env_name("").unwrap_err();
    assert!(err.to_string().contains("empty"));

    let err = validate_env_name("activate").unwrap_err();
    assert!(err.to_string().contains("reserved"));

    let err = validate_env_name("3.12").unwrap_err();
    assert!(err.to_string().contains("version"));

    let err = validate_env_name("-invalid").unwrap_err();
    assert!(err.to_string().contains("must start with a letter"));
}

#[test]
fn test_env_name_max_length() {
    let long_name = "a".repeat(64);
    assert!(is_valid_env_name(&long_name));

    let too_long = "a".repeat(65);
    assert!(!is_valid_env_name(&too_long));
}

// ==========================================================================
// Boundary Value Tests
// ==========================================================================

#[test]
fn test_env_name_boundary_values() {
    // Minimum valid length (1 character)
    assert!(is_valid_env_name("a"));
    assert!(is_valid_env_name("Z"));

    // Just below max length (63 characters)
    let just_under = "a".repeat(63);
    assert!(is_valid_env_name(&just_under));

    // Exactly at max length (64 characters)
    let at_max = "a".repeat(64);
    assert!(is_valid_env_name(&at_max));

    // Just above max length (65 characters)
    let just_over = "a".repeat(65);
    assert!(!is_valid_env_name(&just_over));

    // Way over max length
    let way_over = "a".repeat(1000);
    assert!(!is_valid_env_name(&way_over));
}

#[test]
fn test_env_name_whitespace_handling() {
    // Whitespace-only names must be rejected
    assert!(!is_valid_env_name(" "));
    assert!(!is_valid_env_name("  "));
    assert!(!is_valid_env_name("\t"));
    assert!(!is_valid_env_name("\n"));
    assert!(!is_valid_env_name(" \t\n "));

    // Names with leading/trailing whitespace
    assert!(!is_valid_env_name(" myenv"));
    assert!(!is_valid_env_name("myenv "));
    assert!(!is_valid_env_name(" myenv "));
}

#[test]
fn test_python_version_boundary_values() {
    // Single digit major version
    assert!(is_valid_python_version("3"));
    assert!(is_valid_python_version("2"));

    // Two digit major version (future-proofing)
    assert!(is_valid_python_version("12"));
    assert!(is_valid_python_version("99"));

    // Very long version string
    assert!(is_valid_python_version("3.12.0"));
    assert!(is_valid_python_version("3.12.999"));

    // Pre-release versions
    assert!(is_valid_python_version("3.12.0a1"));
    assert!(is_valid_python_version("3.12.0b99"));
    assert!(is_valid_python_version("3.12.0rc1"));
}

#[test]
fn test_reserved_names_case_insensitivity() {
    // All reserved names should be rejected regardless of case
    for reserved in ["activate", "deactivate", "list", "create", "remove", "use"] {
        assert!(
            !is_valid_env_name(reserved),
            "lowercase '{}' should be rejected",
            reserved
        );
        assert!(
            !is_valid_env_name(&reserved.to_uppercase()),
            "uppercase '{}' should be rejected",
            reserved
        );

        // Mixed case
        let mixed: String = reserved
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_uppercase().next().unwrap()
                } else {
                    c
                }
            })
            .collect();
        assert!(
            !is_valid_env_name(&mixed),
            "mixed case '{}' should be rejected",
            mixed
        );
    }
}

// ==========================================================================
// Security Tests: Path Traversal and Injection Prevention
// ==========================================================================

#[test]
fn test_env_name_path_traversal_rejected() {
    // Path traversal attempts must be rejected
    assert!(!is_valid_env_name("../"));
    assert!(!is_valid_env_name(".."));
    assert!(!is_valid_env_name("../etc"));
    assert!(!is_valid_env_name("../../../etc/passwd"));
    assert!(!is_valid_env_name("foo/../bar"));
    assert!(!is_valid_env_name("foo/.."));
}

#[test]
fn test_env_name_hidden_file_rejected() {
    // Hidden file/directory names (starting with .) must be rejected
    assert!(!is_valid_env_name(".hidden"));
    assert!(!is_valid_env_name("."));
    assert!(!is_valid_env_name(".."));
    assert!(!is_valid_env_name(".gitignore"));
    assert!(!is_valid_env_name(".env"));
}

#[test]
fn test_env_name_absolute_path_rejected() {
    // Absolute paths must be rejected
    assert!(!is_valid_env_name("/etc/passwd"));
    assert!(!is_valid_env_name("/tmp/evil"));
    assert!(!is_valid_env_name("/"));
}

#[test]
fn test_env_name_special_chars_rejected() {
    // Special shell characters must be rejected
    assert!(!is_valid_env_name("env;ls"));
    assert!(!is_valid_env_name("env|cat"));
    assert!(!is_valid_env_name("env&bg"));
    assert!(!is_valid_env_name("env`cmd`"));
    assert!(!is_valid_env_name("env$(cmd)"));
    assert!(!is_valid_env_name("env>file"));
    assert!(!is_valid_env_name("env<file"));
    assert!(!is_valid_env_name("env\nwhoami"));
    assert!(!is_valid_env_name("env\twhoami"));
}

#[test]
fn test_env_name_null_byte_rejected() {
    // Null bytes must be rejected (C string terminator attack)
    assert!(!is_valid_env_name("env\0hidden"));
    assert!(!is_valid_env_name("\0"));
}

#[test]
fn test_env_name_unicode_rejected() {
    // Unicode homoglyphs and non-ASCII must be rejected
    assert!(!is_valid_env_name("еnv")); // Cyrillic 'е' (U+0435) looks like 'e'
    assert!(!is_valid_env_name("환경")); // Korean
    assert!(!is_valid_env_name("環境")); // Chinese
    assert!(!is_valid_env_name("env™"));
    assert!(!is_valid_env_name("env🐍")); // Emoji
}

#[test]
fn test_validate_env_name_path_traversal_error_message() {
    // Verify error message is helpful for security-related rejections
    let result = validate_env_name("../etc");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("must start with a letter"));
}

// ==========================================================================
// Boundary / branch coverage (kills mutation-testing gaps)
// ==========================================================================

/// `validate_env_name` length check: exactly MAX is allowed, MAX+1 rejected.
#[test]
fn validate_env_name_length_boundary() {
    assert!(validate_env_name(&"a".repeat(MAX_ENV_NAME_LENGTH)).is_ok());
    let too_long = validate_env_name(&"a".repeat(MAX_ENV_NAME_LENGTH + 1));
    assert!(too_long.is_err());
    assert!(too_long.unwrap_err().to_string().contains("maximum length"));
}

/// The digit/dot fallback accepts version-shaped strings the regex rejects
/// (e.g. a trailing dot). Pins the `||` fallback branch.
#[test]
fn is_valid_python_version_digit_dot_fallback() {
    assert!(!VERSION_REGEX.is_match("3."), "precondition: regex rejects");
    assert!(is_valid_python_version("3."));
    assert!(is_valid_python_version("3.12.4.5"));
}

#[test]
fn validate_python_path_rejects_nonexistent() {
    let err = validate_python_path(Path::new("/no/such/python")).unwrap_err();
    assert!(err.to_string().contains("file not found"));
}

#[test]
fn validate_python_path_rejects_directory() {
    let dir = tempfile::tempdir().unwrap();
    let err = validate_python_path(dir.path()).unwrap_err();
    assert!(err.to_string().contains("not a file"));
}

#[cfg(unix)]
#[test]
fn validate_python_path_rejects_non_executable_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("python");
    std::fs::write(&file, "").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    let err = validate_python_path(&file).unwrap_err();
    assert!(err.to_string().contains("not executable"));
}

#[cfg(unix)]
#[test]
fn validate_python_path_accepts_executable_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("python");
    std::fs::write(&file, "").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(validate_python_path(&file).is_ok());
}

// `detect_python_version` spawns the interpreter; its parsing logic is
// covered deterministically here via the pure `parse_python_version_output`
// (no subprocess, no temp-script exec — avoids noexec/ETXTBSY flakiness in
// containers). The thin spawning wrapper is excluded from cargo-mutants.

#[test]
fn parse_python_version_output_stdout() {
    // Python 3: version on stdout, trailing newline trimmed.
    assert_eq!(
        parse_python_version_output("Python 3.12.7\n", ""),
        Some("3.12.7".to_string())
    );
}

#[test]
fn parse_python_version_output_stderr_fallback() {
    // Python 2 prints to stderr; stdout empty.
    assert_eq!(
        parse_python_version_output("", "Python 2.7.18\n"),
        Some("2.7.18".to_string())
    );
}

#[test]
fn parse_python_version_output_none_on_non_python() {
    assert_eq!(parse_python_version_output("bash 5.2\n", ""), None);
    assert_eq!(parse_python_version_output("", ""), None);
}
