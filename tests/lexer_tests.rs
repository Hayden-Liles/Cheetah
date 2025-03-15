#[cfg(test)]
mod tests {
    use cheetah::lexer::{Lexer, LexerConfig, Token, TokenType};

    #[test]
    fn test_indentation() {
        // Input code with a function definition and one level of indentation
        let input = "def foo():\n    pass\n";

        // Initialize the lexer with the input
        let mut lexer = Lexer::new(input);
        
        // Tokenize the input
        let tokens = lexer.tokenize();

        // Define the expected token types in order
        let expected_types = vec![
            TokenType::Def,
            TokenType::Identifier("foo".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::Colon,
            TokenType::Newline,
            TokenType::Indent,
            TokenType::Pass,
            TokenType::Newline,
            TokenType::Dedent,
            TokenType::EOF,
        ];

        // Extract token types from the lexer's output
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();

        // Assert that the emitted tokens match the expected sequence
        assert_eq!(token_types, expected_types, "Token sequence does not match expected output");
    }

    // Helper function to simplify token comparison
    #[allow(dead_code)]
    fn assert_tokens(input: &str, expected_tokens: Vec<TokenType>) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), expected_tokens.len() + 1, "Token count mismatch"); // +1 for EOF
        
        for (i, expected_type) in expected_tokens.iter().enumerate() {
            assert_eq!(&tokens[i].token_type, expected_type, 
                        "Token type mismatch at position {}. Expected {:?}, got {:?}", 
                        i, expected_type, tokens[i].token_type);
        }
        
        // Check that the last token is EOF
        assert_eq!(tokens.last().unwrap().token_type, TokenType::EOF);
    }

    // Test for newline handling
    #[test]
    fn test_newline_styles() {
        let input_lf = "x = 1\ny = 2";
        let mut lexer_lf = Lexer::new(input_lf);
        let tokens_lf = lexer_lf.tokenize();
        
        let input_crlf = "x = 1\r\ny = 2";
        let mut lexer_crlf = Lexer::new(input_crlf);
        let tokens_crlf = lexer_crlf.tokenize();
        
        if tokens_lf.len() != tokens_crlf.len() {
            println!("LF tokens ({}): {:?}", tokens_lf.len(), tokens_lf);
            println!("CRLF tokens ({}): {:?}", tokens_crlf.len(), tokens_crlf);
        }
        
        assert_eq!(tokens_lf.len(), tokens_crlf.len(), "Different newline styles should produce same token count");
        for i in 0..tokens_lf.len() {
            assert_eq!(tokens_lf[i].token_type, tokens_crlf[i].token_type, 
                    "Different newline styles should produce same tokens");
        }
    }

    // Test for unicode support in strings and identifiers
    #[test]
    fn test_unicode_support() {
        // Testing indentation handling with a helper function
        fn assert_tokens_ignore_indentation(input: &str, expected_token_types: Vec<TokenType>) {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            
            // Filter out Indent and Dedent tokens
            let filtered_tokens: Vec<_> = tokens
                .into_iter()
                .filter(|token| !matches!(token.token_type, TokenType::Indent | TokenType::Dedent))
                .collect();
            
            assert_eq!(filtered_tokens.len(), expected_token_types.len() + 1, 
                    "Token count mismatch (ignoring indentation tokens)"); // +1 for EOF
            
            for (i, expected_type) in expected_token_types.iter().enumerate() {
                assert_eq!(&filtered_tokens[i].token_type, expected_type, 
                        "Token type mismatch at position {}. Expected {:?}, got {:?}", 
                        i, expected_type, filtered_tokens[i].token_type);
            }
            
            // Check that the last token is EOF
            assert_eq!(filtered_tokens.last().unwrap().token_type, TokenType::EOF);
        }

        // Unicode in identifiers
        assert_tokens_ignore_indentation(
            "π = 3.14159\nñame = \"José\"\n你好 = \"Hello\"",
            vec![
                TokenType::Identifier("π".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(3.14159),
                TokenType::Newline,
                TokenType::Identifier("ñame".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("José".to_string()),
                TokenType::Newline,
                TokenType::Identifier("你好".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("Hello".to_string()),
            ]
        );
        
        // Unicode in string literals
        assert_tokens_ignore_indentation(
            "message = \"Hello, 世界!\"",
            vec![
                TokenType::Identifier("message".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("Hello, 世界!".to_string()),
            ]
        );
        
        // Unicode escape sequences
        assert_tokens_ignore_indentation(
            r#"emoji = "\u{1F600}""#, // 😀 emoji
            vec![
                TokenType::Identifier("emoji".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("😀".to_string()),
            ]
        );
    }
    
    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::EOF);
    }
    
    // Test keywords
    #[test]
    fn test_keywords() {
        assert_tokens(
            "def if elif else while for in break continue pass return",
            vec![
                TokenType::Def,
                TokenType::If,
                TokenType::Elif,
                TokenType::Else,
                TokenType::While,
                TokenType::For,
                TokenType::In,
                TokenType::Break,
                TokenType::Continue,
                TokenType::Pass,
                TokenType::Return,
            ]
        );
        
        assert_tokens(
            "import from as True False None and or not",
            vec![
                TokenType::Import,
                TokenType::From,
                TokenType::As,
                TokenType::True,
                TokenType::False,
                TokenType::None,
                TokenType::And,
                TokenType::Or,
                TokenType::Not,
            ]
        );
        
        assert_tokens(
            "class with assert async await try except finally raise",
            vec![
                TokenType::Class,
                TokenType::With,
                TokenType::Assert,
                TokenType::Async,
                TokenType::Await,
                TokenType::Try,
                TokenType::Except,
                TokenType::Finally,
                TokenType::Raise,
            ]
        );
        
        assert_tokens(
            "lambda global nonlocal yield del is",
            vec![
                TokenType::Lambda,
                TokenType::Global,
                TokenType::Nonlocal,
                TokenType::Yield,
                TokenType::Del,
                TokenType::Is,
            ]
        );
    }
    
    // Test identifiers
    #[test]
    fn test_identifiers() {
        assert_tokens(
            "variable _private name123 camelCase snake_case",
            vec![
                TokenType::Identifier("variable".to_string()),
                TokenType::Identifier("_private".to_string()),
                TokenType::Identifier("name123".to_string()),
                TokenType::Identifier("camelCase".to_string()),
                TokenType::Identifier("snake_case".to_string()),
            ]
        );
        
        // Test identifier that looks like keyword but isn't
        assert_tokens(
            "defining ifdef",
            vec![
                TokenType::Identifier("defining".to_string()),
                TokenType::Identifier("ifdef".to_string()),
            ]
        );
    }
    
    // Test integer literals
    #[test]
    fn test_integer_literals() {
        assert_tokens(
            "123 0 -42 1_000_000",
            vec![
                TokenType::IntLiteral(123),
                TokenType::IntLiteral(0),
                TokenType::Minus,
                TokenType::IntLiteral(42),
                TokenType::IntLiteral(1000000),
            ]
        );
    }
    
    // Test different numeric bases
    #[test]
    fn test_different_bases() {
        assert_tokens(
            "0b1010 0B1100 0o777 0O123 0xABC 0Xdef",
            vec![
                TokenType::BinaryLiteral(10),
                TokenType::BinaryLiteral(12),
                TokenType::OctalLiteral(511), // 777 octal = 511 decimal
                TokenType::OctalLiteral(83),  // 123 octal = 83 decimal
                TokenType::HexLiteral(2748),  // ABC hex = 2748 decimal
                TokenType::HexLiteral(3567),  // def hex = 3567 decimal
            ]
        );
    }
    
    // Test float literals
    #[test]
    fn test_float_literals() {
        assert_tokens(
            "3.14 .5 2. 1e10 1.5e-5 1_000.5 1e+10",
            vec![
                TokenType::FloatLiteral(3.14),
                TokenType::FloatLiteral(0.5),
                TokenType::FloatLiteral(2.0),
                TokenType::FloatLiteral(1e10),
                TokenType::FloatLiteral(1.5e-5),
                TokenType::FloatLiteral(1000.5),
                TokenType::FloatLiteral(1e10),
            ]
        );
    }
    
    // Test string literals
    #[test]
    fn test_string_literals() {
        assert_tokens(
            r#""hello" 'world'"#,
            vec![
                TokenType::StringLiteral("hello".to_string()),
                TokenType::StringLiteral("world".to_string()),
            ]
        );
        
        // Test strings with escape sequences
        assert_tokens(
            r#""hello\nworld" 'escaped\'quote' "tab\tchar" 'bell\a'"#,
            vec![
                TokenType::StringLiteral("hello\nworld".to_string()),
                TokenType::StringLiteral("escaped'quote".to_string()),
                TokenType::StringLiteral("tab\tchar".to_string()),
                TokenType::StringLiteral("bell\u{0007}".to_string()),
            ]
        );
        
        // Test hex and Unicode escapes
        assert_tokens(
            r#""\x41\x42C" "\u00A9 copyright""#,
            vec![
                TokenType::StringLiteral("ABC".to_string()),
                TokenType::StringLiteral("© copyright".to_string()),
            ]
        );
    }
    
    // Test raw strings
    #[test]
    fn test_raw_strings() {
        assert_tokens(
            r#"r"raw\nstring" R'another\tone'"#,
            vec![
                TokenType::RawString("raw\\nstring".to_string()),
                TokenType::RawString("another\\tone".to_string()),
            ]
        );
    }
    
    // Test formatted strings (f-strings)
    #[test]
    fn test_formatted_strings() {
        assert_tokens(
            r#"f"Hello, {name}!" F'Value: {2 + 2}'"#,
            vec![
                TokenType::FString("Hello, {name}!".to_string()),
                TokenType::FString("Value: {2 + 2}".to_string()),
            ]
        );
        
        // Test nested expressions
        assert_tokens(
            r#"f"Nested: {value if condition else {inner}}""#,
            vec![
                TokenType::FString("Nested: {value if condition else {inner}}".to_string()),
            ]
        );
    }
    
    // Test bytes literals
    #[test]
    fn test_bytes_literals() {
        assert_tokens(
            r#"b"bytes" B'\x00\xff'"#,
            vec![
                TokenType::BytesLiteral(b"bytes".to_vec()),
                TokenType::BytesLiteral(vec![0, 255]),
            ]
        );
    }
    
    // Test triple-quoted strings
    #[test]
    fn test_triple_quoted_strings() {
        assert_tokens(
            r#""""Triple quoted string"""'''Another triple quoted'''"#,
            vec![
                TokenType::StringLiteral("Triple quoted string".to_string()),
                TokenType::StringLiteral("Another triple quoted".to_string()),
            ]
        );
        
        // Test with newlines inside
        assert_tokens(
            "\"\"\"Multi\nline\nstring\"\"\"",
            vec![
                TokenType::StringLiteral("Multi\nline\nstring".to_string()),
            ]
        );
    }
    
    // Test prefixed triple-quoted strings
    #[test]
    fn test_prefixed_triple_quoted_strings() {
        assert_tokens(
            r#"r"""Raw\nTriple"""f'''Format {x}'''"#,
            vec![
                TokenType::RawString("Raw\\nTriple".to_string()),
                TokenType::FString("Format {x}".to_string()),
            ]
        );
        
        assert_tokens(
            "b\"\"\"Bytes\nWith\nNewlines\"\"\"",
            vec![
                TokenType::BytesLiteral(b"Bytes\nWith\nNewlines".to_vec()),
            ]
        );
    }
    
    // Test operators
    #[test]
    fn test_basic_operators() {
        assert_tokens(
            "+ - * / % ** // @ & | ^ ~ << >>",
            vec![
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Multiply,
                TokenType::Divide,
                TokenType::Modulo,
                TokenType::Power,
                TokenType::FloorDivide,
                TokenType::At,
                TokenType::BitwiseAnd,
                TokenType::BitwiseOr,
                TokenType::BitwiseXor,
                TokenType::BitwiseNot,
                TokenType::ShiftLeft,
                TokenType::ShiftRight,
            ]
        );
    }
    
    // Test comparison operators
    #[test]
    fn test_comparison_operators() {
        assert_tokens(
            "== != < <= > >=",
            vec![
                TokenType::Equal,
                TokenType::NotEqual,
                TokenType::LessThan,
                TokenType::LessEqual,
                TokenType::GreaterThan,
                TokenType::GreaterEqual,
            ]
        );
    }
    
    // Test assignment operators
    #[test]
    fn test_assignment_operators() {
        assert_tokens(
            "= += -= *= /= %= **= //= &= |= ^= <<= >>=",
            vec![
                TokenType::Assign,
                TokenType::PlusAssign,
                TokenType::MinusAssign,
                TokenType::MulAssign,
                TokenType::DivAssign,
                TokenType::ModAssign,
                TokenType::PowAssign,
                TokenType::FloorDivAssign,
                TokenType::BitwiseAndAssign,
                TokenType::BitwiseOrAssign,
                TokenType::BitwiseXorAssign,
                TokenType::ShiftLeftAssign,
                TokenType::ShiftRightAssign,
            ]
        );
    }
    
    // Test special operators
    #[test]
    fn test_special_operators() {
        assert_tokens(
            ":= ...",
            vec![
                TokenType::Walrus,
                TokenType::Ellipsis,
            ]
        );
    }
    
    // Test delimiters
    #[test]
    fn test_delimiters() {
        assert_tokens(
            "( ) [ ] { } , . : ; -> \\",
            vec![
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBracket,
                TokenType::RightBracket,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Comma,
                TokenType::Dot,
                TokenType::Colon,
                TokenType::SemiColon,
                TokenType::Arrow,
                TokenType::BackSlash,
            ]
        );
    }
    
    // Test indentation
    #[test]
    fn test_indentation2() {
        let input = "def test():\n    print('indented')\n    if True:\n        print('nested')\n";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types
        let expected = vec![
            TokenType::Def,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::Colon,
            TokenType::Newline,
            TokenType::Indent,
            TokenType::Identifier("print".to_string()),
            TokenType::LeftParen,
            TokenType::StringLiteral("indented".to_string()),
            TokenType::RightParen,
            TokenType::Newline,
            TokenType::If,
            TokenType::True,
            TokenType::Colon,
            TokenType::Newline,
            TokenType::Indent,
            TokenType::Identifier("print".to_string()),
            TokenType::LeftParen,
            TokenType::StringLiteral("nested".to_string()),
            TokenType::RightParen,
            TokenType::Newline,
            TokenType::Dedent,
            TokenType::Dedent,
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Indentation tokens don't match expected");
    }
    
    #[test]
    fn test_complex_indentation2() {
        let input = "if x:\n    if y:\n        print('nested')\n    print('outer')\nprint('no indent')";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Count indents and dedents
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 2, "Should have 2 indents");
        assert_eq!(dedent_count, 2, "Should have 2 dedents");
    }
    
    // Test comments
    #[test]
    fn test_comments() {
        // Test inline comment
        let input = "x = 5 # comment\ny = 10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::new(TokenType::Identifier("x".to_string()), 1, 1, "x".to_string()),
            Token::new(TokenType::Assign, 1, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(5), 1, 5, "5".to_string()),
            Token::new(TokenType::Newline, 1, 16, "\n".to_string()), // Corrected to column 16
            Token::new(TokenType::Identifier("y".to_string()), 2, 1, "y".to_string()),
            Token::new(TokenType::Assign, 2, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(10), 2, 5, "10".to_string()),
            Token::new(TokenType::EOF, 2, 7, "".to_string()),
        ];
        assert_eq!(tokens, expected, "Inline comment not handled correctly");

        // Test standalone comment
        let input = "x = 5\n# comment\ny = 10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::new(TokenType::Identifier("x".to_string()), 1, 1, "x".to_string()),
            Token::new(TokenType::Assign, 1, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(5), 1, 5, "5".to_string()),
            Token::new(TokenType::Newline, 1, 6, "\n".to_string()),
            Token::new(TokenType::Newline, 2, 10, "\n".to_string()), // Corrected column
            Token::new(TokenType::Identifier("y".to_string()), 3, 1, "y".to_string()),
            Token::new(TokenType::Assign, 3, 3, "=".to_string()),
            Token::new(TokenType::IntLiteral(10), 3, 5, "10".to_string()),
            Token::new(TokenType::EOF, 3, 7, "".to_string()),
        ];
        assert_eq!(tokens, expected, "Standalone comment line not handled correctly");
    }
    
    // Test for line continuation
    #[test]
    fn test_line_continuation() {
        let input = "x = 1 + \\\n    2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types
        let expected = vec![
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(1),
            TokenType::Plus,
            TokenType::IntLiteral(2),
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Line continuation not handled correctly");
    }
    
    // Test for nested expressions
    #[test]
    fn test_nested_expressions() {
        let input = "result = (a + b) * (c - d) / (e ** f)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract the token types for easier comparison
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // Expected sequence of token types
        let expected = vec![
            TokenType::Identifier("result".to_string()),
            TokenType::Assign,
            TokenType::LeftParen,
            TokenType::Identifier("a".to_string()),
            TokenType::Plus,
            TokenType::Identifier("b".to_string()),
            TokenType::RightParen,
            TokenType::Multiply,
            TokenType::LeftParen,
            TokenType::Identifier("c".to_string()),
            TokenType::Minus,
            TokenType::Identifier("d".to_string()),
            TokenType::RightParen,
            TokenType::Divide,
            TokenType::LeftParen,
            TokenType::Identifier("e".to_string()),
            TokenType::Power,
            TokenType::Identifier("f".to_string()),
            TokenType::RightParen,
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Nested expressions not parsed correctly");
    }
    
    // Test for error handling
    #[test]
    fn test_unterminated_string() {
        let input = "\"unterminated";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check if we have an error token
        assert!(matches!(tokens[0].token_type, TokenType::Invalid(_)), 
                "Unterminated string should produce an Invalid token");
        assert_eq!(lexer.get_errors().len(), 1, "Should report exactly one error");
    }
    
    #[test]
    fn test_invalid_indentation() {
        let input = "def test():\n  print('indented')\n    print('invalid indent')";
        let mut lexer = Lexer::new(input);
        let _tokens = lexer.tokenize();
        
        // We should still get tokens, but there should be errors
        assert!(lexer.get_errors().len() > 0, "Should report indentation errors");
        
        // Find error about inconsistent indentation
        let has_indent_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("indentation") || e.message.contains("indent"));
        assert!(has_indent_error, "Should report an indentation-related error");
    }
    
    #[test]
    fn test_mixed_tabs_spaces() {
        let input = "def test():\n\t  print('mixed tabs and spaces')";
        let mut lexer = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: false,
            ..Default::default()
        });
        let _tokens = lexer.tokenize();
        
        // We should still get tokens, but there should be errors about mixed indentation
        assert!(lexer.get_errors().len() > 0, "Should report mixed indentation errors");
        
        // Find error about mixed tabs and spaces
        let has_mixed_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("Tabs are not allowed"));
        assert!(has_mixed_error, "Should report tabs in indentation error");        
    }
    
    #[test]
    fn test_invalid_number_format() {
        let input = "123.456.789";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check if we have an error token
        assert!(matches!(tokens[0].token_type, TokenType::Invalid(_)),
                "Invalid number format should produce an Invalid token");
        assert_eq!(lexer.get_errors().len(), 1, "Should report exactly one error");
    }
    
    // Test invalid escape sequences
    #[test]
    fn test_invalid_escape_sequences() {
        let input = r#""Invalid escape: \z""#;
        let mut lexer = Lexer::new(input);
        let _tokens = lexer.tokenize();
        
        // We should still get a string token, but there should be errors
        assert!(lexer.get_errors().len() > 0, "Should report escape sequence errors");
        
        let has_escape_error = lexer.get_errors().iter().any(|e| 
            e.message.contains("Unknown escape sequence"));
        assert!(has_escape_error, "Should report an escape sequence error");        
    }
    
    
    
    // Test line and column numbers
    #[test]
    fn test_position_tracking() {
        let input = "x = 1\ny = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Find the specific tokens we want to check
        let x_token = tokens.iter()
            .find(|t| matches!(&t.token_type, TokenType::Identifier(id) if id == "x"))
            .unwrap();
        let y_token = tokens.iter()
            .find(|t| matches!(&t.token_type, TokenType::Identifier(id) if id == "y"))
            .unwrap();
        
        // Check positions
        assert_eq!(x_token.line, 1, "x token should be on line 1");
        assert_eq!(x_token.column, 1, "x token should be at column 1");
        
        assert_eq!(y_token.line, 2, "y token should be on line 2");
        assert_eq!(y_token.column, 1, "y token should be at column 1");
    }
    
    // Test ignoring newlines inside parentheses, brackets, and braces
    #[test]
    fn test_newlines_in_groupings() {
        let input = "func(\n    arg1,\n    arg2\n)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract token types
        let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
        
        // No newline tokens should appear between parentheses
        let expected = vec![
            TokenType::Identifier("func".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("arg1".to_string()),
            TokenType::Comma,
            TokenType::Identifier("arg2".to_string()),
            TokenType::RightParen,
            TokenType::EOF,
        ];
        
        assert_eq!(token_types, expected, "Newlines in groupings not handled correctly");
    }
    
    // Test indentation with empty lines
    #[test]
    fn test_empty_lines() {
        let input = "def test():\n    print('line 1')\n\n    print('line 2')";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Empty lines shouldn't affect indentation
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 1, "Should have 1 indent");
        assert_eq!(dedent_count, 1, "Should have 1 dedent");
    }
    
    // Test for custom lexer config
    #[test]
    fn test_custom_lexer_config() {
        let input = "def test():\n\tprint('using tabs')";
        
        // Default config doesn't allow tabs
        let mut lexer1 = Lexer::new(input);
        let _tokens1 = lexer1.tokenize();
        assert!(lexer1.get_errors().len() > 0, "Default config should report tab errors");
        
        // Custom config allows tabs
        let mut lexer2 = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: true,
            tab_width: 4,
            ..Default::default()
        });
        let _tokens2 = lexer2.tokenize();
        assert_eq!(lexer2.get_errors().len(), 0, "Custom config should allow tabs");
    }
    
    // Test for a comprehensive real-world code example
    #[test]
    fn test_comprehensive_code() {
        let input = r#"
def factorial(n):
    """
    Calculate the factorial of a number.
    
    Args:
        n: A positive integer
        
    Returns:
        The factorial of n
    """
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

class Calculator:
    def __init__(self, value=0):
        self.value = value
    
    def add(self, x):
        self.value += x
        return self
    
    def multiply(self, x):
        self.value *= x
        return self

# Test the calculator
calc = Calculator(5)
result = calc.add(3).multiply(2).value
print(f"Result: {result}")  # Should be 16

# Binary, octal, and hex examples
binary = 0b1010  # 10
octal = 0o777   # 511
hexa = 0xABC    # 2748

# Raw string and bytes
raw_data = r"C:\Users\path\to\file"
bytes_data = b"\x00\x01\x02"
"#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have a lot of tokens and no errors
        assert!(tokens.len() > 50, "Comprehensive example should produce many tokens");
        assert_eq!(lexer.get_errors().len(), 0, "Comprehensive example should not have errors");
        
        // Check a few key tokens to ensure it parsed correctly
        let has_def = tokens.iter().any(|t| t.token_type == TokenType::Def);
        let has_class = tokens.iter().any(|t| t.token_type == TokenType::Class);
        let has_docstring = tokens.iter().any(|t| 
            matches!(&t.token_type, TokenType::StringLiteral(s) if s.contains("Calculate the factorial")));
        
        assert!(has_def, "Should have 'def' tokens");
        assert!(has_class, "Should have 'class' tokens");
        assert!(has_docstring, "Should have docstring token");
    }

    // Add these test functions to your lexer.rs file's tests module

    #[test]
    fn test_invalid_identifiers() {
        let input = "123abc = 5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // The lexer should tokenize this as IntLiteral(123) followed by Identifier("abc"), 
        // not as an Invalid token
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(123), "Should recognize 123 as an integer");
        assert_eq!(tokens[1].token_type, TokenType::Identifier("abc".to_string()), "Should recognize abc as an identifier");
    }

    // Test edge cases for indentation with empty lines and comments
    #[test]
    fn test_indentation_edge_cases() {
        // Test empty lines within indented blocks
        let input = "def test():\n    line1()\n\n    # Comment\n\n    line2()";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Count indents and dedents
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 1, "Should have 1 indent");
        assert_eq!(dedent_count, 1, "Should have 1 dedent");
        
        // Find the line2 token
        let line2 = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "line2")
        ).unwrap();
        
        // It should have proper indentation (same as line1)
        let line1 = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "line1")
        ).unwrap();
        
        assert_eq!(line2.column, line1.column, "line2 should have same indentation as line1");
    }

    #[test]
    fn test_walrus_operator() {
        assert_tokens(
            "if (n := len(items)) > 0: print(n)",
            vec![
                TokenType::If,
                TokenType::LeftParen,
                TokenType::Identifier("n".to_string()),
                TokenType::Walrus,
                TokenType::Identifier("len".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier("items".to_string()),
                TokenType::RightParen,
                TokenType::RightParen,
                TokenType::GreaterThan,
                TokenType::IntLiteral(0),
                TokenType::Colon,
                TokenType::Identifier("print".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier("n".to_string()),
                TokenType::RightParen,
            ]
        );
    }

    // Test for handling mixed line endings
    #[test]
    fn test_mixed_line_endings() {
        let input = "x = 1\ny = 2\r\nz = 3\n";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have 3 lines with proper line numbers
        let x_token = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "x")).unwrap();
        let y_token = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "y")).unwrap();
        let z_token = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "z")).unwrap();
        
        assert_eq!(x_token.line, 1, "x should be on line 1");
        assert_eq!(y_token.line, 2, "y should be on line 2");
        assert_eq!(z_token.line, 3, "z should be on line 3");
    }

    // Test for handling line continuation in different contexts
    #[test]
    fn test_line_continuation_contexts() {
        // Line continuation in lists
        assert_tokens(
            "items = [\n    1,\n    2,\n    3\n]",
            vec![
                TokenType::Identifier("items".to_string()),
                TokenType::Assign,
                TokenType::LeftBracket,
                TokenType::IntLiteral(1),
                TokenType::Comma,
                TokenType::IntLiteral(2),
                TokenType::Comma,
                TokenType::IntLiteral(3),
                TokenType::RightBracket,
            ]
        );
        
        // Line continuation in function calls
        assert_tokens(
            "result = func(\n    arg1,\n    arg2\n)",
            vec![
                TokenType::Identifier("result".to_string()),
                TokenType::Assign,
                TokenType::Identifier("func".to_string()),
                TokenType::LeftParen,
                TokenType::Identifier("arg1".to_string()),
                TokenType::Comma,
                TokenType::Identifier("arg2".to_string()),
                TokenType::RightParen,
            ]
        );
        
        // Explicit line continuation with backslash
        assert_tokens(
            "result = 1 + \\\n    2 + \\\n    3",
            vec![
                TokenType::Identifier("result".to_string()),
                TokenType::Assign,
                TokenType::IntLiteral(1),
                TokenType::Plus,
                TokenType::IntLiteral(2),
                TokenType::Plus,
                TokenType::IntLiteral(3),
            ]
        );
    }

    // Test for position tracking with nested structures
    #[test]
    fn test_position_tracking_nested() {
        let input = "nested = [(1, 2), (3, 4)]";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Find the specific nested tokens
        let left_bracket = tokens.iter().find(|t| t.token_type == TokenType::LeftBracket).unwrap();
        let first_paren = tokens.iter().find(|t| t.token_type == TokenType::LeftParen).unwrap();
        let second_paren = tokens.iter().filter(|t| t.token_type == TokenType::LeftParen).nth(1).unwrap();
        
        // Check relative positions
        assert!(left_bracket.column < first_paren.column, "Left bracket should be before first parenthesis");
        assert!(first_paren.column < second_paren.column, "First parenthesis should be before second parenthesis");
    }


    // Test edge cases for number formats
    #[test]
    fn test_number_format_edge_cases() {
        // Test scientific notation edge cases
        assert_tokens(
            "a = 1e10\nb = 1.5e+20\nc = 1.5e-10\nd = .5e3",
            vec![
                TokenType::Identifier("a".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(1e10),
                TokenType::Newline,
                TokenType::Identifier("b".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(1.5e20),
                TokenType::Newline,
                TokenType::Identifier("c".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(1.5e-10),
                TokenType::Newline,
                TokenType::Identifier("d".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(0.5e3),
            ]
        );
        
        // Test number format errors
        let input = "good = 123\nbad = 123.456.789\nrecovered = 42";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have an error for the invalid number
        assert!(!lexer.get_errors().is_empty(), "Should detect invalid number format");
        
        // But we should still tokenize valid content after the error
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "recovered")
        );
        assert!(recovered.is_some(), "Lexer should recover and find tokens after the error");
    }

    // Test for recovery from errors
    #[test]
    fn test_error_recovery() {
        // Test recovery from unterminated string
        let input = r#"good = "valid"
    bad = "unterminated
    recovered = 42"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have at least one error
        assert!(!lexer.get_errors().is_empty(), "Should detect unterminated string error");
        
        // But we should also have valid tokens after the error
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "recovered")
        );
        assert!(recovered.is_some(), "Lexer should recover and find tokens after the error");
        
        // Test recovery from invalid indentation
        let input = "def test():\n   print('3 spaces')\n      print('6 spaces')\nprint('valid')";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // We should have at least one error
        assert!(!lexer.get_errors().is_empty(), "Should detect indentation error");
        
        // But we should also have valid tokens after the error
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "print") && 
            t.lexeme == "print" && 
            t.line > 3
        );
        assert!(recovered.is_some(), "Lexer should recover and find tokens after the indentation error");
    }

        // Test for complex line continuation
    #[test]
    fn test_complex_line_continuation() {
        assert_tokens(
            "long_string = \"This is a very \\\n    long string that \\\n    spans multiple lines\"",
            vec![
                TokenType::Identifier("long_string".to_string()),
                TokenType::Assign,
                TokenType::StringLiteral("This is a very long string that spans multiple lines".to_string()),
            ]
        );
        
        // Test line continuation inside expressions
        assert_tokens(
            "result = (1 + \\\n          2) * \\\n         3",
            vec![
                TokenType::Identifier("result".to_string()),
                TokenType::Assign,
                TokenType::LeftParen,
                TokenType::IntLiteral(1),
                TokenType::Plus,
                TokenType::IntLiteral(2),
                TokenType::RightParen,
                TokenType::Multiply,
                TokenType::IntLiteral(3),
            ]
        );
    }

    // Test for complex nested structures
    #[test]
    fn test_complex_nesting() {
        assert_tokens(
            "x = [1, (2, 3), {'a': 4, 'b': [5, 6]}]",
            vec![
                TokenType::Identifier("x".to_string()),
                TokenType::Assign,
                TokenType::LeftBracket,
                TokenType::IntLiteral(1),
                TokenType::Comma,
                TokenType::LeftParen,
                TokenType::IntLiteral(2),
                TokenType::Comma,
                TokenType::IntLiteral(3),
                TokenType::RightParen,
                TokenType::Comma,
                TokenType::LeftBrace,
                TokenType::StringLiteral("a".to_string()),
                TokenType::Colon,
                TokenType::IntLiteral(4),
                TokenType::Comma,
                TokenType::StringLiteral("b".to_string()),
                TokenType::Colon,
                TokenType::LeftBracket,
                TokenType::IntLiteral(5),
                TokenType::Comma,
                TokenType::IntLiteral(6),
                TokenType::RightBracket,
                TokenType::RightBrace,
                TokenType::RightBracket,
            ]
        );
    }

    // Test for string escape edge cases
    #[test]
    fn test_string_escape_edge_cases() {
        // Test octal escapes
        assert_tokens(
            r#""\1\22\377""#,
            vec![
                TokenType::StringLiteral("\u{0001}\u{0012}\u{00FF}".to_string()),
            ]
        );
        
        // Test Unicode escapes
        assert_tokens(
            r#""\u00A9\u2764\u{1F600}""#, // copyright, heart, smile emoji
            vec![
                TokenType::StringLiteral("©❤😀".to_string()),
            ]
        );
        
        // Test raw strings with backslashes and quotes
        assert_tokens(
            r#"r"C:\path\to\file" r'\'quoted\''"#,
            vec![
                TokenType::RawString(r"C:\path\to\file".to_string()),
                TokenType::RawString(r"\'quoted\'".to_string()),
            ]
        );
    }

        // Test for complex indentation patterns
    #[test]
    fn test_complex_indentation() {
        let input = "def outer():\n    if condition:\n        nested()\n    else:\n        if another:\n            deep_nested()\n        result = 42\n    return result";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Count indents and dedents
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        
        assert_eq!(indent_count, 4, "Should have 4 indents");
        assert_eq!(dedent_count, 4, "Should have 4 dedents");
    }

    // Test for numeric separators
    #[test]
    fn test_numeric_separators() {
        assert_tokens(
            "a = 1_000_000\nb = 0b1010_1010\nc = 0o777_333\nd = 0xFF_FF_FF\ne = 3.14_15_92",
            vec![
                TokenType::Identifier("a".to_string()),
                TokenType::Assign,
                TokenType::IntLiteral(1000000),
                TokenType::Newline,
                TokenType::Identifier("b".to_string()),
                TokenType::Assign,
                TokenType::BinaryLiteral(170), // 0b10101010
                TokenType::Newline,
                TokenType::Identifier("c".to_string()),
                TokenType::Assign,
                TokenType::OctalLiteral(261851), // 0o777333
                TokenType::Newline,
                TokenType::Identifier("d".to_string()),
                TokenType::Assign,
                TokenType::HexLiteral(16777215), // 0xFFFFFF
                TokenType::Newline,
                TokenType::Identifier("e".to_string()),
                TokenType::Assign,
                TokenType::FloatLiteral(3.141592),
            ]
        );
    }

        // Test for complex comments and docstrings
    #[test]
    fn test_comments_and_docstrings() {
        // Test inline comments
        let input = "x = 5 # This is a comment\ny = 10 # Another comment";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 8, "Should have 7 tokens plus EOF"); // x = 5 \n y = 10 EOF
        
        // Test docstrings (triple-quoted strings)
        let input = "def func():\n    \"\"\"This is a docstring.\n    Multi-line.\n    \"\"\"\n    pass";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Extract just the docstring
        let docstring = tokens.iter().find_map(|t| {
            if let TokenType::StringLiteral(s) = &t.token_type {
                Some(s.as_str())
            } else {
                None
            }
        });
        
        assert_eq!(docstring, Some("This is a docstring.\n    Multi-line.\n    "), "Docstring not parsed correctly");
    }

        // Test for f-string variants and edge cases
    #[test]
    fn test_fstring_variants() {
        // Test basic f-string
        assert_tokens(
            r#"f"Hello, {name}!""#,
            vec![
                TokenType::FString("Hello, {name}!".to_string()),
            ]
        );
        
        // Test nested expressions in f-strings
        assert_tokens(
            r#"f"Value: {2 + 3 * {4 + 5}}""#,
            vec![
                TokenType::FString("Value: {2 + 3 * {4 + 5}}".to_string()),
            ]
        );
        
        // Test f-string with dictionary unpacking
        assert_tokens(
            r#"f"Items: {', '.join(f'{k}={v}' for k, v in items.items())}""#,
            vec![
                TokenType::FString("Items: {', '.join(f'{k}={v}' for k, v in items.items())}".to_string()),
            ]
        );
        
        // Test triple-quoted f-strings
        let input = "f\"\"\"Name: {name}\nAge: {age}\n\"\"\"";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[0].token_type, TokenType::FString(_)), 
                "Triple-quoted f-string should be recognized as an FString token");
    }

    #[test]
    fn test_recovery_after_deep_indentation_error() {
        let input = "def outer():\n    if x:\n        nested()\n   bad_indent()\n    recovered()";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(lexer.get_errors().len() > 0, "Should report indentation error");
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "recovered")
        );
        assert!(recovered.is_some(), "Should recover after indentation error");
    }

    #[test]
    fn test_large_string_literal() {
        let large_string = "a".repeat(10_000);
        let input = format!("\"{}\"", large_string);
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 2, "Should have StringLiteral and EOF");
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral(large_string), 
                    "Should handle large string correctly");
        assert_eq!(lexer.get_errors().len(), 0, "Should process large string without errors");
    }

    #[test]
    fn test_deep_nesting() {
        let input = "(".repeat(1000) + &")".repeat(1000);
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens.len(), 2001, "Should have 2000 tokens plus EOF");
        assert_eq!(lexer.get_errors().len(), 0, "Should handle deep nesting without errors");
    }

        #[test]
    fn test_leading_zeros_in_decimal() {
        let input = "x = 0123";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[2].token_type, TokenType::IntLiteral(123), 
                    "Should parse 0123 as 123, treating leading zero as insignificant");
        // Note: If your lexer should reject leading zeros, replace with Invalid token check
    }

    #[test]
    fn test_standalone_backslash() {
        let input = "x = 1 \\ y = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        let backslash_idx = tokens.iter().position(|t| t.token_type == TokenType::BackSlash).unwrap();
        assert_eq!(tokens[backslash_idx + 1].token_type, TokenType::Identifier("y".to_string()), 
                    "Should tokenize content after standalone backslash");
    }

    #[test]
    fn test_ellipsis_vs_dots() {
        assert_tokens(
            "x = ... y = .. z = . . .",
            vec![
                TokenType::Identifier("x".to_string()),
                TokenType::Assign,
                TokenType::Ellipsis,
                TokenType::Identifier("y".to_string()),
                TokenType::Assign,
                TokenType::Dot,
                TokenType::Dot,
                TokenType::Identifier("z".to_string()),
                TokenType::Assign,
                TokenType::Dot,
                TokenType::Dot,
                TokenType::Dot,
            ]
        );
    }

    #[test]
    fn test_surrogate_pairs() {
        assert_tokens(
            r#""\U0001F600""#, // 😀 emoji (requires surrogate pair in UTF-16)
            vec![
                TokenType::StringLiteral("😀".to_string()),
            ]
        );
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let input = r#""\u12""#; // Incomplete Unicode escape
        let mut lexer = Lexer::new(input);
        let _tokens = lexer.tokenize();
        
        assert_eq!(lexer.get_errors().len(), 1, "Should report an error for invalid Unicode escape");
    }

        #[test]
    fn test_mixed_tabs_and_spaces_with_recovery() {
        let input = "def test():\n    print('ok')\n\t  print('mixed')\n    print('recovered')";
        let mut lexer = Lexer::with_config(input, LexerConfig {
            allow_tabs_in_indentation: false,
            ..Default::default()
        });
        let tokens = lexer.tokenize();
        
        assert!(lexer.get_errors().len() > 0, "Should report mixed indentation error");
        let recovered = tokens.iter().find(|t| 
            matches!(&t.token_type, TokenType::Identifier(s) if s == "print") && 
            t.lexeme == "print" && 
            t.line == 4
        );
        assert!(recovered.is_some(), "Should recover and tokenize after mixed indentation");
    }

    #[test]
    fn test_indentation_with_comments_and_empty_lines() {
        let input = "def func():\n    x = 1\n\n    # Comment\n\n    y = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
        let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
        assert_eq!(indent_count, 1, "Should have 1 indent");
        assert_eq!(dedent_count, 1, "Should have 1 dedent");
    }

    #[test]
    fn test_unterminated_triple_quoted_string() {
        let input = r#"x = """incomplete docstring"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[2].token_type, TokenType::Invalid(_)), 
                "Unterminated triple-quoted string should produce an Invalid token");
        assert_eq!(lexer.get_errors().len(), 1, "Should report one error for unterminated string");
    }

    #[test]
    fn test_escaped_quotes_in_single_quoted_string() {
        assert_tokens(
            r#"'He said \"Hello\"'"#,
            vec![
                TokenType::StringLiteral("He said \"Hello\"".to_string()),
            ]
        );
    }

    #[test]
    fn test_string_with_line_continuation() {
        let input = "\"Line split \\\n    here\"";
        assert_tokens(input, vec![TokenType::StringLiteral("Line split here".to_string())]);
    }

    #[test]
    fn test_cr_only_newlines() {
        let input = "x = 1\ry = 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let x = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "x")).unwrap();
        let y = tokens.iter().find(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "y")).unwrap();
        assert_eq!(x.line, 1, "x should be on line 1");
        assert_eq!(y.line, 2, "y should be on line 2: {:?}", tokens);
    }

    #[test]
    fn test_multiline_comments() {
        let input = "# Line 1\n# Line 2\nx = 1";
        assert_tokens(input, vec![
            TokenType::Newline,
            TokenType::Newline,
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(1),
        ]);
    }

    #[test]
    fn test_float_with_underscore_in_exponent() {
        let input = "x = 1.5e_10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[2].token_type, TokenType::Invalid(_)), "Expected Invalid: {:?}", tokens[2]);
        assert_eq!(lexer.get_errors().len(), 1, "Errors: {:?}", lexer.get_errors());
    }

    #[test]
    fn test_mixed_newline_styles() {
        let input = "a = 1\nb = 2\r\nc = 3\rd = 4";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let lines: Vec<_> = tokens.iter()
            .filter(|t| matches!(&t.token_type, TokenType::Identifier(_)))
            .map(|t| t.line)
            .collect();
        assert_eq!(lines, vec![1, 2, 3, 4], "Line numbers: {:?}", lines);
    }

    #[test]
    fn test_comment_after_line_continuation() {
        let input = "x = 1 + \\\n# Comment\n    2";
        assert_tokens(input, vec![
            TokenType::Identifier("x".to_string()),
            TokenType::Assign,
            TokenType::IntLiteral(1),
            TokenType::Plus,
            TokenType::IntLiteral(2),
        ]);
    }

    #[test]
    fn test_multiple_errors_one_line() {
        let input = "x = \"unterminated\\z 123.456.789";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(lexer.get_errors().len() >= 2, "Expected 2+ errors, got: {:?}", lexer.get_errors());
        assert!(tokens.iter().any(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "x")));
    }

    #[test]
