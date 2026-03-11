#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return,
    };

    register_open_with(&current_exe);
    let wt_settings = configure_windows_terminal();
    let _ = launch_editor(&current_exe, &args);
    restore_windows_terminal(wt_settings);
}

#[cfg(target_os = "windows")]
fn register_open_with(current_exe: &Path) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let command = format!("\"{}\" \"%1\"", current_exe.display());
    let icon = format!("\"{}\",0", current_exe.display());
    let exe_name = current_exe
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("notepad_open_with.exe");

    let updates = [
        (
            format!(r"HKCU\Software\Classes\Applications\{}", exe_name),
            Some(("FriendlyAppName", "REG_SZ", "NOTEPAD".to_string())),
        ),
        (
            format!(r"HKCU\Software\Classes\Applications\{}\shell\open\command", exe_name),
            Some(("", "REG_SZ", command.clone())),
        ),
        (
            format!(r"HKCU\Software\Classes\Applications\{}\SupportedTypes", exe_name),
            Some((".txt", "REG_SZ", String::new())),
        ),
        (
            r"HKCU\Software\Classes\NOTEPAD.txt".to_string(),
            Some(("", "REG_SZ", "NOTEPAD text document".to_string())),
        ),
        (
            r"HKCU\Software\Classes\NOTEPAD.txt\DefaultIcon".to_string(),
            Some(("", "REG_SZ", icon)),
        ),
        (
            r"HKCU\Software\Classes\NOTEPAD.txt\shell\open\command".to_string(),
            Some(("", "REG_SZ", command.clone())),
        ),
        (
            r"HKCU\Software\Classes\.txt\OpenWithProgids".to_string(),
            Some(("NOTEPAD.txt", "REG_SZ", String::new())),
        ),
    ];

    for (key, value) in updates {
        let mut cmd = Command::new("reg.exe");
        cmd.creation_flags(CREATE_NO_WINDOW).arg("add").arg(key).arg("/f");

        if let Some((name, kind, data)) = value {
            if name.is_empty() {
                cmd.arg("/ve");
            } else {
                cmd.arg("/v").arg(name);
            }

            cmd.arg("/t").arg(kind).arg("/d").arg(data);
        }

        let _ = cmd.status();
    }
}

#[cfg(not(target_os = "windows"))]
fn register_open_with(_current_exe: &Path) {}

fn launch_editor(current_exe: &Path, args: &[OsString]) -> io::Result<()> {
    let root = current_exe
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "launcher directory not found"))?;

    let exe_path = root.join("NOTEPAD.exe");
    if exe_path.exists() {
        let mut command = Command::new(exe_path);
        for arg in args {
            command.arg(arg);
        }

        return command.status().map(|_| ());
    }

    let mut command = Command::new("cargo");
    command.arg("run").arg("--bin").arg("NOTEPAD").arg("--");

    for arg in args {
        command.arg(arg);
    }

    command.current_dir(root).status().map(|_| ())
}

#[cfg(target_os = "windows")]
struct WindowsTerminalState {
    settings_path: PathBuf,
    original_raw: String,
}

#[cfg(not(target_os = "windows"))]
struct WindowsTerminalState;

#[cfg(target_os = "windows")]
fn configure_windows_terminal() -> Option<WindowsTerminalState> {
    let settings_path = windows_terminal_settings_path()?;
    let original_raw = usable_settings_raw(&settings_path)?;
    let mut settings: serde_json::Value = serde_json::from_str(&original_raw).ok()?;

    unbind_ctrl_v(&mut settings);
    upsert_scheme(&mut settings);
    set_default_scheme(&mut settings);

    let updated_raw = serde_json::to_string_pretty(&settings).ok()?;
    write_atomic_utf8(&settings_path, &updated_raw).ok()?;

    Some(WindowsTerminalState {
        settings_path,
        original_raw,
    })
}

#[cfg(not(target_os = "windows"))]
fn configure_windows_terminal() -> Option<WindowsTerminalState> {
    None
}

#[cfg(target_os = "windows")]
fn restore_windows_terminal(state: Option<WindowsTerminalState>) {
    if let Some(state) = state {
        let _ = write_atomic_utf8(&state.settings_path, &state.original_raw);
    }
}

#[cfg(not(target_os = "windows"))]
fn restore_windows_terminal(_state: Option<WindowsTerminalState>) {}

