use anyhow::Result;
use std::fmt::Write;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

lazy_static::lazy_static!(
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
);

pub fn highlight_text(text: &str, extension: &str, theme: Option<&str>) -> Result<String> {
    let syntax = if let Some(s) = SYNTAX_SET.find_syntax_by_extension(extension) {
        s
    } else {
        SYNTAX_SET.find_syntax_plain_text()
    };
    let mut h = HighlightLines::new(
        syntax,
        &THEME_SET.themes[theme.unwrap_or("base16-ocean.dark")],
    );
    let mut output = String::new();
    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, &SYNTAX_SET).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges, false);
        write!(&mut output, "{}", escaped)?;
    }
    Ok(output)
}
