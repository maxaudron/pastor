use std::fs::File;
use std::io::prelude::*;

/// Generates CSS from SCSS and writes it to a file.
/// Could be extended further if more than one SCSS file exists.
fn main() -> Result<(), Box<grass::Error>> {
    let sass: String = grass::from_path("static/styles/main.scss", &grass::Options::default())?;
    let mut buffer = File::create("static/styles/main.css")?;
    buffer.write_all(sass.as_bytes())?;
    println!("cargo:rerun-if-changed=static/styles/main.scss");
    Ok(())
}
