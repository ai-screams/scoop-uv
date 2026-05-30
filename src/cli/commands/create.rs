//! Create command

use std::path::Path;

use rust_i18n::t;

use crate::core::VirtualenvService;
use crate::error::Result;
use crate::output::{CreateData, Output};
use crate::paths;
use crate::validate;

/// Execute the create command
pub fn execute(
    output: &Output,
    name: &str,
    python: &str,
    python_path: Option<&Path>,
    force: bool,
    install_python: bool,
) -> Result<()> {
    let service = VirtualenvService::auto()?;

    // Check if exists and handle force
    if service.exists(name)? {
        if force {
            output.info(&t!("create.removing_existing", name = name));
            service.delete(name)?;
        } else {
            return Err(crate::error::ScoopError::VirtualenvExists {
                name: name.to_string(),
            });
        }
    }

    if let Some(pp) = python_path {
        // --python-path mode: validate, canonicalize, detect version, create
        validate::validate_python_path(pp)?;

        let canonical =
            std::fs::canonicalize(pp).map_err(|_| crate::error::ScoopError::InvalidPythonPath {
                path: pp.to_path_buf(),
                reason: "could not resolve path".to_string(),
            })?;

        // Detect Python version from the binary
        let detected_version =
            validate::detect_python_version(&canonical).unwrap_or_else(|| "unknown".to_string());

        output.info(&t!(
            "create.creating_with_path",
            name = name,
            path = canonical.display()
        ));

        let env_path = service.create_with_python_path(name, &detected_version, &canonical)?;

        // JSON output
        if output.is_json() {
            output.json_success(
                "create",
                CreateData {
                    name: name.to_string(),
                    python: detected_version,
                    path: env_path.display().to_string(),
                    python_path: Some(canonical.display().to_string()),
                },
            );
            return Ok(());
        }

        output.success(&t!("create.success", name = name));
        output.info(&t!("create.path", path = paths::abbreviate_home(&env_path)));
        output.info(&t!("create.activate_hint", name = name));
    } else {
        // Standard version-based mode

        // Lazy install: opt-in. If the requested version isn't installed yet,
        // ask uv to fetch it before handing off to venv creation. Without the
        // flag, an unavailable version still surfaces as the usual uv error,
        // so default behaviour is unchanged.
        if install_python && !service.is_python_installed(python)? {
            output.info(&t!("create.installing_python", version = python));
            service.install_python(python)?;
        }

        output.info(&t!("create.creating", name = name, python = python));

        let path = service.create(name, python)?;

        // JSON output
        if output.is_json() {
            output.json_success(
                "create",
                CreateData {
                    name: name.to_string(),
                    python: python.to_string(),
                    path: path.display().to_string(),
                    python_path: None,
                },
            );
            return Ok(());
        }

        output.success(&t!("create.success", name = name));
        output.info(&t!("create.path", path = paths::abbreviate_home(&path)));
        output.info(&t!("create.activate_hint", name = name));
    }

    Ok(())
}
