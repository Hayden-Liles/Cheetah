#[cfg(test)]
mod lexer_edge_cases_tests {
    use cheetah::lexer::{Lexer, LexerConfig, Token, TokenType};

    // Helper function to check if a string can be tokenized without errors
    fn assert_parses(input: &str) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check that we have at least one token (EOF)
        assert!(!tokens.is_empty(), "Failed to tokenize: {}", input);
        
        // Check that the last token is EOF
        assert_eq!(tokens.last().unwrap().token_type, TokenType::EOF, 
                  "Last token should be EOF for input: {}", input);
    }

    #[test]
    fn test_string_edge_cases() {
        // Empty strings
        assert_parses("''");
        assert_parses("\"\"");
        
        // Strings with escapes
        assert_parses("'String with \\'quote\\''");
        assert_parses("\"String with \\\"double quote\\\"\"");
        
        // String with Unicode escapes
        assert_parses("'\\u0041\\u0042\\u0043'");  // ABC
        
        // String with hex escapes
        assert_parses("'\\x41\\x42\\x43'");  // ABC
        
        // String with unusual characters
        assert_parses("'String with tab\\t and newline\\n'");
        
        // Triple-quoted string edge cases
        assert_parses("'''String with both ' and \" quotes'''");
        assert_parses("'''''"); // Single quote inside triple quotes
    }

    #[test]
    fn test_complex_string_literals() {
        // Raw string
        assert_parses("r'Raw\\nString'");
        
        // Byte string
        assert_parses("b'Byte String'");
        
        // Raw byte string
        assert_parses("br'Raw\\nByte String'");
        
        // Triple-quoted strings
        assert_parses("'''Triple quoted\nstring'''");
        
        // F-string with triple quotes
        assert_parses("f'''Multi-line\nf-string with {value}'''");
    }

    #[test]
    fn test_boundary_numbers() {
        // Zero
        assert_parses("0");
        
        // Maximum safe integer
        assert_parses("9223372036854775807");  // i64::MAX
        
        // Minimum safe integer
        assert_parses("-9223372036854775808");  // i64::MIN
        
        // Floating point precision
        assert_parses("0.1 + 0.2");
        
        // Scientific notation
        assert_parses("1.23e45");
        assert_parses("1.23e-45");
        
        // Different number bases
        assert_parses("0x123ABC");  // Hex
        assert_parses("0o123");     // Octal
        assert_parses("0b101010");  // Binary
    }
}
