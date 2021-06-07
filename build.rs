use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), Box<grass::Error>> {
    generate_css("main")
}

/// Generates CSS from SCSS and writes it to a file.
fn generate_css(scss_name: &str) -> Result<(), Box<grass::Error>> {
    let sass: String = grass::from_path(
        &format!("static/styles/{}.scss", scss_name),
        &grass::Options::default(),
    )?;
    let mut buffer = File::create(format!("static/styles/{}.css", scss_name))?;
    buffer.write_all(sass.as_bytes())?;
    println!("cargo:rerun-if-changed=static/styles/{}.scss", scss_name);
    Ok(())
}
