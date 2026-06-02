//! Handler for the `scoop prune` command.
//!
//! Thin wrapper around `uv cache prune` so users don't need to remember the
//! exact uv invocation. The heavy lifting (size accounting, safe traversal)
//! lives in uv itself; we only forward the result.

use rust_i18n::t;
use serde::Serialize;

use crate::error::Result;
use crate::output::Output;
use crate::uv::UvClient;

#[derive(Serialize)]
struct PruneData {
    /// Raw stdout from `uv cache prune`. Useful for scripts that want to
    /// surface the freed-bytes number without re-running uv.
    output: String,
}

/// Execute the `prune` command.
pub fn execute(output: &Output) -> Result<()> {
    let uv = UvClient::new()?;
    let stdout = uv.cache_prune()?;

    if output.is_json() {
        output.json_success("prune", PruneData { output: stdout });
        return Ok(());
    }

    output.success(&t!("prune.success"));
    let trimmed = stdout.trim_end();
    if !trimmed.is_empty() {
        // Forward uv's own output verbatim — it includes the freed-bytes line
        // which is the only piece users actually care about.
        println!("{trimmed}");
    }
    Ok(())
}
