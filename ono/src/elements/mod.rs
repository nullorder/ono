//! Hand-written Ratatui source for atomic elements.
//!
//! Each module is self-contained: the only non-std import it makes is
//! `ratatui` and `super::super::theme::Palette`. When `ono add` copies one of
//! these files into a user's project, that `super::super::theme` path still
//! resolves — to the `theme.rs` the CLI emits alongside.
//!
//! Naming note: the `box` element's Rust module is named `boxed` because
//! `box` is a reserved Rust keyword. The TOML spec and `ono list` entry
//! remain "box" — that's the user-facing name.

pub mod boxed;
pub mod percentage;
pub mod progress;
pub mod sparkline;
pub mod spinner;
pub mod typewriter;
