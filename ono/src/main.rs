const PITCH: &str = "Beautiful terminal UI components. Copy-paste, framework-agnostic, code you own.";
const REPO: &str = "https://github.com/nullorder/ono";

fn main() {
    println!();
    println!("  ono v{}", env!("CARGO_PKG_VERSION"));
    println!("  {}", PITCH);
    println!();
    println!("  Status: placeholder. The real CLI ships in v0.1.0.");
    println!("  Follow along: {}", REPO);
    println!();
}
