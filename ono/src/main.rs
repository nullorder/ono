const PITCH: &str = "Beautiful terminal UI components. Copy-paste, framework-agnostic, code you own.";
const REPO: &str = "https://github.com/nullorder/ono";

fn main() {
    println!();
    println!("  ono v{}", env!("CARGO_PKG_VERSION"));
    println!("  {}", PITCH);
    println!();
    println!("  Status: placeholder. The real CLI is in active development.");
    println!("  Follow along: {}", REPO);
    println!();
}
