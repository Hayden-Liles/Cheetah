use cheetah::lexer::Lexer;
use cheetah::parser::parse;

#[test]
fn test_error_recovery() {
    // A simple test with multiple errors
    let source = r#"
def func(x y): # Missing comma
    print(x + y)

x = 10
y = 20
print(x + y)
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    
    // Parse the tokens
    let result = parse(tokens);
    
    // We expect the parsing to fail
    assert!(result.is_err(), "Parsing should fail");
    
    // But we should have collected multiple errors
    if let Err(errors) = result {
        println!("Number of errors: {}", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);
        }
        
        // We should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");
    }
}
