pub mod lexer;
pub mod parser;

pub use lexer::{Lexer, Token, TokenType, LexerConfig};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[cfg(test)]
mod tests {
    #[test]
    fn hello() {
        assert_eq!(2 + 2, 4);
    }
}