//! CLI integration tests
//!
//! These tests verify the CLI behavior using `assert_cmd`.
//! Note: Tests that require Python installation (create, activate) are marked as `#[ignore]`
//! to allow running in CI environments without uv/Python installed.

// `Command::cargo_bin()` is deprecated since assert_cmd 2.1.0 due to
// incompatibility with custom cargo build directories. The recommended
// replacement is `escargot` crate for more flexible binary building.
// For now, we allow deprecated usage as it works correctly for standard
// cargo layouts. See: https://docs.rs/assert_cmd/latest/assert_cmd/cargo/
// TODO: Consider migrating to escargot if custom build-dir support is needed.
#![allow(deprecated)]

mod support;

mod color;
mod dispatch;
mod errors;
mod general;
mod list;
mod output_format;
mod remove;
mod requires_uv;
mod shell;
mod uninstall;
