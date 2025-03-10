#[cfg(test)]
mod tests {
    use cheetah::lexer::Lexer;
    use cheetah::parser::{Parser, ParseError};
    use cheetah::ast::{Module, Stmt, Expr, Number};

    // Helper function to parse a string and return the Module
    fn parse_code(source: &str) -> Result<Module, Vec<ParseError>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        if !lexer.get_errors().is_empty() {
            panic!("Lexer errors: {:?}", lexer.get_errors());
        }
        
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // Helper function to assert parsing succeeds
    fn assert_parses(source: &str) {
        match parse_code(source) {
            Ok(_) => {},
            Err(errors) => panic!("Parsing failed with errors: {:?}", errors),
        }
    }

    // Helper to assert parsing fails with a specific error type
    fn assert_parse_fails(source: &str) {
        match parse_code(source) {
            Ok(_) => panic!("Expected parsing to fail, but it succeeded"),
            Err(_) => {}, // Expected to fail
        }
    }

    #[test]
    fn test_parse_simple_assignment() {
        let module = parse_code("x = 42").unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::Assign { targets, value, .. } = &*module.body[0] {
            assert_eq!(targets.len(), 1);
            
            if let Expr::Name { id, .. } = &*targets[0] {
                assert_eq!(id, "x");
            } else {
                panic!("Expected Name expression");
            }
            
            if let Expr::Num { value: Number::Integer(i), .. } = &**value {
                assert_eq!(*i, 42);
            } else {
                panic!("Expected integer value");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_parse_function_definition() {
        let source = "
def add(a, b):
    return a + b
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::FunctionDef { name, params, body, .. } = &*module.body[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
            
            assert_eq!(body.len(), 1);
            if let Stmt::Return { value, .. } = &*body[0] {
                assert!(value.is_some());
                
                if let Expr::BinOp { left, right, .. } = &**value.as_ref().unwrap() {
                    if let Expr::Name { id: left_id, .. } = &**left {
                        assert_eq!(left_id, "a");
                    } else {
                        panic!("Expected name in binary op left side");
                    }
                    
                    if let Expr::Name { id: right_id, .. } = &**right {
                        assert_eq!(right_id, "b");
                    } else {
                        panic!("Expected name in binary op right side");
                    }
                } else {
                    panic!("Expected binary operation in return statement");
                }
            } else {
                panic!("Expected return statement in function body");
            }
        } else {
            panic!("Expected function definition");
        }
    }

    #[test]
    fn test_parse_class_definition() {
        let source = "
class Person:
    def __init__(self, name):
        self.name = name
    
    def greet(self):
        return \"Hello, \" + self.name
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::ClassDef { name, body, .. } = &*module.body[0] {
            assert_eq!(name, "Person");
            assert_eq!(body.len(), 2); // Two methods
            
            // Check __init__ method
            if let Stmt::FunctionDef { name: method_name, .. } = &*body[0] {
                assert_eq!(method_name, "__init__");
            } else {
                panic!("Expected __init__ method");
            }
            
            // Check greet method
            if let Stmt::FunctionDef { name: method_name, .. } = &*body[1] {
                assert_eq!(method_name, "greet");
            } else {
                panic!("Expected greet method");
            }
        } else {
            panic!("Expected class definition");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "
if x > 0:
    y = 1
elif x < 0:
    y = -1
else:
    y = 0
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::If { test, body, orelse, .. } = &*module.body[0] {
            // Check the condition
            if let Expr::Compare { left, .. } = &**test {
                if let Expr::Name { id, .. } = &**left {
                    assert_eq!(id, "x");
                } else {
                    panic!("Expected name in if condition");
                }
            } else {
                panic!("Expected comparison in if condition");
            }
            
            // Check the if body
            assert_eq!(body.len(), 1);
            
            // Check the elif/else part
            assert_eq!(orelse.len(), 1);
            if let Stmt::If { test: elif_test, body: elif_body, orelse: else_body, .. } = &*orelse[0] {
                // Check elif condition
                if let Expr::Compare { left, .. } = &**elif_test {
                    if let Expr::Name { id, .. } = &**left {
                        assert_eq!(id, "x");
                    } else {
                        panic!("Expected name in elif condition");
                    }
                } else {
                    panic!("Expected comparison in elif condition");
                }
                
                // Check elif body
                assert_eq!(elif_body.len(), 1);
                
                // Check else part
                assert_eq!(else_body.len(), 1);
            }
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let source = "
for i in range(10):
    print(i)
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::For { target, iter, body, .. } = &*module.body[0] {
            // Check loop variable
            if let Expr::Name { id, .. } = &**target {
                assert_eq!(id, "i");
            } else {
                panic!("Expected name as loop variable");
            }
            
            // Check iterable
            if let Expr::Call { func, .. } = &**iter {
                if let Expr::Name { id, .. } = &**func {
                    assert_eq!(id, "range");
                } else {
                    panic!("Expected range function call");
                }
            } else {
                panic!("Expected function call as iterable");
            }
            
            // Check body
            assert_eq!(body.len(), 1);
            if let Stmt::Expr { value, .. } = &*body[0] {
                if let Expr::Call { func, .. } = &**value {
                    if let Expr::Name { id, .. } = &**func {
                        assert_eq!(id, "print");
                    } else {
                        panic!("Expected print function call in loop body");
                    }
                }
            }
        } else {
            panic!("Expected for statement");
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let source = "
while x > 0:
    x -= 1
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::While { test, body, .. } = &*module.body[0] {
            // Check condition
            if let Expr::Compare { left, .. } = &**test {
                if let Expr::Name { id, .. } = &**left {
                    assert_eq!(id, "x");
                } else {
                    panic!("Expected name in while condition");
                }
            } else {
                panic!("Expected comparison in while condition");
            }
            
            // Check body
            assert_eq!(body.len(), 1);
            if let Stmt::AugAssign { target, .. } = &*body[0] {
                if let Expr::Name { id, .. } = &**target {
                    assert_eq!(id, "x");
                } else {
                    panic!("Expected name in augmented assignment");
                }
            } else {
                panic!("Expected augmented assignment in while body");
            }
        } else {
            panic!("Expected while statement");
        }
    }

    #[test]
    fn test_parse_list_comprehension() {
        let source = "squares = [x*x for x in range(10) if x % 2 == 0]";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::Assign { targets, value, .. } = &*module.body[0] {
            assert_eq!(targets.len(), 1);
            
            if let Expr::Name { id, .. } = &*targets[0] {
                assert_eq!(id, "squares");
            } else {
                panic!("Expected Name expression");
            }
            
            if let Expr::ListComp { elt, generators, .. } = &**value {
                // Check the expression
                if let Expr::BinOp { left, right, .. } = &**elt {
                    if let (Expr::Name { id: left_id, .. }, Expr::Name { id: right_id, .. }) = (&**left, &**right) {
                        assert_eq!(left_id, "x");
                        assert_eq!(right_id, "x");
                    } else {
                        panic!("Expected names in binary operation");
                    }
                } else {
                    panic!("Expected binary operation in list comprehension");
                }
                
                // Check the generator
                assert_eq!(generators.len(), 1);
                let comp = &generators[0];
                
                if let Expr::Name { id, .. } = &*comp.target {
                    assert_eq!(id, "x");
                } else {
                    panic!("Expected name in comprehension target");
                }
                
                // Check the if condition
                assert_eq!(comp.ifs.len(), 1);
            } else {
                panic!("Expected list comprehension");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_parse_dictionary() {
        let source = "data = {\"name\": \"John\", \"age\": 30}";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::Assign { targets, value, .. } = &*module.body[0] {
            assert_eq!(targets.len(), 1);
            
            if let Expr::Name { id, .. } = &*targets[0] {
                assert_eq!(id, "data");
            } else {
                panic!("Expected Name expression");
            }
            
            if let Expr::Dict { keys, values, .. } = &**value {
                assert_eq!(keys.len(), 2);
                assert_eq!(values.len(), 2);
                
                // Check the first key-value pair
                if let Some(key) = &keys[0] {
                    if let Expr::Str { value: key_value, .. } = &**key {
                        assert_eq!(key_value, "name");
                    } else {
                        panic!("Expected string key");
                    }
                }
                
                if let Expr::Str { value, .. } = &*values[0] {
                    assert_eq!(value, "John");
                } else {
                    panic!("Expected string value");
                }
                
                // Check the second key-value pair
                if let Some(key) = &keys[1] {
                    if let Expr::Str { value: key_value, .. } = &**key {
                        assert_eq!(key_value, "age");
                    } else {
                        panic!("Expected string key");
                    }
                }
                
                if let Expr::Num { value: Number::Integer(i), .. } = &*values[1] {
                    assert_eq!(*i, 30);
                } else {
                    panic!("Expected integer value");
                }
            } else {
                panic!("Expected dictionary expression");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_parse_try_except() {
        let source = "
try:
    result = risky_operation()
except ValueError as e:
    print(\"Value error:\", e)
except Exception:
    print(\"Unknown error\")
else:
    print(\"Success\")
finally:
    cleanup()
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::Try { body, handlers, orelse, finalbody, .. } = &*module.body[0] {
            // Check try body
            assert_eq!(body.len(), 1);
            
            // Check except handlers
            assert_eq!(handlers.len(), 2);
            
            let first_handler = &handlers[0];
            assert!(first_handler.typ.is_some());
            assert!(first_handler.name.is_some());
            assert_eq!(first_handler.name.as_ref().unwrap(), "e");
            
            let second_handler = &handlers[1];
            assert!(second_handler.typ.is_some());
            assert!(second_handler.name.is_none());
            
            // Check else body
            assert_eq!(orelse.len(), 1);
            
            // Check finally body
            assert_eq!(finalbody.len(), 1);
        } else {
            panic!("Expected try statement");
        }
    }

    #[test]
    fn test_parse_import() {
        let source = "
import math
from os import path, system
from . import module
";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 3);
        
        if let Stmt::Import { names, .. } = &*module.body[0] {
            assert_eq!(names.len(), 1);
            assert_eq!(names[0].name, "math");
            assert!(names[0].asname.is_none());
        } else {
            panic!("Expected import statement");
        }
        
        if let Stmt::ImportFrom { module: mod_name, names, level, .. } = &*module.body[2] {
            assert!(mod_name.is_none());
            assert_eq!(*level, 1);
            assert_eq!(names.len(), 1);
            assert_eq!(names[0].name, "module");
        } else {
            panic!("Expected import from statement");
        }
        
        if let Stmt::ImportFrom { module: mod_name, names, level, .. } = &*module.body[2] {
            assert!(mod_name.is_some());
            assert_eq!(*level, 1);  // Relative import with one dot
            assert_eq!(names.len(), 1);
            assert_eq!(names[0].name, "module");
        } else {
            panic!("Expected relative import statement");
        }
    }

    #[test]
    fn test_parse_lambda() {
        let source = "func = lambda x, y=10: x + y";
        let module = parse_code(source).unwrap();
        
        assert_eq!(module.body.len(), 1);
        
        if let Stmt::Assign { targets, value, .. } = &*module.body[0] {
            assert_eq!(targets.len(), 1);
            
            if let Expr::Name { id, .. } = &*targets[0] {
                assert_eq!(id, "func");
            } else {
                panic!("Expected Name expression");
            }
            
            if let Expr::Lambda { args, body, .. } = &**value {
                assert_eq!(args.len(), 2);
                assert_eq!(args[0].name, "x");
                assert_eq!(args[1].name, "y");
                assert!(args[1].default.is_some());
                
                if let Expr::BinOp { left, right, .. } = &**body {
                    if let Expr::Name { id: left_id, .. } = &**left {
                        assert_eq!(left_id, "x");
                    } else {
                        panic!("Expected name in binary op left side");
                    }
                    
                    if let Expr::Name { id: right_id, .. } = &**right {
                        assert_eq!(right_id, "y");
                    } else {
                        panic!("Expected name in binary op right side");
                    }
                } else {
                    panic!("Expected binary operation in lambda body");
                }
            } else {
                panic!("Expected lambda expression");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_parse_syntax_errors() {
        // Missing colon in if statement
        assert_parse_fails("if x > 0\n    y = 1");
        
        // Invalid indentation
        assert_parse_fails("def test():\n  x = 1\n y = 2");
        
        // Unclosed parentheses
        assert_parse_fails("f(1, 2, 3");
        
        // Invalid assignment target
        assert_parse_fails("1 = x");
    }

    #[test]
    fn test_parse_complex_program() {
        let source = "
class BinaryTree:
    def __init__(self, value=None):
        self.value = value
        self.left = None
        self.right = None
    
    def insert(self, value):
        if self.value is None:
            self.value = value
            return
        
        if value < self.value:
            if self.left is None:
                self.left = BinaryTree(value)
            else:
                self.left.insert(value)
        else:
            if self.right is None:
                self.right = BinaryTree(value)
            else:
                self.right.insert(value)
    
    def in_order_traversal(self):
        result = []
        
        if self.left:
            result.extend(self.left.in_order_traversal())
        
        if self.value is not None:
            result.append(self.value)
            
        if self.right:
            result.extend(self.right.in_order_traversal())
            
        return result

# Create a new tree
tree = BinaryTree()

# Insert values
for value in [5, 3, 7, 2, 4, 6, 8]:
    tree.insert(value)

# Get sorted values
sorted_values = tree.in_order_traversal()
print(\"Sorted values:\", sorted_values)
";
        
        // This test just checks if the complex program parses successfully
        assert_parses(source);
    }
}