use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    copy_launcher("run_notepad_wt.bat");
    copy_launcher("run_notepad_wt.cmd");
    copy_launcher("run_notepad_wt.ps1");
}

fn copy_launcher(file_name: &str) {
    println!("cargo:rerun-if-changed={file_name}");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("target profile directory");

    copy_if_exists(&manifest_dir.join(file_name), &target_dir.join(file_name));
}

fn copy_if_exists(source: &Path, destination: &Path) {
    if !source.exists() {
        return;
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).expect("create destination directory");
    }

    fs::copy(source, destination).expect("copy launcher");
}
