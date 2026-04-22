//! Beautiful terminal UI components for Ratatui.
//!
//! Ono ships themeable, parameterized widgets that drop into an existing
//! Ratatui app like any other `Widget`. The library is the default path —
//! the `ono` CLI (`list` / `preview` / `add`) is a helper for discovery and
//! for users who want to eject the source into their own tree.
//!
//! # Quick start
//!
//! ```no_run
//! use std::time::Duration;
//! use figlet_rs::FIGlet;
//! use ono::components::splash::{Banner, Splash};
//! use ono::theme::Theme;
//! use ratatui::widgets::Widget;
//! # use ratatui::{buffer::Buffer, layout::Rect};
//! # let mut buf = Buffer::empty(Rect::new(0, 0, 80, 16));
//! # let area = buf.area;
//!
//! let theme = Theme::Forest;
//! let font = FIGlet::standard().unwrap();
//! let banner = Banner::from_text("ono", &font);
//!
//! Splash::new(&banner, theme.palette(), theme.knobs())
//!     .elapsed(Duration::from_millis(1200))
//!     .render(area, &mut buf);
//! ```
//!
//! # What's in the crate
//!
//! - [`theme`] — [`theme::Theme`], [`theme::Palette`] (9 semantic roles),
//!   [`theme::Knobs`] (animation + behavior).
//! - [`elements`] — atomic widgets: `box`, `progress`, `spinner`,
//!   `percentage`, `sparkline`, `typewriter`.
//! - [`components`] — composites: `splash`, `boot`, `dashboard`,
//!   `statusbar`, `map`.
//!
//! # Themes
//!
//! Forest is the canonical theme and the only one built by default. Enable
//! the `theme-retro`, `theme-minimal`, `theme-cyber`, or `all-themes` cargo
//! features to unlock the others. Components never branch on theme identity
//! for visual logic — they pull colors from the palette and behaviour from
//! the knobs.
//!
//! # Ejecting to source
//!
//! Run `cargo install ono` and then `ono add splash` in your project to copy
//! a component's source (plus transitive deps and a `theme.rs` you own) into
//! `./src/ono/`. Ejected code imports only `ratatui` and your own `theme.rs`
//! — no runtime dependency on this crate.
//!
//! # Semver
//!
//! The public surface covered by semver is everything under [`theme`],
//! [`elements`], and [`components`]. The CLI (`cli`) and spec engine
//! (internal) are not semver-stable.
//!
//! # Narrative docs
//!
//! Rustdoc covers the API; the repo's `docs/` directory covers the
//! conceptual side — getting started, theming guide, component catalog,
//! eject guide.
//!
//! <https://github.com/nullorder/ono/tree/main/docs>

#![warn(missing_docs)]

#[doc(hidden)]
pub mod cli;
pub mod components;
pub mod elements;
pub(crate) mod spec;
pub mod theme;
