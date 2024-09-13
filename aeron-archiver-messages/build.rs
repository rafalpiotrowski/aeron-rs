use std::fs::OpenOptions;
use std::io::{self, Read, Seek, Write};
use std::process::Command;
use std::str;

fn suppress_warnings(module: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("./generated/{module}/src/lib.rs"))?;

    let mut contents = "#![allow(warnings)]\n".to_owned();
    file.read_to_string(&mut contents)?;

    file.rewind()?;
    file.write_all(contents.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=schema");

    let output = Command::new("sbe")
        .arg("schema")
        .arg("generate")
        .arg("--file")
        .arg("schema/messages.xml")
        .arg("--output-dir")
        .arg("generated")
        .arg("--language")
        .arg("rust")
        .output()
        .expect("Unable to execute SBE generator");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr).unwrap();
        panic!("SBE generation failed\n{}", stderr);
    }

    suppress_warnings("aeron_archiver_codecs").expect("Unable to prepend warning suppression");
}
