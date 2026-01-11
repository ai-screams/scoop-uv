//! Package extraction from source environments

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, ScoopError};

/// A package specification extracted from an environment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpec {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Whether this is an editable install
    pub editable: bool,
    /// Path for editable installs
    pub editable_path: Option<PathBuf>,
}

impl PackageSpec {
    /// Creates a requirements.txt format string.
    pub fn to_requirement(&self) -> String {
        if self.editable {
            if let Some(path) = &self.editable_path {
                return format!("-e {}", path.display());
            }
        }
        format!("{}=={}", self.name, self.version)
    }
}

/// Result of package extraction
#[derive(Debug)]
pub struct ExtractionResult {
    /// Successfully extracted packages
    pub packages: Vec<PackageSpec>,
    /// Packages that failed to parse
    pub failed: Vec<String>,
    /// Total count of lines processed
    pub total_found: usize,
}

impl ExtractionResult {
    /// Generates requirements.txt content from extracted packages.
    pub fn to_requirements(&self) -> String {
        self.packages
            .iter()
            .map(|p| p.to_requirement())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns only non-editable packages.
    pub fn regular_packages(&self) -> Vec<&PackageSpec> {
        self.packages.iter().filter(|p| !p.editable).collect()
    }

    /// Returns only editable packages.
    pub fn editable_packages(&self) -> Vec<&PackageSpec> {
        self.packages.iter().filter(|p| p.editable).collect()
    }
}

/// Extracts package information from source environments
#[derive(Debug, Default)]
pub struct PackageExtractor {
    /// Whether to include editable packages
    include_editable: bool,
}

impl PackageExtractor {
    /// Creates a new package extractor.
    pub fn new() -> Self {
        Self {
            include_editable: true,
        }
    }

    /// Sets whether to include editable packages.
    pub fn include_editable(mut self, include: bool) -> Self {
        self.include_editable = include;
        self
    }

