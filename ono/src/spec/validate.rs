//! Static validation over a loaded `Catalog`.
//!
//! Checks:
//! * every class value is a real palette role (`theme::PALETTE_ROLES`)
//! * every `when` names a real parent param
//! * every `pass` key names a real child param
//! * every identifier inside a `pass` expression names a real parent param
//! * every compose entry resolves to a known catalog name

use super::catalog::Catalog;
use super::expr;
use super::types::{Kind, Spec};
use crate::theme::PALETTE_ROLES;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("`{spec}`: class `{class}` maps to unknown palette role `{role}`")]
    UnknownRole { spec: String, class: String, role: String },
    #[error("`{spec}`: `when = \"{name}\"` references unknown param")]
    WhenUnknownParam { spec: String, name: String },
    #[error("`{parent}`: compose entry references unknown child `{child}`")]
    UnknownChild { parent: String, child: String },
    #[error("`{parent}`: compose entry has neither `element` nor `component`")]
    EmptyCompose { parent: String },
    #[error("`{parent}`: pass to `{child}` uses unknown child param `{key}`")]
    PassUnknownChildParam { parent: String, child: String, key: String },
    #[error("`{parent}`: pass expression for `{key}` references unknown parent param `{ident}`")]
    PassUnknownParentParam { parent: String, key: String, ident: String },
    #[error("`{parent}`: elements cannot compose (found [[compose]] in element `{parent}`)")]
    ElementComposes { parent: String },
}

/// Validate every spec in the catalog. Returns all errors discovered rather
/// than stopping at the first.
pub fn validate_all(catalog: &Catalog) -> Vec<ValidateError> {
    let mut errors = vec![];
    for (name, spec) in catalog.iter() {
        validate_one(catalog, name, spec, &mut errors);
    }
    errors
}

fn validate_one(catalog: &Catalog, name: &str, spec: &Spec, errors: &mut Vec<ValidateError>) {
    // classes → palette roles
    for (class, role) in &spec.classes {
        if !PALETTE_ROLES.contains(&role.as_str()) {
            errors.push(ValidateError::UnknownRole {
                spec: name.to_string(),
                class: class.clone(),
                role: role.clone(),
            });
        }
    }

    if spec.component.kind == Kind::Element && !spec.compose.is_empty() {
        errors.push(ValidateError::ElementComposes { parent: name.to_string() });
    }

    for compose in &spec.compose {
        let Some(child_name) = compose.child() else {
            errors.push(ValidateError::EmptyCompose { parent: name.to_string() });
            continue;
        };
        let Some(child_spec) = catalog.get(child_name) else {
            errors.push(ValidateError::UnknownChild {
                parent: name.to_string(),
                child: child_name.to_string(),
            });
            continue;
        };

        if let Some(w) = &compose.when {
            if !spec.params.contains_key(w) {
                errors.push(ValidateError::WhenUnknownParam {
                    spec: name.to_string(),
                    name: w.clone(),
                });
            }
        }

        for (key, value) in &compose.pass {
            if !child_spec.params.contains_key(key) {
                errors.push(ValidateError::PassUnknownChildParam {
                    parent: name.to_string(),
                    child: child_name.to_string(),
                    key: key.clone(),
                });
            }
            for ident in expr::idents_in_pass(value) {
                if !spec.params.contains_key(&ident) {
                    errors.push(ValidateError::PassUnknownParentParam {
                        parent: name.to_string(),
                        key: key.clone(),
                        ident,
                    });
                }
            }
        }
    }
}
