//! Build script: track non-Rust inputs that proc-macros read at expansion time.

fn main() {
    // rust_i18n's `i18n!()` reads locales/app.yml during macro expansion,
    // but cargo doesn't know about that dependency — without this line a
    // yml-only edit reuses the stale compiled strings and `cargo test`
    // reports false-green.
    println!("cargo:rerun-if-changed=locales/app.yml");
}
