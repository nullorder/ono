//! Serde-deserializable shapes for the TOML spec format.

use serde::Deserialize;
use std::collections::BTreeMap;

/// One full component spec — what a single `.toml` file parses into.
#[derive(Debug, Deserialize)]
pub struct Spec {
    pub component: Meta,
    #[serde(default)]
    pub params: BTreeMap<String, Param>,
    #[serde(default)]
    pub classes: BTreeMap<String, String>,
    #[serde(default)]
    pub compose: Vec<Compose>,
    /// Freeform animation knobs (e.g. splash's `pulse_left_phase`).
    #[serde(default)]
    pub animation: toml::Table,
    #[serde(default)]
    pub theme_knobs: ThemeKnobs,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub name: String,
    pub kind: Kind,
    pub description: String,
    #[serde(default)]
    pub targets: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Element,
    Component,
}

#[derive(Debug, Deserialize)]
pub struct Param {
    #[serde(rename = "type")]
    pub ty: ParamType,
    pub default: toml::Value,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub min: Option<toml::Value>,
    #[serde(default)]
    pub range: Option<[f64; 2]>,
    #[serde(default)]
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Float,
    Int,
    String,
    Bool,
    Enum,
}

/// A `[[compose]]` entry. Exactly one of `element` or `component` must be set;
/// the child is looked up by name in the catalog.
#[derive(Debug, Deserialize)]
pub struct Compose {
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub slot: Option<String>,
    /// Name of a parent param whose truthiness gates this child. When absent,
    /// the child is unconditional.
    #[serde(default)]
    pub when: Option<String>,
    /// Map of child-param-name → expression string. Expressions use `${...}`
    /// to reference parent params; bare values are literal strings.
    #[serde(default)]
    pub pass: BTreeMap<String, String>,
}

impl Compose {
    /// Name of the child being composed. Returns `None` if neither `element`
    /// nor `component` is set (invalid spec; caught by validation).
    pub fn child(&self) -> Option<&str> {
        self.element
            .as_deref()
            .or(self.component.as_deref())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ThemeKnobs {
    #[serde(default)]
    pub uses: Vec<String>,
}
