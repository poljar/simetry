#[cfg(windows)]
fn main() {
    use simetry::iracing::commands;
    commands::pit::fuel(31)
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
