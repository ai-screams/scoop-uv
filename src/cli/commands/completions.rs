//! Completions command

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::{Cli, ShellType};
use crate::error::Result;

/// Execute the completions command
pub fn execute(shell: ShellType) -> Result<()> {
    let mut cmd = Cli::command();

    let shell = match shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
        ShellType::Fish => Shell::Fish,
        ShellType::Powershell => Shell::PowerShell,
    };

    generate(shell, &mut cmd, "scoop", &mut std::io::stdout());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate completions for `shell` into an in-memory buffer.
    /// Helper so the per-shell tests stay one-liners.
    fn generate_for(shell: Shell) -> String {
        let mut cmd = Cli::command();
        let mut buf: Vec<u8> = Vec::new();
        generate(shell, &mut cmd, "scoop", &mut buf);
        String::from_utf8(buf).expect("completion output must be valid UTF-8")
    }

    /// Regression guard: every non-hidden subcommand in the clap tree
    /// must appear in the generated completion output for every shell.
    /// Without this, a future CLI command added to `Commands` would
    /// silently miss completion coverage if a regression somewhere
    /// stripped it from the tree clap-complete consumes.
    ///
    /// Naive substring check works because every shell that clap-
    /// complete supports embeds subcommand names verbatim somewhere
    /// in the body (bash function names, zsh case branches, fish
    /// `complete` lines, PowerShell `[CompletionResult]` entries).
    fn assert_every_subcommand_present(shell: Shell, body: &str) {
        let cmd = Cli::command();
        for sub in cmd.get_subcommands() {
            if sub.is_hide_set() {
                continue;
            }
            let name = sub.get_name();
            assert!(
                body.contains(name),
                "expected subcommand '{name}' in {shell:?} completion output \
                 (body length {} chars)",
                body.len()
            );
        }
    }

    #[test]
    fn bash_completion_includes_every_non_hidden_subcommand() {
        let body = generate_for(Shell::Bash);
        assert_every_subcommand_present(Shell::Bash, &body);
    }

    #[test]
    fn zsh_completion_includes_every_non_hidden_subcommand() {
        let body = generate_for(Shell::Zsh);
        assert_every_subcommand_present(Shell::Zsh, &body);
    }

    #[test]
    fn fish_completion_includes_every_non_hidden_subcommand() {
        let body = generate_for(Shell::Fish);
        assert_every_subcommand_present(Shell::Fish, &body);
    }

    #[test]
    fn powershell_completion_includes_every_non_hidden_subcommand() {
        let body = generate_for(Shell::PowerShell);
        assert_every_subcommand_present(Shell::PowerShell, &body);
    }

    // Note: a "hidden subcommands don't appear in completions" probe
    // was considered and rejected. clap-complete embeds every
    // subcommand name (including hidden ones) in helper-function
    // identifiers like `_scoop__subcmd__activate` — they participate
    // in the dispatch table but aren't offered as interactive
    // suggestions. Asserting their *absence* in the body needs a
    // shell-specific parser, which is out of proportion to the
    // value: man.rs::render_to_dir_skips_hidden_subcommands already
    // pins the user-facing contract for hidden commands.
}