fn test_match_and_case_keywords() {
    // Test basic match/case structure
    let input = "
match value:
    case 1:
        print('one')
    case 2:
        print('two')
    case _:
        print('default')
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    // Extract the token types for easier comparison
    let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
    
    // Find the match token
    let match_token = tokens.iter().find(|t| 
        matches!(&t.token_type, TokenType::Match)
    ).unwrap();
    
    // Find all case tokens
    let case_tokens: Vec<&Token> = tokens.iter().filter(|t| 
        matches!(t.token_type, TokenType::Case)
    ).collect();
    
    // Verify match and case are recognized as keywords
    assert_eq!(match_token.lexeme, "match");
    assert_eq!(case_tokens.len(), 3); // Should find 3 case keywords
    
    // Verify correct tokenization of the rest of the structure
    let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::Dedent)).count();
    
    // In the Python indentation model, there are 4 indentation levels here:
    // 1. The initial level at "match value:"
    // 2. The indentation for each "case" statement
    // 3. The further indentation for each "print" statement
    // 4. Plus another level that captures the end of indented blocks
    assert_eq!(indent_count, 4, "Should have 4 indentation levels");
    assert_eq!(dedent_count, 4, "Should have 4 dedentation levels");
}

#[test]
fn test_complex_pattern_matching() {
    // Test more complex pattern matching syntax
    let input = "
match point:
    case (0, 0):
        print('Origin')
    case (0, y):
        print(f'Y={y}')
    case (x, 0):
        print(f'X={x}')
    case (x, y) if x == y:
        print(f'X=Y={x}')
    case (x, y):
        print(f'X={x}, Y={y}')
    case _:
        print('Not a point')
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    // Verify complex pattern matching tokens
    let case_count = tokens.iter().filter(|t| 
        matches!(t.token_type, TokenType::Case)
    ).count();
    
    assert_eq!(case_count, 6, "Should have 6 case patterns");
    
    // Check for parentheses in pattern matching
    let left_paren_count = tokens.iter().filter(|t| 
        matches!(t.token_type, TokenType::LeftParen)
    ).count();
    
    let right_paren_count = tokens.iter().filter(|t| 
        matches!(t.token_type, TokenType::RightParen)
    ).count();
    
    assert_eq!(left_paren_count, right_paren_count, "Parentheses should be balanced");
    assert!(left_paren_count >= 5, "Should have at least 5 sets of parentheses for patterns");
    
    // Check for if guard in pattern matching
    let if_in_case = tokens.iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.token_type, TokenType::If))
        .any(|(i, _)| {
            // Check if there's a case keyword before this if
            tokens[..i].iter().rev().any(|t| 
                matches!(t.token_type, TokenType::Case)
            )
        });
    
    assert!(if_in_case, "Should have at least one if guard in a case pattern");
}

