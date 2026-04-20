//! Compile-time-embedded spec catalog.
//!
//! TOML files under `ono/specs/` are baked into the binary via `include_dir!`.
//! `Catalog::load()` parses them into a name-keyed map on first call.

use super::types::{Kind, Spec};
use include_dir::{include_dir, Dir, DirEntry};
use std::collections::BTreeMap;
use thiserror::Error;

static SPECS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/specs");

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("spec `{path}`: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("spec `{path}` is not valid UTF-8")]
    NotUtf8 { path: String },
    #[error("duplicate spec name `{name}` (found in both `{a}` and `{b}`)")]
    DuplicateName { name: String, a: String, b: String },
    #[error("spec `{path}`: file name `{file}` must match component name `{name}`")]
    NameMismatch { path: String, file: String, name: String },
}

#[derive(Debug)]
pub struct Catalog {
    specs: BTreeMap<String, Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub spec: Spec,
    /// Source path inside `ono/specs/`, e.g. `"elements/progress.toml"`.
    pub source: String,
}

impl Catalog {
    /// Parse every embedded spec file. Returns an error on the first bad
    /// parse or duplicate name.
    pub fn load() -> Result<Self, CatalogError> {
        let mut specs: BTreeMap<String, Entry> = BTreeMap::new();
        let mut stack: Vec<&Dir<'_>> = vec![&SPECS_DIR];
        while let Some(dir) = stack.pop() {
            for entry in dir.entries() {
                match entry {
                    DirEntry::Dir(d) => stack.push(d),
                    DirEntry::File(file) if file.path().extension().and_then(|s| s.to_str()) == Some("toml") => {
                        load_file(file, &mut specs)?;
                    }
                    DirEntry::File(_) => {}
                }
            }
        }
        Ok(Catalog { specs })
    }

    pub fn get(&self, name: &str) -> Option<&Spec> {
        self.specs.get(name).map(|e| &e.spec)
    }

    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.specs.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Spec)> {
        self.specs.iter().map(|(k, v)| (k.as_str(), &v.spec))
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.specs.keys().map(|s| s.as_str())
    }

    pub fn elements(&self) -> impl Iterator<Item = &Spec> {
        self.specs
            .values()
            .map(|e| &e.spec)
            .filter(|s| s.component.kind == Kind::Element)
    }

    pub fn components(&self) -> impl Iterator<Item = &Spec> {
        self.specs
            .values()
            .map(|e| &e.spec)
            .filter(|s| s.component.kind == Kind::Component)
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

fn load_file(
    file: &include_dir::File<'_>,
    specs: &mut BTreeMap<String, Entry>,
) -> Result<(), CatalogError> {
    let path = file.path().to_string_lossy().into_owned();
    let content = file
        .contents_utf8()
        .ok_or_else(|| CatalogError::NotUtf8 { path: path.clone() })?;
    let spec: Spec = toml::from_str(content)
        .map_err(|e| CatalogError::Parse { path: path.clone(), source: e })?;

    let stem = file
        .path()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if stem != spec.component.name {
        return Err(CatalogError::NameMismatch {
            path: path.clone(),
            file: stem.to_string(),
            name: spec.component.name.clone(),
        });
    }

    let name = spec.component.name.clone();
    if let Some(existing) = specs.get(&name) {
        return Err(CatalogError::DuplicateName {
            name,
            a: existing.source.clone(),
            b: path,
        });
    }
    specs.insert(name, Entry { spec, source: path });
    Ok(())
}
