//! Transitive composition resolution with cycle detection.

use super::catalog::Catalog;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("unknown component `{0}`")]
    Unknown(String),
    #[error("compose entry in `{parent}` has neither `element` nor `component`")]
    EmptyCompose { parent: String },
    #[error("cycle detected: {}", path.join(" → "))]
    Cycle { path: Vec<String> },
}

/// Transitively collect every spec referenced (directly or indirectly) by
/// `root`, in depth-first post-order with `root` appearing last. The returned
/// list is deduplicated and suitable for driving `ono add` file copies.
pub fn deps(catalog: &Catalog, root: &str) -> Result<Vec<String>, ResolveError> {
    if catalog.get(root).is_none() {
        return Err(ResolveError::Unknown(root.to_string()));
    }
    let mut out: Vec<String> = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    visit(catalog, root, &mut out, &mut stack)?;
    Ok(out)
}

fn visit(
    catalog: &Catalog,
    name: &str,
    out: &mut Vec<String>,
    stack: &mut Vec<String>,
) -> Result<(), ResolveError> {
    if out.iter().any(|n| n == name) {
        return Ok(());
    }
    if stack.iter().any(|n| n == name) {
        let mut path = stack.clone();
        path.push(name.to_string());
        return Err(ResolveError::Cycle { path });
    }
    let spec = catalog
        .get(name)
        .ok_or_else(|| ResolveError::Unknown(name.to_string()))?;

    stack.push(name.to_string());
    for entry in &spec.compose {
        let child = entry
            .child()
            .ok_or_else(|| ResolveError::EmptyCompose { parent: name.to_string() })?;
        visit(catalog, child, out, stack)?;
    }
    stack.pop();
    out.push(name.to_string());
    Ok(())
}