#[test]
fn test_walrus_operator_in_patterns() {
    // Test walrus operator in pattern context
    let input = "
match data:
    case [x, y] if (z := x + y) > 10:
        print(f'Sum {z} exceeds 10')
    case [x, y] if (z := x + y) <= 10:
        print(f'Sum {z} is 10 or less')
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    // Find walrus operators
    let walrus_count = tokens.iter().filter(|t| 
        matches!(t.token_type, TokenType::Walrus)
    ).count();
    
    assert_eq!(walrus_count, 2, "Should find 2 walrus operators");
    
    // Make sure they're in the right context (after case patterns)
    let case_indices: Vec<usize> = tokens.iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.token_type, TokenType::Case))
        .map(|(i, _)| i)
        .collect();
    
    let walrus_indices: Vec<usize> = tokens.iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.token_type, TokenType::Walrus))
        .map(|(i, _)| i)
        .collect();
    
    // For each walrus operator, check that there's a case keyword before it
    for walrus_idx in &walrus_indices {
        let has_case_before = case_indices.iter().any(|case_idx| case_idx < walrus_idx);
        assert!(has_case_before, "Walrus operator should appear after a case keyword");
    }
}

#[test]
fn test_nested_match_statements() {
    // Test nested match statements
    let input = "
def process(data):
    match data:
        case {'type': 'user', 'info': info}:
            match info:
                case {'level': level} if level > 5:
                    print('High level user')
                case {'level': _}:
                    print('Regular user')
                case _:
                    print('Unknown user type')
        case {'type': 'admin'}:
            print('Admin access')
        case _:
            print('Unknown data format')
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    // Find all match keywords
    let match_tokens: Vec<&Token> = tokens.iter().filter(|t| 
        matches!(t.token_type, TokenType::Match)
    ).collect();
    
    assert_eq!(match_tokens.len(), 2, "Should find 2 match keywords");
    
    // Check indentation nesting
    let indent_indices: Vec<usize> = tokens.iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.token_type, TokenType::Indent))
        .map(|(i, _)| i)
        .collect();
    
    let dedent_indices: Vec<usize> = tokens.iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.token_type, TokenType::Dedent))
        .map(|(i, _)| i)
        .collect();
    
    assert_eq!(indent_indices.len(), dedent_indices.len(), 
               "Indentation and dedentation should be balanced");
    assert!(indent_indices.len() >= 3, "Should have at least 3 levels of indentation");
}

