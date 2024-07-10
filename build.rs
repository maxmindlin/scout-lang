use core::panic;
use std::path::Path;
use std::{env, fs, io};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            let content = fs::read_to_string(entry.path())?;
            fs::write(dst.as_ref().join(entry.file_name()), content)?;
        }
    }
    Ok(())
}

fn main() {
    println!("cargo::rerun-if-changed=scout-lib/");
    let scout_dir = match env::var("SCOUT_PATH") {
        Ok(s) => Path::new(&s).to_path_buf(),
        Err(_) => match env::var("HOME") {
            Ok(s) => Path::new(&s).join("scout-lang"),
            Err(_) => panic!("Unable to find $HOME or $SCOUT_PATH. Please set one."),
        },
    };
    let path = scout_dir.join("scout-lib").to_owned();
    copy_dir_all("scout-lib", path).unwrap();
}