#[cfg(target_os = "windows")]
fn windows_terminal_settings_path() -> Option<PathBuf> {
    let local_app_data = env::var_os("LOCALAPPDATA")?;
    let local_app_data = PathBuf::from(local_app_data);
    let candidates = [
        local_app_data.join(r"Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"),
        local_app_data.join(r"Packages\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\LocalState\settings.json"),
        local_app_data.join(r"Microsoft\Windows Terminal\settings.json"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(target_os = "windows")]
fn usable_settings_raw(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    if serde_json::from_str::<serde_json::Value>(&raw).is_ok() {
        return Some(raw);
    }

    repair_settings_raw(&raw)
}

#[cfg(target_os = "windows")]
fn repair_settings_raw(raw: &str) -> Option<String> {
    if serde_json::from_str::<serde_json::Value>(raw).is_ok() {
        return Some(raw.to_string());
    }

    let marker = "{\"$help\"";
    let mut start = 0usize;
    while let Some(index) = raw[start..].find(marker) {
        let candidate_start = start + index;
        let candidate = &raw[candidate_start..];
        if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
            return Some(candidate.to_string());
        }
        start = candidate_start + marker.len();
    }

    None
}

#[cfg(target_os = "windows")]
fn unbind_ctrl_v(settings: &mut serde_json::Value) {
    let list_name = if settings.get("keybindings").is_some() {
        "keybindings"
    } else if settings.get("actions").is_some() {
        "actions"
    } else {
        settings["keybindings"] = serde_json::Value::Array(Vec::new());
        "keybindings"
    };

    let items = settings
        .get(list_name)
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut filtered = Vec::new();
    for item in items {
        if !binding_has_ctrl_v(&item) {
            filtered.push(item);
        }
    }

    filtered.push(serde_json::json!({
        "id": serde_json::Value::Null,
        "keys": ["ctrl+v"]
    }));

    settings[list_name] = serde_json::Value::Array(filtered);
}

#[cfg(target_os = "windows")]
fn binding_has_ctrl_v(binding: &serde_json::Value) -> bool {
    let Some(keys) = binding.get("keys") else {
        return false;
    };

    if let Some(value) = keys.as_str() {
        return value.eq_ignore_ascii_case("ctrl+v");
    }

    keys.as_array()
        .map(|items| {
            items.iter().any(|item| {
                item.as_str()
                    .is_some_and(|value| value.eq_ignore_ascii_case("ctrl+v"))
            })
        })
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn upsert_scheme(settings: &mut serde_json::Value) {
    if settings.get("schemes").is_none() {
        settings["schemes"] = serde_json::Value::Array(Vec::new());
    }

    let schemes = settings
        .get("schemes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut updated = Vec::new();
    for scheme in schemes {
        if scheme
            .get("name")
            .and_then(|value| value.as_str())
            != Some("One Half Dark (Copy)")
        {
            updated.push(scheme);
        }
    }

    updated.push(serde_json::json!({
        "background": "#282C34",
        "black": "#282C34",
        "blue": "#61AFEF",
        "brightBlack": "#5A6374",
        "brightBlue": "#61AFEF",
        "brightCyan": "#56B6C2",
        "brightGreen": "#98C379",
        "brightPurple": "#C678DD",
        "brightRed": "#E06C75",
        "brightWhite": "#DCDFE4",
        "brightYellow": "#E5C07B",
        "cursorColor": "#FFFFFF",
        "cyan": "#56B6C2",
        "foreground": "#DCDFE4",
        "green": "#98C379",
        "name": "One Half Dark (Copy)",
        "purple": "#C678DD",
        "red": "#E06C75",
        "selectionBackground": "#FFFFFF",
        "white": "#DCDFE4",
        "yellow": "#E5C07B"
    }));

    settings["schemes"] = serde_json::Value::Array(updated);
}

#[cfg(target_os = "windows")]
fn set_default_scheme(settings: &mut serde_json::Value) {
    if settings.get("profiles").is_none() || !settings["profiles"].is_object() {
        settings["profiles"] = serde_json::json!({});
    }

    if settings["profiles"].get("defaults").is_none() || !settings["profiles"]["defaults"].is_object() {
        settings["profiles"]["defaults"] = serde_json::json!({});
    }

    settings["profiles"]["defaults"]["colorScheme"] =
        serde_json::Value::String("One Half Dark (Copy)".to_string());
}

#[cfg(target_os = "windows")]
fn write_atomic_utf8(path: &Path, content: &str) -> io::Result<()> {
    let temp_path = path.with_extension(format!(
        "{}.notepad.tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
    ));
    fs::write(&temp_path, content)?;
    fs::rename(temp_path, path)?;
    Ok(())
}