#[test]
fn test_structural_pattern_matching() {
    // Test structural pattern matching with complex patterns
    let input = "
match command:
    case ['quit']:
        print('Exiting')
    case ['load', filename]:
        print(f'Loading {filename}')
    case ['save', filename]:
        print(f'Saving {filename}')
    case ['search', *keywords]:
        print(f'Searching for {keywords}')
    case ['filter', name, *args, **kwargs]:
        print(f'Filtering by {name}')
    case _:
        print('Unknown command')
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    // Find all case patterns
    let case_tokens: Vec<&Token> = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::Case))
        .collect();
    
    assert_eq!(case_tokens.len(), 6, "Should have 6 case patterns");
    
    // Check for pattern elements
    let left_bracket_count = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::LeftBracket))
        .count();
    
    let right_bracket_count = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::RightBracket))
        .count();
    
    assert_eq!(left_bracket_count, right_bracket_count, 
              "Should have balanced brackets for list patterns");
    assert!(left_bracket_count >= 5, "Should have at least 5 list patterns");
    
    // Check for star expressions in patterns
    let multiply_tokens = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::Multiply))
        .count();
    
    assert!(multiply_tokens >= 2, "Should have at least 2 star expressions in patterns");
    
    // Check for double star expressions
    let power_tokens = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::Power))
        .count();
    
    assert!(power_tokens >= 1, "Should have at least 1 double star expression in patterns");
}

