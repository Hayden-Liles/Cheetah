use cheetah::lexer::Lexer;
use cheetah::parser::parse;

fn main() {
    // A simple test with multiple errors
    let source = r#"
def func(x y): # Missing comma
    print(x + y)

x = 10
y = 20
print(x + y)
    "#;
    
    println!("Testing error recovery with source:");
    println!("{}", source);
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    
    // Parse the tokens
    let result = parse(tokens);
    
    // We expect the parsing to fail
    if let Err(errors) = result {
        println!("Number of errors: {}", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);
        }
    } else {
        println!("Parsing succeeded unexpectedly!");
    }
}
