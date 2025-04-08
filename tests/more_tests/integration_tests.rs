#[cfg(test)]
mod integration_tests {
    use cheetah::lexer::Lexer;
    use cheetah::parser::Parser;
    use cheetah::formatter::CodeFormatter;
    use cheetah::visitor::Visitor;
    use cheetah::symtable::SymbolTableBuilder;

    // Helper function to parse and format code
    fn parse_and_format(source: &str, indent_size: usize) -> Result<String, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        println!("Tokens: {:?}", tokens);
        if !lexer.get_errors().is_empty() {
            return Err(format!("Lexer errors: {:?}", lexer.get_errors()));
        }
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(module) => {
                println!("AST: {:?}", module.body);
                let mut formatter = CodeFormatter::new(indent_size);
                formatter.visit_module(&module);
                Ok(formatter.get_output().to_string())
            },
            Err(errors) => {
                println!("Parse errors: {:?}", errors);
                Err(format!("Parser errors: {:?}", errors))
            },
        }
    }

    // Helper function to parse and build symbol table
    fn parse_and_analyze(source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        if !lexer.get_errors().is_empty() {
            return Err(format!("Lexer errors: {:?}", lexer.get_errors()));
        }
        
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(module) => {
                let mut symbol_table = SymbolTableBuilder::new();
                symbol_table.visit_module(&module);
                
                Ok(())
            },
            Err(errors) => Err(format!("Parser errors: {:?}", errors)),
        }
    }

    #[test]
    fn test_parse_format_roundtrip() {
        let source = "
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

# Print the first 10 Fibonacci numbers
for i in range(10):
    print(fibonacci(i))
";
        
        let formatted = parse_and_format(source, 4).unwrap();
        // Parse the formatted code again to ensure it's valid
        let reparsed = parse_and_format(&formatted, 4).unwrap();
        
        // The second formatting should be idempotent
        assert_eq!(formatted, reparsed);
    }

    #[test]
    fn test_parse_analyze_success() {
        let source = "
def calculate_sum(numbers):
    total = 0
    for num in numbers:
        total += num
    return total

result = calculate_sum([1, 2, 3, 4, 5])
print(\"Sum:\", result)
";
        
        // Should analyze without errors
        assert!(parse_and_analyze(source).is_ok());
    }

    #[test]
    fn test_parse_analyze_undefined_variable() {
        let source = "
def calculate_sum(numbers):
    total = 0
    for num in numbers:
        total += num
    return total

# Uses undefined variable 'nums'
result = calculate_sum(nums)
print(\"Sum:\", result)
";
        
        // Should analyze without errors (symtable builder doesn't fail on undefined variables)
        assert!(parse_and_analyze(source).is_ok());
        
        // In a real test, you might check that 'nums' is reported as undefined
        // but this depends on how you want to handle undefined variables
    }

    #[test]
    fn test_lexer_parser_integration() {
        // Test various tokens and syntax elements
        let source = "
x = 42
y = 3.14
s = \"Hello, world!\"
b = True
n = None
lst = [1, 2, 3]
tup = (4, 5, 6)
dict = {\"key\": \"value\"}
";
        
        // Should parse without errors
        assert!(parse_and_format(source, 4).is_ok());
    }

    #[test]
    fn test_complex_expressions_breakdown() {
        // Test each expression individually for better debugging
        let source1 = "result = (1 + 2) * 3 ** 2 // 4 % 3";
        println!("Testing arithmetic expression...");
        assert!(parse_and_format(source1, 4).is_ok());
        
        let source2 = "condition = not (a > b and c <= d or e != f)";
        println!("Testing boolean expression...");
        assert!(parse_and_format(source2, 4).is_ok());
        
        let source3 = "values = [x*y for x in range(5) for y in range(5) if (x + y) % 2 == 0]";
        println!("Testing list comprehension...");
        assert!(parse_and_format(source3, 4).is_ok());
    }

    #[test]
    fn test_syntax_errors_are_detected() {
        // Missing closing parenthesis
        let source1 = "print(\"Hello, world!\"";
        assert!(parse_and_format(source1, 4).is_err());
        
        // Invalid indentation
        let source2 = "def test():\n  pass\n    pass";
        assert!(parse_and_format(source2, 4).is_err());
        
        // Invalid syntax in expression
        let source3 = "x = 1 +* 2";
        assert!(parse_and_format(source3, 4).is_err());
    }

    #[test]
    fn test_all_statement_types() {
        let source = "
# Function definition
def func(a, b=5):
    return a + b

# Class definition
class Test:
    x = 1
    
    def method(self):
        return self.x

# If-elif-else
if x > 0:
    y = 1
elif x < 0:
    y = -1
else:
    y = 0

# For loop
for i in range(10):
    print(i)

# While loop
while x > 0:
    x -= 1

# With statement
with open('file.txt') as f:
    content = f.read()

# Try-except-else-finally
try:
    result = risky_function()
except Exception as e:
    print(e)
else:
    print(\"Success\")
finally:
    cleanup()

# Import statements
import math
from os import path

# Assignment variations
x = 1
x += 2
y: int = 3

# Delete statement
del x

# Assert statement
assert x > 0, \"x must be positive\"

# Pass, break, continue
def empty():
    pass

for i in range(10):
    if i % 2 == 0:
        continue
    if i > 5:
        break
    print(i)

# Global and nonlocal
global_var = 0

def outer():
    var = 1
    def inner():
        nonlocal var
        var = 2
    inner()
    return var
";
        
        // This test just verifies that all statement types can be parsed
        assert!(parse_and_format(source, 4).is_ok());
    }
}