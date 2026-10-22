//! Regenerate synthetic examples in a new destination directory.
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let destination = args.next().unwrap_or_else(|| "examples/demo-en".into());
    let locale = args.next().unwrap_or_else(|| "en".into());
    let path = match args.next().as_deref() {
        None => remain_core::demo::create(destination, &locale)?,
        Some("--classes") => remain_core::demo::create_classified(destination, &locale)?,
        Some(option) => anyhow::bail!("Unknown option {option:?}; expected --classes"),
    };
    println!("{}", path.display());
    Ok(())
}
