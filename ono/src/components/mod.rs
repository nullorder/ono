//! Hand-written Ratatui source for composite components.
//!
//! Each module is self-contained the same way `elements/` modules are:
//! the only non-std imports are `ratatui`, sibling element/component
//! modules, and `super::super::theme`. When `ono add` copies a component,
//! all its transitive deps (elements and other components) are copied
//! too, so the relative `super::super::elements::*` paths still resolve.

pub mod boot;
pub mod dashboard;
pub mod map;
pub mod splash;
pub mod statusbar;