#[test]
fn test_advanced_indent_consistency() {
    // Test various indentation patterns and edge cases
    let input = "
def func1():
    # 4 spaces
    print('Level 1')
    
    if condition:
        # 8 spaces
        print('Level 2')
        
        for item in items:
            # 12 spaces
            print('Level 3')
            
            # Empty lines shouldn't affect indentation

            # Comment at same indentation
            print('Still level 3')
            
        # Back to 8 spaces
        print('Back to level 2')
        
    # Back to 4 spaces
    print('Back to level 1')

# No indentation
print('No indentation')
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    // Extract newlines and indentation changes
    let structural_tokens: Vec<TokenType> = tokens.iter()
        .filter(|t| matches!(t.token_type, 
                            TokenType::Indent | 
                            TokenType::Dedent | 
                            TokenType::Newline))
        .map(|t| t.token_type.clone())
        .collect();
    
    // Count indentation levels
    let indent_count = structural_tokens.iter()
        .filter(|t| matches!(t, TokenType::Indent))
        .count();
    
    let dedent_count = structural_tokens.iter()
        .filter(|t| matches!(t, TokenType::Dedent))
        .count();
    
    assert_eq!(indent_count, dedent_count, 
              "Should have balanced indentation (equal indents and dedents)");
    assert_eq!(indent_count, 3, "Should have exactly 3 indentation levels");
    
    // Verify print statements at different levels
    let print_tokens: Vec<&Token> = tokens.iter()
        .filter(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "print"))
        .collect();
    
    // Corrected the expected count to 7 print statements
    assert_eq!(print_tokens.len(), 7, "Should have 7 print statements");
    
    // Check that line numbers are increasing
    let mut prev_line = 0;
    for token in print_tokens {
        assert!(token.line > prev_line, 
               "Line numbers should be strictly increasing");
        prev_line = token.line;
    }
}
}