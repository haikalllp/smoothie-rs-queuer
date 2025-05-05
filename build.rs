// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("InternalName", "SMOOTHIE-QUEUER.EXE");
        res.compile()?;
    }
    Ok(())
}