//! Individual doctor checks, one per file, registered via [`default_checks`].

mod home;
mod symlink;
mod uv;
mod virtualenv;

use super::types::Check;

/// The default set of checks, in display order. Single place to register a check.
pub(super) fn default_checks() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(uv::UvCheck),
        Box::new(home::HomeCheck),
        Box::new(virtualenv::VirtualenvCheck),
        Box::new(symlink::SymlinkCheck),
        // TEMPORARY until Task 5 — still defined in mod.rs:
        Box::new(super::ShellCheck),
        Box::new(super::VersionCheck),
        Box::new(super::LegacyCheck),
    ]
}
