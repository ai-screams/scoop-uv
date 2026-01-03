//! Deactivate command

use crate::error::Result;

/// Execute the deactivate command
/// Outputs shell script to be eval'd
pub fn execute() -> Result<()> {
    // Output deactivation script for eval
    println!(
        r#"if [ -n "$VIRTUAL_ENV" ]; then
    PATH="${{PATH#$VIRTUAL_ENV/bin:}}"
    export PATH
    unset VIRTUAL_ENV
    unset UVENV_ACTIVE
fi"#
    );

    Ok(())
}
