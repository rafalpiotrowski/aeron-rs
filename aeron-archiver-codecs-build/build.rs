use std::fs::OpenOptions;
use std::io::{self, Read, Seek, Write};
use std::process::Command;
use std::str;

fn suppress_warnings(module: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("../{module}/src/lib.rs"))?;

    let mut contents = "#![allow(warnings)]\n".to_owned();
    file.read_to_string(&mut contents)?;

    file.rewind()?;
    file.write_all(contents.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn cargo_toml_add_crate_io_necessary_information(module: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("../{module}/Cargo.toml"))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Check if the [package] section exists
    let package_section = "[package]";
    if let Some(package_start) = contents.find(package_section) {
        // Find where the [package] section ends
        let package_end = contents[package_start..]
            .find("\n[")
            .map(|idx| package_start + idx)
            .unwrap_or(contents.len());

        // Extract the package section content
        let package_content = &contents[package_start..package_end];

        let mut new_package_content = package_content.to_string();

        // If the license field is not already present, add it
        if !package_content.contains("license") {
            new_package_content.push_str("license = \"MIT OR Apache-2.0\"\n");
        }

        if !package_content.contains("description") {
            new_package_content.push_str("description = \"Aeron Archiver codecs\"\n");
        }

        if !package_content.contains("description") {
            new_package_content.push_str("readme = \"README.md\"\n");
        }

        if !package_content.contains("repository") {
            new_package_content.push_str("repository = \"https://github.com/rafalpiotrowski/aeron-rs\"\n");
        }

        if !package_content.contains("keywords") {
            new_package_content.push_str("keywords = [ \"aeron\", \"sbe\", \"messaging\", \"HFT\", \"finance\" ]\n");
        }

        if !package_content.contains("categories") {
            new_package_content.push_str("categories = [ \"network-programming\", ]\n");
        }

        // Replace the old package section with the new one
        let new_contents = contents.replacen(package_content, &new_package_content, 1);

        // Update the file with new contents
        file.set_len(0)?; // Clear the file
        file.seek(io::SeekFrom::Start(0))?;
        file.write_all(new_contents.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=schema");

    let output = Command::new("sbe-cli")
        .arg("schema")
        .arg("generate")
        .arg("--file")
        .arg("schema/messages.xml")
        .arg("--output-dir")
        .arg("..")
        .arg("--language")
        .arg("rust")
        .output()
        .expect("Unable to execute SBE generator");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr).unwrap();
        panic!("SBE generation failed\n{}", stderr);
    }

    suppress_warnings("aeron_archiver_codecs").expect("Unable to prepend warning suppression");
    cargo_toml_add_crate_io_necessary_information("aeron_archiver_codecs")
        .expect("Unable to add necessary information to Cargo.toml");
}
