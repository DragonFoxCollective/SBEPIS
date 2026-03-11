use std::{env, io};

use winres::WindowsResource;

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = WindowsResource::new();

        if cfg!(unix) {
            res.set_toolkit_path("/usr/bin");
            res.set_windres_path("x86_64-w64-mingw32-windres");
        }

        // This path can be absolute, or relative to your crate root.
        res.set_icon("assets/house.ico");
        res.compile()?;
    }
    Ok(())
}
