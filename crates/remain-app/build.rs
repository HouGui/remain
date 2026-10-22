//! Embed the application icon and package version in Windows executables.

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=../../packaging/remain.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("../../packaging/remain.ico")
            .set("ProductName", "REMAIN")
            .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
            .compile()?;
    }
    Ok(())
}
