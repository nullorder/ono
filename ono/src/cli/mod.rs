use clap::{Parser, Subcommand, ValueEnum};

use crate::spec::{Catalog, Kind};
use crate::theme::Theme;

mod add;
mod preview;

const PITCH: &str = "Beautiful terminal UI components. Copy-paste, framework-agnostic, code you own.";

#[derive(Parser, Debug)]
#[command(name = "ono", version, about = PITCH, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List available elements and components
    List,
    /// Preview a component live in the terminal
    Preview {
        /// Component or element name
        name: String,
        /// Theme to render with
        #[arg(long, value_enum, default_value_t = ThemeArg::Forest)]
        theme: ThemeArg,
    },
    /// Copy a component (and its deps) into ./src/ono/
    Add {
        /// Component or element name
        name: String,
        /// Theme baked into theme.rs on first add
        #[arg(long, value_enum, default_value_t = ThemeArg::Forest)]
        theme: ThemeArg,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ThemeArg {
    Forest,
    Retro,
    Minimal,
    Cyber,
}

impl ThemeArg {
    pub fn as_name(self) -> &'static str {
        match self {
            ThemeArg::Forest => "forest",
            ThemeArg::Retro => "retro",
            ThemeArg::Minimal => "minimal",
            ThemeArg::Cyber => "cyber",
        }
    }

    /// Resolve to a concrete [`Theme`]. Non-forest themes are gated behind
    /// cargo features; if the requested theme wasn't compiled in, fall back
    /// to forest after warning the user.
    pub fn resolve(self) -> Theme {
        match self {
            ThemeArg::Forest => Theme::Forest,
            #[cfg(feature = "theme-retro")]
            ThemeArg::Retro => Theme::Retro,
            #[cfg(feature = "theme-minimal")]
            ThemeArg::Minimal => Theme::Minimal,
            #[cfg(feature = "theme-cyber")]
            ThemeArg::Cyber => Theme::Cyber,
            #[cfg(not(feature = "theme-retro"))]
            ThemeArg::Retro => fallback("retro"),
            #[cfg(not(feature = "theme-minimal"))]
            ThemeArg::Minimal => fallback("minimal"),
            #[cfg(not(feature = "theme-cyber"))]
            ThemeArg::Cyber => fallback("cyber"),
        }
    }
}

#[allow(dead_code)]
fn fallback(name: &str) -> Theme {
    eprintln!(
        "theme {name:?} not enabled in this build — rebuild with --features theme-{name}. \
         Falling back to forest."
    );
    Theme::Forest
}

pub fn run() {
    let cli = Cli::parse();
    match cli.command {
        None => print_banner(),
        Some(Command::List) => cmd_list(),
        Some(Command::Preview { name, theme }) => cmd_preview(&name, theme),
        Some(Command::Add { name, theme }) => cmd_add(&name, theme),
    }
}

fn print_banner() {
    println!();
    println!("  ono v{}", env!("CARGO_PKG_VERSION"));
    println!("  {}", PITCH);
    println!();
    println!("  Try: ono list | ono preview <name> | ono add <name>");
    println!();
}

fn cmd_list() {
    let catalog = match Catalog::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load catalog: {e}");
            std::process::exit(1);
        }
    };

    let name_width = catalog
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);

    print_group("ELEMENTS", &catalog, Kind::Element, name_width);
    println!();
    print_group("COMPONENTS", &catalog, Kind::Component, name_width);
}

fn print_group(header: &str, catalog: &Catalog, kind: Kind, name_width: usize) {
    println!("{header}");
    let mut entries: Vec<(&str, &str)> = catalog
        .iter()
        .filter(|(_, spec)| spec.component.kind == kind)
        .map(|(name, spec)| (name, first_sentence(&spec.component.description)))
        .collect();
    entries.sort_by_key(|(name, _)| *name);
    for (name, desc) in entries {
        println!("  {name:<width$}  {desc}", name = name, width = name_width, desc = desc);
    }
}

fn first_sentence(s: &str) -> &str {
    match s.find(". ") {
        Some(i) => &s[..i + 1],
        None => s.trim_end_matches('.'),
    }
}

fn cmd_preview(name: &str, theme: ThemeArg) {
    let catalog = match Catalog::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load catalog: {e}");
            std::process::exit(1);
        }
    };
    if catalog.get(name).is_none() {
        eprintln!("unknown component `{name}`. Try `ono list`.");
        std::process::exit(1);
    }
    if let Err(e) = preview::run(name, theme.resolve()) {
        eprintln!("preview failed: {e}");
        std::process::exit(1);
    }
}

fn cmd_add(name: &str, theme: ThemeArg) {
    if let Err(e) = add::run(name, theme) {
        eprintln!("add failed: {e}");
        std::process::exit(1);
    }
}
