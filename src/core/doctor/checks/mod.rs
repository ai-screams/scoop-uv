//! Individual doctor checks, one per file, registered via [`default_checks`].

mod home;
mod legacy;
mod shell;
mod symlink;
mod uv;
mod version;
mod virtualenv;

#[cfg(test)]
mod test_support;

use super::types::Check;

/// The default set of checks, in display order. Single place to register a check.
pub(super) fn default_checks() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(uv::UvCheck),
        Box::new(home::HomeCheck),
        Box::new(virtualenv::VirtualenvCheck),
        Box::new(symlink::SymlinkCheck),
        Box::new(shell::ShellCheck),
        Box::new(version::VersionCheck),
        Box::new(legacy::LegacyCheck),
    ]
}
