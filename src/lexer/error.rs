use std::fmt;

#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub suggestion: Option<String>,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Line {}, Column {}: {}",
            self.line, self.column, self.message
        )?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " - Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}
