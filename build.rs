use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_windows_icon();
}

fn compile_windows_icon() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let icon_path = manifest_dir.join("assets").join("text_document_icon.ico");
    let manifest_path = manifest_dir.join("assets").join("notepad_open_with.manifest");

    println!("cargo:rerun-if-changed={}", icon_path.display());
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    if !icon_path.exists() {
        panic!("Missing icon asset: {}", icon_path.display());
    }
    if !manifest_path.exists() {
        panic!("Missing manifest asset: {}", manifest_path.display());
    }

    let escaped_icon_path = icon_path.to_string_lossy().replace('\\', "\\\\");
    let escaped_manifest_path = manifest_path.to_string_lossy().replace('\\', "\\\\");
    let rc_path = out_dir.join("notepad_open_with.rc");
    let res_path = out_dir.join("notepad_open_with.res");
    let obj_path = out_dir.join("notepad_open_with.res.obj");
    let rc_body = format!(
        "APP_ICON ICON \"{}\"\n1 24 \"{}\"\n",
        escaped_icon_path,
        escaped_manifest_path
    );

    fs::write(&rc_path, rc_body).expect("write icon rc file");

    let rc_exe = find_rc_exe().expect("failed to locate rc.exe in PATH or Windows SDK");

    let status = Command::new(rc_exe)
        .arg("/nologo")
        .arg(format!("/fo{}", res_path.display()))
        .arg(&rc_path)
        .status()
        .expect("run rc.exe for icon resource");

    if !status.success() {
        panic!("rc.exe failed to compile icon resource");
    }

    let cvtres_exe = find_cvtres_exe().expect("failed to locate cvtres.exe in Visual Studio tools");
    let machine = match env::var("CARGO_CFG_TARGET_ARCH").ok().as_deref() {
        Some("x86_64") => "X64",
        Some("x86") => "X86",
        Some("aarch64") => "ARM64",
        _ => "X64",
    };

    let status = Command::new(cvtres_exe)
        .arg(format!("/MACHINE:{machine}"))
        .arg(format!("/OUT:{}", obj_path.display()))
        .arg(&res_path)
        .status()
        .expect("run cvtres.exe for icon resource");

    if !status.success() {
        panic!("cvtres.exe failed to convert icon resource");
    }

    println!(
        "cargo:rustc-link-arg-bin=notepad_open_with={}",
        obj_path.display()
    );
}

fn find_rc_exe() -> Option<PathBuf> {
    let path_candidate = find_in_path("rc.exe");
    if path_candidate.is_some() {
        return path_candidate;
    }

    let program_files_x86 = env::var_os("ProgramFiles(x86)")?;
    let arch_dir = match env::var("CARGO_CFG_TARGET_ARCH").ok().as_deref() {
        Some("x86_64") => "x64",
        Some("x86") => "x86",
        Some("aarch64") => "arm64",
        _ => "x64",
    };

    let bin_root = PathBuf::from(program_files_x86)
        .join("Windows Kits")
        .join("10")
        .join("bin");

    let mut candidates = Vec::new();
    if let Ok(entries) = fs::read_dir(&bin_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let candidate = path.join(arch_dir).join("rc.exe");
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    candidates.sort();
    candidates.pop()
}

fn find_in_path(file_name: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for path in env::split_paths(&paths) {
        let candidate = path.join(file_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn find_cvtres_exe() -> Option<PathBuf> {
    let path_candidate = find_in_path("cvtres.exe");
    if path_candidate.is_some() {
        return path_candidate;
    }

    let program_files = env::var_os("ProgramFiles")?;
    let tools_root = PathBuf::from(program_files)
        .join("Microsoft Visual Studio")
        .join("2022");

    let arch_dir = match env::var("CARGO_CFG_TARGET_ARCH").ok().as_deref() {
        Some("x86_64") => "Hostx64\\x64",
        Some("x86") => "Hostx64\\x86",
        Some("aarch64") => "Hostx64\\arm64",
        _ => "Hostx64\\x64",
    };

    let mut candidates = Vec::new();
    if let Ok(editions) = fs::read_dir(&tools_root) {
        for edition in editions.flatten() {
            let msvc_root = edition
                .path()
                .join("VC")
                .join("Tools")
                .join("MSVC");

            if let Ok(versions) = fs::read_dir(msvc_root) {
                for version in versions.flatten() {
                    let candidate = version.path().join("bin").join(arch_dir).join("cvtres.exe");
                    if candidate.exists() {
                        candidates.push(candidate);
                    }
                }
            }
        }
    }

    candidates.sort();
    candidates.pop()
}
