use crossterm::event::{KeyCode, KeyModifiers};

use crate::config::HotkeyBinding;
use crate::input::EKeyCommand;
use crate::logger::FileLogger;

#[derive(Clone)]
enum ShortcutKey {
    Char(Vec<char>),
    Named(KeyCode),
}

#[derive(Clone)]
struct Shortcut {
    key: ShortcutKey,
    shift: bool,
    ctrl: bool,
    alt: bool,
    super_key: bool,
}

impl Shortcut {
    fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if modifiers.contains(KeyModifiers::SHIFT) != self.shift {
            return false;
        }
        if modifiers.contains(KeyModifiers::CONTROL) != self.ctrl {
            return false;
        }
        if modifiers.contains(KeyModifiers::ALT) != self.alt {
            return false;
        }
        if modifiers.contains(KeyModifiers::SUPER) != self.super_key {
            return false;
        }

        match (&self.key, code) {
            (ShortcutKey::Named(expected), actual) => *expected == actual,
            (ShortcutKey::Char(chars), KeyCode::Char(actual)) => {
                let actual = normalize_char(actual);
                chars.iter().copied().any(|value| value == actual)
            }
            _ => false,
        }
    }
}

pub struct ShortcutMap {
    bindings: Vec<(Shortcut, EKeyCommand)>,
}

impl ShortcutMap {
    pub fn from_bindings(bindings: &[HotkeyBinding], logger: &mut FileLogger) -> Self {
        let mut parsed = Vec::new();

        for binding in bindings {
            let Some(command) = command_from_action(&binding.action) else {
                logger.log(format!("Unknown hotkey action: {}", binding.action));
                continue;
            };

            let Some(shortcut) = parse_shortcut(&binding.shortcut) else {
                logger.log(format!(
                    "Invalid hotkey shortcut for {}: {}",
                    binding.action, binding.shortcut
                ));
                continue;
            };

            parsed.push((shortcut, command));
        }

        Self { bindings: parsed }
    }

    pub fn resolve(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<EKeyCommand> {
        self.bindings
            .iter()
            .find_map(|(shortcut, command)| shortcut.matches(code, modifiers).then_some(*command))
    }
}

impl Default for ShortcutMap {
    fn default() -> Self {
        Self { bindings: Vec::new() }
    }
}

fn parse_shortcut(value: &str) -> Option<Shortcut> {
    let parts: Vec<String> = value
        .split('+')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect();

    if parts.is_empty() {
        return None;
    }

    let mut shortcut = Shortcut {
        key: ShortcutKey::Named(KeyCode::Null),
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
    };

    for (index, part) in parts.iter().enumerate() {
        let normalized = part.to_ascii_lowercase();
        let is_last = index + 1 == parts.len();

        match normalized.as_str() {
            "shift" if !is_last => shortcut.shift = true,
            "ctrl" | "control" if !is_last => shortcut.ctrl = true,
            "alt" if !is_last => shortcut.alt = true,
            "super" | "win" | "windows" | "cmd" | "command" if !is_last => {
                shortcut.super_key = true
            }
            _ if is_last => {
                shortcut.key = parse_key(part)?;
            }
            _ => return None,
        }
    }

    Some(shortcut)
}

fn parse_key(value: &str) -> Option<ShortcutKey> {
    if value.chars().count() == 1 {
        let ch = normalize_char(value.chars().next()?);
        return Some(ShortcutKey::Char(expand_equivalent_chars(ch)));
    }

    match value.to_ascii_lowercase().as_str() {
        "insert" | "ins" => Some(ShortcutKey::Named(KeyCode::Insert)),
        "enter" | "return" => Some(ShortcutKey::Named(KeyCode::Enter)),
        "tab" => Some(ShortcutKey::Named(KeyCode::Tab)),
        "backspace" => Some(ShortcutKey::Named(KeyCode::Backspace)),
        "delete" | "del" => Some(ShortcutKey::Named(KeyCode::Delete)),
        "home" => Some(ShortcutKey::Named(KeyCode::Home)),
        "end" => Some(ShortcutKey::Named(KeyCode::End)),
        "pageup" | "pgup" => Some(ShortcutKey::Named(KeyCode::PageUp)),
        "pagedown" | "pgdn" => Some(ShortcutKey::Named(KeyCode::PageDown)),
        "left" => Some(ShortcutKey::Named(KeyCode::Left)),
        "right" => Some(ShortcutKey::Named(KeyCode::Right)),
        "up" => Some(ShortcutKey::Named(KeyCode::Up)),
        "down" => Some(ShortcutKey::Named(KeyCode::Down)),
        _ => None,
    }
}

fn command_from_action(action: &str) -> Option<EKeyCommand> {
    match action.trim().to_ascii_lowercase().as_str() {
        "find" => Some(EKeyCommand::Find),
        "replace" => Some(EKeyCommand::Replace),
        "new_file" => Some(EKeyCommand::NewFile),
        "open_file" => Some(EKeyCommand::OpenFile),
        "open_in_explorer" => Some(EKeyCommand::OpenInExplorer),
        "save_file" => Some(EKeyCommand::SaveFile),
        "save_file_as" => Some(EKeyCommand::SaveFileAs),
        "undo" => Some(EKeyCommand::Undo),
        "redo" => Some(EKeyCommand::Redo),
        "select_all" => Some(EKeyCommand::SelectAll),
        "copy" => Some(EKeyCommand::Copy),
        "cut" => Some(EKeyCommand::Cut),
        "paste" => Some(EKeyCommand::Paste),
        _ => None,
    }
}

fn normalize_char(value: char) -> char {
    value.to_lowercase().next().unwrap_or(value)
}

fn expand_equivalent_chars(value: char) -> Vec<char> {
    let mut chars = vec![value];

    if let Some(alias) = keyboard_pair(value) {
        chars.push(alias);
    }

    chars.sort_unstable();
    chars.dedup();
    chars
}

fn keyboard_pair(value: char) -> Option<char> {
    match value {
        'a' => Some('ф'),
        'b' => Some('и'),
        'c' => Some('с'),
        'd' => Some('в'),
        'e' => Some('у'),
        'f' => Some('а'),
        'g' => Some('п'),
        'h' => Some('р'),
        'i' => Some('ш'),
        'j' => Some('о'),
        'k' => Some('л'),
        'l' => Some('д'),
        'm' => Some('ь'),
        'n' => Some('т'),
        'o' => Some('щ'),
        'p' => Some('з'),
        'q' => Some('й'),
        'r' => Some('к'),
        's' => Some('ы'),
        't' => Some('е'),
        'u' => Some('г'),
        'v' => Some('м'),
        'w' => Some('ц'),
        'x' => Some('ч'),
        'y' => Some('н'),
        'z' => Some('я'),
        'ф' => Some('a'),
        'и' => Some('b'),
        'с' => Some('c'),
        'в' => Some('d'),
        'у' => Some('e'),
        'а' => Some('f'),
        'п' => Some('g'),
        'р' => Some('h'),
        'ш' => Some('i'),
        'о' => Some('j'),
        'л' => Some('k'),
        'д' => Some('l'),
        'ь' => Some('m'),
        'т' => Some('n'),
        'щ' => Some('o'),
        'з' => Some('p'),
        'й' => Some('q'),
        'к' => Some('r'),
        'ы' => Some('s'),
        'е' => Some('t'),
        'г' => Some('u'),
        'м' => Some('v'),
        'ц' => Some('w'),
        'ч' => Some('x'),
        'н' => Some('y'),
        'я' => Some('z'),
        _ => None,
    }
}
