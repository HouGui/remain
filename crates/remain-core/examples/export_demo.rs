//! Export a workspace from a developer smoke test, without starting the GUI.
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let source = args
        .next()
        .unwrap_or_else(|| "examples/demo-en/workspace.json".into());
    let destination = args.next().unwrap_or_else(|| "demo-report".into());
    let analysis = remain_core::Workspace::load(source)?.export(destination)?;
    println!(
        "Exported {} projects and {} observations.",
        analysis.projects.len(),
        analysis.observations.len()
    );
    Ok(())
}
