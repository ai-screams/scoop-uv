//! JSON output types for CLI commands

use serde::Serialize;

/// Success response wrapper
#[derive(Serialize)]
pub struct JsonResponse<T: Serialize> {
    /// Response status (always "success")
    pub status: &'static str,
    /// Command that was executed
    pub command: &'static str,
    /// Response data
    pub data: T,
}

impl<T: Serialize> JsonResponse<T> {
    /// Create a new success response
    pub fn success(command: &'static str, data: T) -> Self {
        Self {
            status: "success",
            command,
            data,
        }
    }
}

/// Error response wrapper
#[derive(Serialize)]
pub struct JsonErrorResponse {
    /// Response status (always "error")
    pub status: &'static str,
    /// Command that was executed
    pub command: &'static str,
    /// Error details
    pub error: JsonError,
}

/// Error details
#[derive(Serialize)]
pub struct JsonError {
    /// Error code (e.g., "ENV_NOT_FOUND")
    pub code: &'static str,
    /// Human-readable error message
    pub message: String,
    /// Suggested fix (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl JsonErrorResponse {
    /// Create a new error response
    pub fn error(command: &'static str, code: &'static str, message: String) -> Self {
        Self {
            status: "error",
            command,
            error: JsonError {
                code,
                message,
                suggestion: None,
            },
        }
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.error.suggestion = Some(suggestion);
        self
    }
}

// ============================================================================
// Command-specific data types
// ============================================================================

/// List virtualenvs response data
#[derive(Serialize)]
pub struct ListEnvsData {
    pub virtualenvs: Vec<VirtualenvInfo>,
    pub total: usize,
}

/// Virtualenv info for JSON output
#[derive(Serialize)]
pub struct VirtualenvInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    pub path: String,
    pub active: bool,
}

/// List pythons response data
#[derive(Serialize)]
pub struct ListPythonsData {
    pub pythons: Vec<PythonInfo>,
    pub total: usize,
}

/// Python info for JSON output
#[derive(Serialize)]
pub struct PythonInfo {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Create response data
#[derive(Serialize)]
pub struct CreateData {
    pub name: String,
    pub python: String,
    pub path: String,
}

/// Use response data
#[derive(Serialize)]
pub struct UseData {
    pub name: String,
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink: Option<String>,
}

/// Remove response data
#[derive(Serialize)]
pub struct RemoveData {
    pub name: String,
    pub path: String,
}

/// Install response data
#[derive(Serialize)]
pub struct InstallData {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Uninstall response data
#[derive(Serialize)]
pub struct UninstallData {
    pub version: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // JsonResponse Tests
    // ========================================

    #[test]
    fn test_json_response_success_creates_correct_status() {
        let response = JsonResponse::success("test", "data");
        assert_eq!(response.status, "success");
        assert_eq!(response.command, "test");
        assert_eq!(response.data, "data");
    }

    #[test]
    fn test_json_response_serialization_format() {
        let response = JsonResponse::success(
            "list",
            ListEnvsData {
                virtualenvs: vec![],
                total: 0,
            },
        );
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""status":"success""#));
        assert!(json.contains(r#""command":"list""#));
        assert!(json.contains(r#""total":0"#));
    }

    #[test]
    fn test_json_response_with_string_data() {
        let response = JsonResponse::success("echo", "hello world");
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["data"], "hello world");
    }

    #[test]
    fn test_json_response_with_struct_data() {
        let response = JsonResponse::success(
            "create",
            CreateData {
                name: "myenv".into(),
                python: "3.12".into(),
                path: "/path/to/env".into(),
            },
        );
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["data"]["name"], "myenv");
        assert_eq!(parsed["data"]["python"], "3.12");
    }

    #[test]
    fn test_json_response_with_vec_data() {
        let response = JsonResponse::success("list", vec!["env1", "env2", "env3"]);
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["data"].as_array().unwrap().len(), 3);
    }

    // ========================================
    // JsonErrorResponse Tests
    // ========================================

    #[test]
    fn test_json_error_response_creates_error_status() {
        let response = JsonErrorResponse::error("test", "ERR_CODE", "error message".into());
        assert_eq!(response.status, "error");
        assert_eq!(response.command, "test");
        assert_eq!(response.error.code, "ERR_CODE");
        assert_eq!(response.error.message, "error message");
        assert!(response.error.suggestion.is_none());
    }

    #[test]
    fn test_json_error_response_with_suggestion() {
        let response = JsonErrorResponse::error("test", "ERR", "msg".into())
            .with_suggestion("try this".into());
        assert_eq!(response.error.suggestion, Some("try this".into()));
    }

    #[test]
    fn test_json_error_without_suggestion_omits_field() {
        let response = JsonErrorResponse::error("test", "ERR_CODE", "message".into());
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("suggestion"));
    }

