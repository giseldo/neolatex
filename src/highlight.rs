use egui::{
    text::{LayoutJob, TextWrapping},
    Color32, FontId, TextFormat,
};

const TEXT:    Color32 = Color32::from_rgb(200, 203, 211);
const COMMAND: Color32 = Color32::from_rgb(97, 175, 239);  // \foo
const KEYWORD: Color32 = Color32::from_rgb(198, 120, 221); // \begin \end \usepackage ...
const SECTION: Color32 = Color32::from_rgb(229, 192, 123); // \section ...
const ENV:     Color32 = Color32::from_rgb(152, 195, 121); // {itemize}
const COMMENT: Color32 = Color32::from_rgb(92, 99, 112);
const MATH:    Color32 = Color32::from_rgb(209, 154, 102);
const BRACE:   Color32 = Color32::from_rgb(125, 130, 142);
const SPECIAL: Color32 = Color32::from_rgb(86, 182, 194);  // & _ ^

fn fmt(color: Color32, size: f32, italics: bool) -> TextFormat {
    TextFormat { font_id: FontId::monospace(size), color, italics, ..Default::default() }
}

fn command_color(name: &str) -> Color32 {
    match name {
        "\\begin" | "\\end" | "\\documentclass" | "\\usepackage" | "\\input" | "\\include"
        | "\\newcommand" | "\\renewcommand" | "\\def" => KEYWORD,
        "\\part" | "\\chapter" | "\\section" | "\\subsection" | "\\subsubsection"
        | "\\paragraph" | "\\section*" | "\\subsection*" | "\\chapter*" => SECTION,
        _ => COMMAND,
    }
}

pub fn highlight(source: &str, font_size: f32) -> LayoutJob {
    let mut job = LayoutJob {
        wrap: TextWrapping { max_width: f32::INFINITY, ..Default::default() },
        ..Default::default()
    };

    let b = source.as_bytes();
    let n = b.len();
    let mut i = 0;
    let mut env_next = false;

    macro_rules! push {
        ($range:expr, $color:expr) => {
            if !$range.is_empty() {
                job.append(&source[$range], 0.0, fmt($color, font_size, false));
            }
        };
    }

    while i < n {
        let was_env = std::mem::replace(&mut env_next, false);

        match b[i] {
            b'%' => {
                let s = i;
                while i < n && b[i] != b'\n' { i += 1; }
                job.append(&source[s..i], 0.0, fmt(COMMENT, font_size, true));
            }
            b'\\' => {
                let s = i;
                i += 1;
                if i < n {
                    if b[i].is_ascii_alphabetic() {
                        while i < n && (b[i].is_ascii_alphabetic() || b[i] == b'@') { i += 1; }
                        if i < n && b[i] == b'*' { i += 1; }
                    } else {
                        // single special char, may be a multibyte codepoint
                        i += source[i..].chars().next().map_or(1, char::len_utf8);
                    }
                }
                let name = &source[s..i];
                env_next = matches!(name, "\\begin" | "\\end");
                push!(s..i, command_color(name));
            }
            b'$' => {
                let s = i;
                let display = i + 1 < n && b[i + 1] == b'$';
                let skip = if display { 2 } else { 1 };
                i += skip;
                let needle = if display { "$$" } else { "$" };
                match source[i..].find(needle) {
                    Some(pos) => i += pos + needle.len(),
                    None => i = n,
                }
                push!(s..i, MATH);
            }
            b'{' if was_env => {
                push!(i..i + 1, BRACE);
                i += 1;
                let s = i;
                while i < n && b[i] != b'}' && b[i] != b'\n' { i += 1; }
                push!(s..i, ENV);
            }
            b'{' | b'}' | b'[' | b']' => { push!(i..i + 1, BRACE);   i += 1; }
            b'&' | b'_' | b'^'        => { push!(i..i + 1, SPECIAL); i += 1; }
            _ => {
                let s = i;
                while i < n && !matches!(b[i], b'%' | b'\\' | b'$' | b'{' | b'}' | b'[' | b']' | b'&' | b'_' | b'^') {
                    i += 1;
                }
                push!(s..i, TEXT);
            }
        }
    }

    job
}
