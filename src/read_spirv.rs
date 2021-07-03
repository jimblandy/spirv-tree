//! Utilites for reading SPIR-V files.

use std::{ffi, fs, io, path, process};

/// Read SPIR-V bytes from `path`, assembling if necessary.
///
/// If `path` ends with `.spv`, return the file's contents as bytes.
///
/// If `path` ends with `.spvasm`, run `spirv-as` to assemble it, and return the
/// bytes of the assembled SPIR-V file.
pub fn read_spirv_bytes(path: impl AsRef<path::Path>) -> io::Result<Vec<u8>> {
    let path = path.as_ref();
    let extension = path.extension();
    if extension == Some(ffi::OsStr::new("spv")) {
        fs::read(path)
    } else if extension == Some(ffi::OsStr::new("spvasm")) {
        let output = process::Command::new("spirv-as")
            .arg(path)
            .args(&["--preserve-numeric-ids", "-o", "-"])
            .output()?;
        if !output.status.success() {
            let message = format!(
                "spirv-as failed on {}:\n{}",
                path.display(),
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(io::Error::new(io::ErrorKind::Other, message));
        }
        Ok(output.stdout)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("unrecognized file name extension: {}", path.display()),
        ))
    }
}
