use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cheetah::lexer::Lexer;

fn lex_small_program(c: &mut Criterion) {
    let source = r#"
def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

result = factorial(5)
print("Factorial of 5 is", result)
"#;

    c.bench_function("lex_small_program", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            lexer.tokenize()
        })
    });
}

fn lex_medium_program(c: &mut Criterion) {
    // Load sample program from file
    let source = include_str!("../examples/advanced.ch");

    c.bench_function("lex_medium_program", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            lexer.tokenize()
        })
    });
}

fn lex_long_string(c: &mut Criterion) {
    // Test with a very long string
    let long_string = "\"".to_string() + &"a".repeat(10000) + "\"";

    c.bench_function("lex_long_string", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&long_string));
            lexer.tokenize()
        })
    });
}

fn lex_deep_nesting(c: &mut Criterion) {
    // Create a deeply nested structure
    let mut nested = String::new();
    nested.push_str("def nested():\n");
    
    // Create 50 levels of nesting
    for i in 0..50 {
        let indent = "    ".repeat(i + 1);
        nested.push_str(&format!("{}if True:\n", indent));
    }
    
    // Add a statement at the deepest level
    nested.push_str(&"    ".repeat(51));
    nested.push_str("return \"deep\"");

    c.bench_function("lex_deep_nesting", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&nested));
            lexer.tokenize()
        })
    });
}

criterion_group!(
    benches, 
    lex_small_program, 
    lex_medium_program, 
    lex_long_string,
    lex_deep_nesting
);
criterion_main!(benches);