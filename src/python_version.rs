//! Parsed Python version numbers: comparing what a user asked for with what
//! uv lists. Depends on nothing else in the crate, so `uv`, `core` and `cli`
//! can all use it without a cycle.

/// Parsed Python version components.
///
/// # Examples
///
/// ```
/// use scoop_uv::python_version::PythonVersion;
///
/// let v = PythonVersion::parse("3.12.0").unwrap();
/// assert_eq!(v.major, 3);
/// assert_eq!(v.minor, Some(12));
/// assert_eq!(v.patch, Some(0));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonVersion {
    /// Major version number (e.g., 3 in "3.12.0")
    pub major: u32,
    /// Minor version number (e.g., 12 in "3.12.0")
    pub minor: Option<u32>,
    /// Patch version number (e.g., 0 in "3.12.0")
    pub patch: Option<u32>,
    /// Pre-release suffix (e.g., "a1", "rc1")
    pub suffix: Option<String>,
}

impl PythonVersion {
    /// Parse a version string into components.
    ///
    /// # Examples
    ///
    /// ```
    /// use scoop_uv::python_version::PythonVersion;
    ///
    /// // Parse major.minor.patch
    /// let v = PythonVersion::parse("3.12.0").unwrap();
    /// assert_eq!(v.major, 3);
    /// assert_eq!(v.minor, Some(12));
    ///
    /// // Parse with pre-release suffix (suffix in minor position)
    /// let v = PythonVersion::parse("3.13a1").unwrap();
    /// assert_eq!(v.suffix, Some("a1".to_string()));
    ///
    /// // Returns None for invalid versions
    /// assert!(PythonVersion::parse("").is_none());
    /// assert!(PythonVersion::parse("latest").is_none());
    /// ```
    pub fn parse(version: &str) -> Option<Self> {
        let trimmed = version.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Handle special aliases
        if matches!(trimmed.to_lowercase().as_str(), "latest" | "stable") {
            return None;
        }

        // Split by dots and parse
        let mut parts = trimmed.split('.');
        let major_str = parts.next()?;
        let major: u32 = major_str.parse().ok()?;

        let minor = parts.next().and_then(|s| {
            // Minor might have suffix like "12a1"
            let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        });

        let patch = parts.next().and_then(|s| {
            let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        });

        // Extract suffix (e.g., "a1", "b2", "rc1"). char_indices, not
        // chars().position(): slicing needs a BYTE offset, and a char index
        // diverges from it (and can panic) once any multi-byte character
        // precedes the suffix.
        let suffix = trimmed.split('.').nth(1).and_then(|minor_str| {
            minor_str
                .char_indices()
                .find(|(_, c)| c.is_ascii_alphabetic())
                .map(|(pos, _)| minor_str[pos..].to_string())
        });

        Some(Self {
            major,
            minor,
            patch,
            suffix,
        })
    }

    /// Check if this version matches another (for version resolution)
    /// e.g., "3.12" matches "3.12.0", "3.12.1", etc.
    pub fn matches(&self, other: &Self) -> bool {
        if self.major != other.major {
            return false;
        }

        if let Some(minor) = self.minor
            && other.minor != Some(minor)
        {
            return false;
        }

        if let Some(patch) = self.patch
            && other.patch != Some(patch)
        {
            return false;
        }

        true
    }
}

impl std::fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }
        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }
        if let Some(ref suffix) = self.suffix {
            write!(f, "{suffix}")?;
        }
        Ok(())
    }
}

impl Ord for PythonVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // Compare major version
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Compare minor version (None treated as 0)
        match self.minor.unwrap_or(0).cmp(&other.minor.unwrap_or(0)) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Compare patch version (None treated as 0)
        match self.patch.unwrap_or(0).cmp(&other.patch.unwrap_or(0)) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Compare suffix: None (stable) > Some (pre-release)
        // e.g., 3.12.0 > 3.12.0rc1 > 3.12.0b1 > 3.12.0a1
        match (&self.suffix, &other.suffix) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater, // stable > pre-release
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for PythonVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptests;
