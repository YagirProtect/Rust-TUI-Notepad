use crate::screen_buffer::Color;

const BLUE_KEYWORDS: &[&str] = &[
    "var",
    "let",
    "new",
    "out",
    "in",
    "lock",
    "void",
    "fn",
    "#ifdef",
    "#define",
    "#pragma",
    "#if",
    "#endif",
    "#include",
    "#region",
    "#endregion",
    "function",
    "struct",
    "class",
    "public",
    "pub",
    "static",
    "const",
    "int",
    "float",
    "double",
    "uint",
    "float2",
    "float3",
    "float4",
    "if",
    "else",
    "elseif",
    "include",
    "Pass",
    "Tags",
    "SubShader",
    "using",
    "namespace",
    "this",
    "ref",
    "private",
    "interface",
    "enum",
    "bool",
    "long",
    "override",
    "virtual"
];

const GREEN_KEYWORDS: &[&str] = &[
    "Array",
    "List",
    "Vector",
    "Vector2",
    "Vector3",
    "Vector4",
    "T",
];

const PINK_KEYWORDS: &[&str] = &[
    "for",
    "foreach",
    "while",
    "do",
    "return",
    "break",
    "case",
    "default",
];

#[derive(Copy, Clone, Default)]
pub struct HighlightState {
    pub in_block_comment: bool,
    pub in_html_comment: bool,
}

pub fn state_before_line(lines: &[Vec<char>], end_line_exclusive: usize) -> HighlightState {
    let mut state = HighlightState::default();
    let end = end_line_exclusive.min(lines.len());

    for line in &lines[..end] {
        (_, state) = line_colors_with_state(line, state);
    }

    state
}

pub fn line_colors_with_state(
    line: &[char],
    mut state: HighlightState,
) -> (Vec<Option<Color>>, HighlightState) {
    let mut colors = vec![None; line.len()];
    let mut index = 0;

    while index < line.len() {
        let ch = line[index];

        if state.in_block_comment {
            colors[index] = Some(Color::DarkGreen);
            if starts_with(line, index, &['*', '/']) {
                if index + 1 < line.len() {
                    colors[index + 1] = Some(Color::DarkGreen);
                }
                index += 2;
                state.in_block_comment = false;
            } else {
                index += 1;
            }
            continue;
        }

        if state.in_html_comment {
            colors[index] = Some(Color::DarkGreen);
            if starts_with(line, index, &['-', '-', '>']) {
                if index + 1 < line.len() {
                    colors[index + 1] = Some(Color::DarkGreen);
                }
                if index + 2 < line.len() {
                    colors[index + 2] = Some(Color::DarkGreen);
                }
                index += 3;
                state.in_html_comment = false;
            } else {
                index += 1;
            }
            continue;
        }

        if ch == '/' && line.get(index + 1) == Some(&'/') {
            for color in &mut colors[index..] {
                *color = Some(Color::DarkGreen);
            }
            break;
        }

        if starts_with(line, index, &['/', '*']) {
            colors[index] = Some(Color::DarkGreen);
            if index + 1 < line.len() {
                colors[index + 1] = Some(Color::DarkGreen);
            }
            index += 2;
            state.in_block_comment = true;
            continue;
        }

        if starts_with(line, index, &['<', '!', '-', '-']) {
            for offset in 0..4 {
                if index + offset < line.len() {
                    colors[index + offset] = Some(Color::DarkGreen);
                }
            }
            index += 4;
            state.in_html_comment = true;
            continue;
        }

        if is_quote(ch) {
            let quote = ch;
            colors[index] = Some(Color::Green);
            index += 1;

            while index < line.len() {
                colors[index] = Some(Color::Green);
                if line[index] == quote && !is_escaped(line, index) {
                    index += 1;
                    break;
                }
                index += 1;
            }
            continue;
        }

        if ch.is_ascii_digit() && can_start_number(line, index) {
            let start = index;
            index += 1;
            while index < line.len() && is_number_char(line[index]) {
                index += 1;
            }
            for color in &mut colors[start..index] {
                *color = Some(Color::Yellow);
            }
            continue;
        }

        if is_bracket(ch) {
            colors[index] = Some(Color::Red);
            index += 1;
            continue;
        }

        if let Some((keyword_len, color)) = match_keyword_at(line, index) {
            for slot in &mut colors[index..index + keyword_len] {
                *slot = Some(color);
            }
            index += keyword_len;
            continue;
        }

        index += 1;
    }

    (colors, state)
}

fn is_quote(ch: char) -> bool {
    matches!(ch, '"' | '\'' | '`')
}

fn is_bracket(ch: char) -> bool {
    matches!(ch, '(' | ')' | '[' | ']' | '{' | '}')
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_number_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | 'x' | 'X')
}

fn can_start_number(line: &[char], index: usize) -> bool {
    if index == 0 {
        return true;
    }

    let prev = line[index - 1];
    prev.is_whitespace() || is_bracket(prev) || prev.is_ascii_digit()
}

fn is_escaped(line: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }

    let mut slash_count = 0;
    let mut current = index;
    while current > 0 {
        current -= 1;
        if line[current] != '\\' {
            break;
        }
        slash_count += 1;
    }

    slash_count % 2 == 1
}

fn starts_with(line: &[char], start: usize, pattern: &[char]) -> bool {
    let end = start + pattern.len();
    end <= line.len() && line[start..end] == pattern[..]
}

fn match_keyword_at(line: &[char], start: usize) -> Option<(usize, Color)> {
    match_keyword_from_list(line, start, GREEN_KEYWORDS, Color::Green)
        .or_else(|| match_keyword_from_list(line, start, PINK_KEYWORDS, Color::Pink))
        .or_else(|| match_keyword_from_list(line, start, BLUE_KEYWORDS, Color::Blue))
}

fn match_keyword_from_list(
    line: &[char],
    start: usize,
    keywords: &[&str],
    color: Color,
) -> Option<(usize, Color)> {
    let mut best_len = None;

    for keyword in keywords {
        let keyword_chars: Vec<char> = keyword.chars().collect();
        let end = start + keyword_chars.len();
        if end > line.len() {
            continue;
        }

        if line[start..end] != keyword_chars[..] {
            continue;
        }

        let prev_ok = start == 0 || !is_word_char(line[start - 1]);
        let next_ok = end == line.len() || !is_word_char(line[end]);
        if !prev_ok || !next_ok {
            continue;
        }

        if best_len.is_none_or(|current| keyword_chars.len() > current) {
            best_len = Some(keyword_chars.len());
        }
    }

    best_len.map(|len| (len, color))
}
