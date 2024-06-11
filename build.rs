use std::fs::File;
use std::io::prelude::*;

/// Goes through all SCSS files and generates CSS files.
fn main() -> Result<(), Box<grass::Error>> {
    for entry in glob::glob("static/styles/*.scss").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => generate_css(path.to_str().unwrap())?,
            Err(e) => println!("{:?}", e),
        }
    }

    #[cfg(feature = "magic_static")]
    println!("cargo:rustc-link-lib=static=magic");

    //#[cfg(feature = "magic_static")]
    //println!("cargo:rustc-env=PASTOR_MIME_DB=/usr/share/misc/magic.mgc");

    Ok(())
}

/// Generates CSS from SCSS and writes it to a file.
fn generate_css(scss_path: &str) -> Result<(), Box<grass::Error>> {
    let sass: String = grass::from_path(scss_path, &grass::Options::default())?;
    let mut buffer = File::create(scss_path.replacen("scss", "css", 1))?;
    buffer.write_all(sass.as_bytes())?;

    // This instructs cargo to rerun this build script if this input file has changed.
    // Since the directory method above does not work, this also means if a new file is added,
    // a change in an existing file must be made for cargo to start tracking the new file.
    // println!("cargo:rerun-if-changed={}", scss_path);

    Ok(())
}
