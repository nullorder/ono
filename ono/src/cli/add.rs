//! `ono add` — copy a component (and its transitive deps) into the user's
//! project.
//!
//! Destination layout:
//!   ./src/ono/elements/<name>.rs
//!   ./src/ono/components/<name>.rs
//!   ./src/ono/theme.rs      (generated on first add, reused after)
//!   ./src/ono/mod.rs        (touched so `pub mod elements/components/theme;`
//!                            always reflects what's on disk)
//!
//! v0.1.0 is file-copy: the source `.rs` files from this crate ship verbatim
//! into the user's tree. Codegen-from-spec is v0.2.0.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::ThemeArg;
use crate::spec::{Catalog, Kind};

pub fn run(name: &str, theme: ThemeArg) -> Result<(), AddError> {
    let catalog = Catalog::load().map_err(|e| AddError::Catalog(e.to_string()))?;

    let deps = crate::spec::deps(&catalog, name).map_err(|e| match e {
        crate::spec::ResolveError::Unknown(n) => AddError::Unknown(n),
        other => AddError::Catalog(other.to_string()),
    })?;

    let root = PathBuf::from("src/ono");
    let elements_dir = root.join("elements");
    let components_dir = root.join("components");
    fs::create_dir_all(&elements_dir)?;
    fs::create_dir_all(&components_dir)?;

    let mut copied: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for dep in &deps {
        let spec = catalog.get(dep).expect("dep resolved against catalog");
        let (subdir, source) = match spec.component.kind {
            Kind::Element => (&elements_dir, element_source(dep)?),
            Kind::Component => (&components_dir, component_source(dep)?),
        };
        let filename = format!("{}.rs", rust_module_name(dep));
        let dest = subdir.join(&filename);

        if dest.exists() {
            if !prompt_overwrite(&dest)? {
                skipped.push(format!("{}", dest.display()));
                continue;
            }
        }
        fs::write(&dest, source)?;
        copied.push(format!("{}", dest.display()));
    }

    let theme_path = root.join("theme.rs");
    if theme_path.exists() {
        eprintln!(
            "theme.rs already exists at {} — keeping existing palette. Delete it first to regenerate.",
            theme_path.display()
        );
    } else {
        fs::write(&theme_path, theme_rs_contents(theme))?;
        copied.push(format!("{}", theme_path.display()));
    }

    write_mod_rs(&elements_dir, &collect_module_names(&elements_dir)?)?;
    write_mod_rs(&components_dir, &collect_module_names(&components_dir)?)?;
    write_root_mod_rs(&root)?;

    println!("ono add {name} --theme {}", theme.as_name());
    for c in &copied {
        println!("  + {c}");
    }
    for s in &skipped {
        println!("  · kept {s}");
    }
    println!();
    println!("Next: add `mod ono;` to your crate root, then");
    println!("  use ono::components::{}::*;  // or ono::elements::<name>", name);

    Ok(())
}

fn prompt_overwrite(path: &Path) -> Result<bool, AddError> {
    eprint!("{} exists. Overwrite? [y/N] ", path.display());
    io::stderr().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(matches!(buf.trim(), "y" | "Y" | "yes"))
}

fn rust_module_name(spec_name: &str) -> &str {
    match spec_name {
        "box" => "boxed",
        other => other,
    }
}

fn element_source(name: &str) -> Result<&'static str, AddError> {
    Ok(match name {
        "box" => include_str!("../elements/boxed.rs"),
        "percentage" => include_str!("../elements/percentage.rs"),
        "progress" => include_str!("../elements/progress.rs"),
        "sparkline" => include_str!("../elements/sparkline.rs"),
        "spinner" => include_str!("../elements/spinner.rs"),
        "typewriter" => include_str!("../elements/typewriter.rs"),
        other => return Err(AddError::NoSource(other.to_string())),
    })
}

fn component_source(name: &str) -> Result<&'static str, AddError> {
    Ok(match name {
        "boot" => include_str!("../components/boot.rs"),
        "dashboard" => include_str!("../components/dashboard.rs"),
        "map" => include_str!("../components/map.rs"),
        "splash" => include_str!("../components/splash.rs"),
        "statusbar" => include_str!("../components/statusbar.rs"),
        other => return Err(AddError::NoSource(other.to_string())),
    })
}

fn collect_module_names(dir: &Path) -> io::Result<Vec<String>> {
    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some("mod") | None => continue,
            Some(s) => s.to_string(),
        };
        names.push(stem);
    }
    names.sort();
    Ok(names)
}

fn write_mod_rs(dir: &Path, modules: &[String]) -> io::Result<()> {
    let mut body = String::new();
    for m in modules {
        body.push_str(&format!("pub mod {m};\n"));
    }
    fs::write(dir.join("mod.rs"), body)
}

fn write_root_mod_rs(root: &Path) -> io::Result<()> {
    let mut mods: Vec<&'static str> = Vec::new();
    if root.join("theme.rs").exists() {
        mods.push("theme");
    }
    if root.join("elements").is_dir() {
        mods.push("elements");
    }
    if root.join("components").is_dir() {
        mods.push("components");
    }
    // Components ship with the full builder surface. A freshly vendored tree
    // won't exercise every knob on day one; allow dead code across the whole
    // `src/ono/` subtree so users don't see a wall of warnings.
    let mut body = String::from("#![allow(dead_code)]\n\n");
    for m in &mods {
        body.push_str(&format!("pub mod {m};\n"));
    }
    fs::write(root.join("mod.rs"), body)
}

fn theme_rs_contents(theme: ThemeArg) -> String {
    // Wrap the engine's theme module (which already holds every palette and
    // knob set) and re-export the requested default.
    let default = match theme {
        ThemeArg::Forest => "Forest",
        ThemeArg::Retro => "Retro",
        ThemeArg::Minimal => "Minimal",
        ThemeArg::Cyber => "Cyber",
    };
    let raw: &str = include_str!("../theme/mod.rs");
    // Strip the engine's cargo-feature gates so the vendored module compiles
    // in a plain user crate with all four themes available unconditionally.
    let body: String = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with("#[cfg(feature = \"theme-"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "//! Generated by `ono add --theme {name}`.
//!
//! Palette, Knobs, and helpers live here. You own this file — tweak colors
//! freely. The copied components read only the `Palette` and `Knobs` fields,
//! so adding your own palette roles is safe as long as you don't remove the
//! existing ones.
//!
//! All four themes ship unconditionally; delete the ones you don't want.
//! Regenerate by deleting this file and re-running `ono add <any>`.

{body}
pub const DEFAULT_THEME: Theme = Theme::{default};
",
        name = theme.as_name(),
        default = default,
        body = body,
    )
}

#[derive(Debug, thiserror::Error)]
pub enum AddError {
    #[error("unknown component `{0}`")]
    Unknown(String),
    #[error("catalog error: {0}")]
    Catalog(String),
    #[error("no source available for `{0}` (this is a packaging bug — please report)")]
    NoSource(String),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