    /// Extracts packages from the given environment path using pip freeze.
    ///
    /// # Errors
    ///
    /// Returns [`ScoopError::PackageExtractionFailed`] if pip freeze fails.
    pub fn extract(&self, env_path: &Path) -> Result<ExtractionResult> {
        let pip_path = env_path.join("bin").join("pip");

        if !pip_path.exists() {
            return Err(ScoopError::PackageExtractionFailed {
                reason: format!("pip not found at {}", pip_path.display()),
            });
        }

        // Run pip freeze
        let output = Command::new(&pip_path)
            .arg("freeze")
            .output()
            .map_err(|e| ScoopError::PackageExtractionFailed {
                reason: format!("Failed to run pip freeze: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScoopError::PackageExtractionFailed {
                reason: format!("pip freeze failed: {}", stderr),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_freeze_output(&stdout)
    }

    /// Parses pip freeze output into package specs.
    fn parse_freeze_output(&self, output: &str) -> Result<ExtractionResult> {
        let mut packages = Vec::new();
        let mut failed = Vec::new();
        let mut total_found = 0;

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            total_found += 1;

            match self.parse_line(line) {
                Some(spec) => {
                    // Skip editable if not included
                    if spec.editable && !self.include_editable {
                        continue;
                    }
                    packages.push(spec);
                }
                None => {
                    failed.push(line.to_string());
                }
            }
        }

        Ok(ExtractionResult {
            packages,
            failed,
            total_found,
        })
    }

    /// Parses a single line from pip freeze output.
    fn parse_line(&self, line: &str) -> Option<PackageSpec> {
        // Handle editable installs: -e git+https://... or -e /path/to/package
        if line.starts_with("-e ") {
            return self.parse_editable(line);
        }

        // Handle standard format: package==version
        if let Some((name, version)) = line.split_once("==") {
            return Some(PackageSpec {
                name: name.trim().to_string(),
                version: version.trim().to_string(),
                editable: false,
                editable_path: None,
            });
        }

        // Handle @ format: package @ file:///path or package @ https://...
        if let Some((name, rest)) = line.split_once(" @ ") {
            // Try to extract version from the rest if it looks like a direct reference
            let version = if rest.starts_with("file://") {
                "local".to_string()
            } else {
                "unknown".to_string()
            };
            return Some(PackageSpec {
                name: name.trim().to_string(),
                version,
                editable: false,
                editable_path: None,
            });
        }

        None
    }

    /// Parses an editable install line.
    fn parse_editable(&self, line: &str) -> Option<PackageSpec> {
        let rest = line.strip_prefix("-e ")?.trim();

        // git+https://github.com/user/repo.git@version#egg=name
        if rest.starts_with("git+") {
            // Extract package name from #egg=name
            if let Some(egg_idx) = rest.find("#egg=") {
                let name = &rest[egg_idx + 5..];
                // Try to extract version from @tag
                let version = if let Some(at_idx) = rest.find('@') {
                    let end = rest.find('#').unwrap_or(rest.len());
                    rest[at_idx + 1..end].to_string()
                } else {
                    "git".to_string()
                };
                return Some(PackageSpec {
                    name: name.to_string(),
                    version,
                    editable: true,
                    editable_path: Some(PathBuf::from(rest)),
                });
            }
        }

        // Local path: -e /path/to/package or -e .
        let path = PathBuf::from(rest);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Some(PackageSpec {
            name,
            version: "editable".to_string(),
            editable: true,
            editable_path: Some(path),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_package() {
        let extractor = PackageExtractor::new();
        let result = extractor.parse_freeze_output("requests==2.31.0").unwrap();

        assert_eq!(result.packages.len(), 1);
        assert_eq!(result.packages[0].name, "requests");
        assert_eq!(result.packages[0].version, "2.31.0");
        assert!(!result.packages[0].editable);
    }

    #[test]
    fn test_parse_multiple_packages() {
        let extractor = PackageExtractor::new();
        let output = "requests==2.31.0\nflask==3.0.0\nnumpy==1.26.0";
        let result = extractor.parse_freeze_output(output).unwrap();

        assert_eq!(result.packages.len(), 3);
        assert_eq!(result.total_found, 3);
        assert!(result.failed.is_empty());
    }

    #[test]
    fn test_parse_editable_local() {
        let extractor = PackageExtractor::new();
        let result = extractor
            .parse_freeze_output("-e /home/user/mypackage")
            .unwrap();

        assert_eq!(result.packages.len(), 1);
        assert!(result.packages[0].editable);
        assert_eq!(result.packages[0].name, "mypackage");
    }

    #[test]
    fn test_parse_editable_git() {
        let extractor = PackageExtractor::new();
        let result = extractor
            .parse_freeze_output("-e git+https://github.com/user/repo.git@v1.0.0#egg=myrepo")
            .unwrap();

        assert_eq!(result.packages.len(), 1);
        assert!(result.packages[0].editable);
        assert_eq!(result.packages[0].name, "myrepo");
        assert_eq!(result.packages[0].version, "v1.0.0");
    }

    #[test]
    fn test_skip_comments_and_empty() {
        let extractor = PackageExtractor::new();
        let output = "# comment\n\nrequests==2.31.0\n   \n# another comment";
        let result = extractor.parse_freeze_output(output).unwrap();

        assert_eq!(result.packages.len(), 1);
        assert_eq!(result.total_found, 1);
    }

    #[test]
    fn test_exclude_editable() {
        let extractor = PackageExtractor::new().include_editable(false);
        let output = "requests==2.31.0\n-e /home/user/mypackage";
        let result = extractor.parse_freeze_output(output).unwrap();

        assert_eq!(result.packages.len(), 1);
        assert_eq!(result.packages[0].name, "requests");
    }

    #[test]
    fn test_to_requirements() {
        let result = ExtractionResult {
            packages: vec![
                PackageSpec {
                    name: "requests".to_string(),
                    version: "2.31.0".to_string(),
                    editable: false,
                    editable_path: None,
                },
                PackageSpec {
                    name: "flask".to_string(),
                    version: "3.0.0".to_string(),
                    editable: false,
                    editable_path: None,
                },
            ],
            failed: vec![],
            total_found: 2,
        };

        let requirements = result.to_requirements();
        assert!(requirements.contains("requests==2.31.0"));
        assert!(requirements.contains("flask==3.0.0"));
    }
}
