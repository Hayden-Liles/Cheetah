#[derive(Debug, Clone)]
pub struct LexerConfig {
    pub tab_width: usize,
    pub enforce_indent_consistency: bool,
    pub standard_indent_size: usize,
    pub allow_trailing_semicolon: bool,
    pub allow_tabs_in_indentation: bool,
}

impl Default for LexerConfig {
    fn default() -> Self {
        LexerConfig {
            tab_width: 4,
            enforce_indent_consistency: true,
            standard_indent_size: 4,
            allow_trailing_semicolon: true,
            allow_tabs_in_indentation: false,
        }
    }
}