//! Spec parsing, composition resolution, validation, and evaluation.
//!
//! The entry point for consumers is [`Catalog::load`]; once loaded, use the
//! helpers in [`resolve`] to walk deps, [`validate`] to check integrity, and
//! [`expr`] to evaluate pass-expressions at instantiation time.

pub mod catalog;
pub mod expr;
pub mod resolve;
pub mod types;
pub mod validate;

#[cfg(test)]
mod tests;

pub use catalog::{Catalog, CatalogError};
pub use expr::{eval_expr, eval_pass, ExprError, ParamCtx, Value};
pub use resolve::{deps, ResolveError};
pub use types::{Compose, Kind, Meta, Param, ParamType, Spec, ThemeKnobs};
pub use validate::{validate_all, ValidateError};

use crate::theme::Palette;
use ratatui::style::Color;

/// Resolve a class name (declared in `[classes]`) to a concrete color under a
/// given palette. Returns `None` if the class is unknown, or if the class maps
/// to a palette role that doesn't exist (the latter is caught earlier by
/// [`validate::validate_all`]).
pub fn class_color(spec: &Spec, palette: &Palette, class: &str) -> Option<Color> {
    let role = spec.classes.get(class)?;
    palette.role(role)
}