    #[test]
    fn test_json_error_with_suggestion_includes_field() {
        let response = JsonErrorResponse::error("test", "ERR_CODE", "msg".into())
            .with_suggestion("try this".into());
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""suggestion":"try this""#));
    }

    #[test]
    fn test_json_error_response_serialization() {
        let response = JsonErrorResponse::error("create", "ENV_EXISTS", "already exists".into())
            .with_suggestion("use --force".into());
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["status"], "error");
        assert_eq!(parsed["command"], "create");
        assert_eq!(parsed["error"]["code"], "ENV_EXISTS");
        assert_eq!(parsed["error"]["message"], "already exists");
        assert_eq!(parsed["error"]["suggestion"], "use --force");
    }

    // ========================================
    // ListEnvsData / VirtualenvInfo Tests
    // ========================================

    #[test]
    fn test_list_envs_data_empty() {
        let data = ListEnvsData {
            virtualenvs: vec![],
            total: 0,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["virtualenvs"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["total"], 0);
    }

    #[test]
    fn test_list_envs_data_with_envs() {
        let data = ListEnvsData {
            virtualenvs: vec![
                VirtualenvInfo {
                    name: "env1".into(),
                    python: Some("3.12".into()),
                    path: "/path1".into(),
                    active: true,
                },
                VirtualenvInfo {
                    name: "env2".into(),
                    python: None,
                    path: "/path2".into(),
                    active: false,
                },
            ],
            total: 2,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["virtualenvs"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["total"], 2);
    }

    #[test]
    fn test_virtualenv_info_without_python_omits_field() {
        let info = VirtualenvInfo {
            name: "test".into(),
            python: None,
            path: "/path".into(),
            active: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.contains("python"));
    }

    #[test]
    fn test_virtualenv_info_with_python() {
        let info = VirtualenvInfo {
            name: "test".into(),
            python: Some("3.11".into()),
            path: "/path".into(),
            active: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains(r#""python":"3.11""#));
        assert!(json.contains(r#""active":true"#));
    }

    // ========================================
    // ListPythonsData / PythonInfo Tests
    // ========================================

    #[test]
    fn test_list_pythons_data_empty() {
        let data = ListPythonsData {
            pythons: vec![],
            total: 0,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["pythons"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_list_pythons_data_with_pythons() {
        let data = ListPythonsData {
            pythons: vec![
                PythonInfo {
                    version: "3.12.0".into(),
                    path: Some("/usr/bin/python3.12".into()),
                },
                PythonInfo {
                    version: "3.11.0".into(),
                    path: None,
                },
            ],
            total: 2,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["pythons"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_python_info_without_path_omits_field() {
        let info = PythonInfo {
            version: "3.12".into(),
            path: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.contains("path"));
    }

    #[test]
    fn test_python_info_with_path() {
        let info = PythonInfo {
            version: "3.12".into(),
            path: Some("/usr/bin/python3".into()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains(r#""path":"/usr/bin/python3""#));
    }

    // ========================================
    // CreateData Tests
    // ========================================

    #[test]
    fn test_create_data_serialization() {
        let data = CreateData {
            name: "myenv".into(),
            python: "3.12".into(),
            path: "/home/user/.scoop/virtualenvs/myenv".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "myenv");
        assert_eq!(parsed["python"], "3.12");
        assert!(parsed["path"].as_str().unwrap().contains("myenv"));
    }

    // ========================================
    // UseData Tests
    // ========================================

    #[test]
    fn test_use_data_local_mode() {
        let data = UseData {
            name: "myenv".into(),
            mode: "local",
            version_file: Some("/project/.scoop-version".into()),
            symlink: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["mode"], "local");
        assert!(parsed["version_file"].is_string());
        assert!(parsed.get("symlink").is_none());
    }

    #[test]
    fn test_use_data_global_mode() {
        let data = UseData {
            name: "myenv".into(),
            mode: "global",
            version_file: None,
            symlink: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(data.mode, "global");
        assert!(!json.contains("version_file"));
        assert!(!json.contains("symlink"));
    }

    #[test]
    fn test_use_data_with_symlink() {
        let data = UseData {
            name: "myenv".into(),
            mode: "local",
            version_file: Some("/project/.scoop-version".into()),
            symlink: Some("/project/.venv".into()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""symlink":"/project/.venv""#));
    }

    // ========================================
    // RemoveData Tests
    // ========================================

    #[test]
    fn test_remove_data_serialization() {
        let data = RemoveData {
            name: "oldenv".into(),
            path: "/home/user/.scoop/virtualenvs/oldenv".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "oldenv");
        assert!(parsed["path"].as_str().unwrap().contains("oldenv"));
    }

    // ========================================
    // InstallData Tests
    // ========================================

    #[test]
    fn test_install_data_with_path() {
        let data = InstallData {
            version: "3.12.0".into(),
            path: Some("/usr/local/bin/python3.12".into()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""path""#));
    }

    #[test]
    fn test_install_data_without_path() {
        let data = InstallData {
            version: "3.12.0".into(),
            path: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(!json.contains("path"));
    }

    // ========================================
    // UninstallData Tests
    // ========================================

    #[test]
    fn test_uninstall_data_serialization() {
        let data = UninstallData {
            version: "3.11.0".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "3.11.0");
    }

    // ========================================
    // Edge Cases
    // ========================================

    #[test]
    fn test_json_with_unicode_characters() {
        let info = VirtualenvInfo {
            name: "한글환경".into(),
            python: Some("3.12".into()),
            path: "/경로/테스트".into(),
            active: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        // Should serialize correctly
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "한글환경");
        assert_eq!(parsed["path"], "/경로/테스트");
    }

    #[test]
    fn test_json_with_empty_strings() {
        let data = CreateData {
            name: "".into(),
            python: "".into(),
            path: "".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "");
        assert_eq!(parsed["python"], "");
    }

    #[test]
    fn test_json_with_special_characters() {
        let data = CreateData {
            name: r#"test"env"#.into(),
            python: "3.12".into(),
            path: r#"/path/with\backslash"#.into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        // JSON escaping should handle special chars
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], r#"test"env"#);
        assert_eq!(parsed["path"], r#"/path/with\backslash"#);
    }

    #[test]
    fn test_json_with_long_strings() {
        let long_name = "a".repeat(1000);
        let data = CreateData {
            name: long_name.clone(),
            python: "3.12".into(),
            path: "/path".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"].as_str().unwrap().len(), 1000);
    }

    #[test]
    fn test_json_path_with_spaces() {
        let data = CreateData {
            name: "myenv".into(),
            python: "3.12".into(),
            path: "/path/with spaces/to/env".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["path"], "/path/with spaces/to/env");
    }

    #[test]
    fn test_json_with_newlines_and_tabs() {
        let data = JsonErrorResponse::error("test", "ERR", "line1\nline2\ttab".into());
        let json = serde_json::to_string(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["error"]["message"], "line1\nline2\ttab");
    }

    #[test]
    fn test_json_roundtrip_integrity() {
        let original = ListEnvsData {
            virtualenvs: vec![
                VirtualenvInfo {
                    name: "env1".into(),
                    python: Some("3.12".into()),
                    path: "/path/to/env1".into(),
                    active: true,
                },
                VirtualenvInfo {
                    name: "env2".into(),
                    python: None,
                    path: "/path/to/env2".into(),
                    active: false,
                },
            ],
            total: 2,
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify structure integrity
        assert_eq!(parsed["virtualenvs"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["virtualenvs"][0]["name"], "env1");
        assert_eq!(parsed["virtualenvs"][0]["active"], true);
        assert_eq!(parsed["virtualenvs"][1]["name"], "env2");
        assert!(parsed["virtualenvs"][1].get("python").is_none());
    }
}
