use anyhow::Result;
use leansticky_core::load_or_bootstrap;

fn main() -> Result<()> {
    let bootstrap = load_or_bootstrap()?;
    println!(
        "LeanSticky TUI skeleton: {} notes loaded. Full terminal UI is a later milestone.",
        bootstrap.notes.len()
    );
    Ok(())
}
