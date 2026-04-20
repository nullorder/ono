use clap::{Parser, Subcommand, ValueEnum};

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
    eprintln!("ono list: not implemented yet (workstream E)");
}

fn cmd_preview(name: &str, theme: ThemeArg) {
    eprintln!("ono preview {name} --theme {theme:?}: not implemented yet (workstream E)");
}

fn cmd_add(name: &str, theme: ThemeArg) {
    eprintln!("ono add {name} --theme {theme:?}: not implemented yet (workstream E)");
}
