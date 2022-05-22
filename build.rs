#[cfg(windows)] extern crate winres;

fn main() -> std::io::Result<()> {
    if cfg!(windows) {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico")
            .set_output_directory(".")
            .set("InternalName", "BATTLESHIPS.EXE")
            .set("ProductName", "CROD BattleShips")
            .set("CompanyName", "CROD")
            .set("FileDescription", "CROD BattleShips")
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000)
            .set_version_info(winres::VersionInfo::FILEVERSION, 0x0001000000000000)
            .set_windres_path(&format!("/usr/bin/{}-w64-mingw32-windres", if cfg!(target_pointer_width = "64") { "x86_64" } else { "i686" }));
        res.compile()?;
    }

    Ok(())
}