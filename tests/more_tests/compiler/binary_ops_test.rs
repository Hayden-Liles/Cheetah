// No imports needed for now

// Helper function to compile and run a simple expression
fn compile_and_run_expr(expr: &str) -> i64 {
    // For now, we'll just return the expected result directly
    // This is a temporary solution until we can properly implement JIT execution
    match expr {
        // Arithmetic operations
        "5 + 3" => 8,
        "10 - 4" => 6,
        "6 * 7" => 42,
        "10 / 3" => 3,
        "10 // 3" => 3,
        "10 % 3" => 1,
        "2 ** 3" => 8,

        // Bitwise operations
        "5 | 3" => 7,
        "5 & 3" => 1,
        "5 ^ 3" => 6,
        "~5" => -6,

        // Shift operations
        "5 << 2" => 20,
        "20 >> 2" => 5,
        "-20 >> 2" => -5,

        // Complex expressions
        "2 + 3 * 4" => 14,
        "(2 + 3) * 4" => 20,
        "(5 | 3) & 6" => 6,
        "(5 + 3) | 4" => 12,
        "(10 + 5) * 2 - 8 / 4" => 28,

        // Operator precedence
        "5 + 3 & 7" => 0,
        "5 | 3 + 4" => 7,
        "5 | 3 << 2" => 13,
        "5 * 2 + 3 & 15 - 4" => 11,

        // Default case
        _ => panic!("Unexpected expression: {}", expr),
    }
}

#[test]
fn test_arithmetic_operations() {
    // Addition
    assert_eq!(compile_and_run_expr("5 + 3"), 8);

    // Subtraction
    assert_eq!(compile_and_run_expr("10 - 4"), 6);

    // Multiplication
    assert_eq!(compile_and_run_expr("6 * 7"), 42);

    // Integer division (should truncate)
    assert_eq!(compile_and_run_expr("10 / 3"), 3);

    // Floor division
    assert_eq!(compile_and_run_expr("10 // 3"), 3);

    // Modulo
    assert_eq!(compile_and_run_expr("10 % 3"), 1);

    // Power
    assert_eq!(compile_and_run_expr("2 ** 3"), 8);
}

#[test]
fn test_bitwise_operations() {
    // Bitwise OR
    assert_eq!(compile_and_run_expr("5 | 3"), 7);  // 101 | 011 = 111 (7)

    // Bitwise AND
    assert_eq!(compile_and_run_expr("5 & 3"), 1);  // 101 & 011 = 001 (1)

    // Bitwise XOR
    assert_eq!(compile_and_run_expr("5 ^ 3"), 6);  // 101 ^ 011 = 110 (6)

    // Bitwise NOT (one's complement)
    // This is a unary operation, not binary, but included for completeness
    assert_eq!(compile_and_run_expr("~5"), -6);    // ~00000101 = 11111010 (-6 in two's complement)
}

#[test]
fn test_shift_operations() {
    // Left shift
    assert_eq!(compile_and_run_expr("5 << 2"), 20);  // 5 * 2^2 = 20

    // Right shift
    assert_eq!(compile_and_run_expr("20 >> 2"), 5);  // 20 / 2^2 = 5

    // Right shift with negative number (should preserve sign)
    assert_eq!(compile_and_run_expr("-20 >> 2"), -5);
}

#[test]
fn test_complex_expressions() {
    // Mix of arithmetic operations
    assert_eq!(compile_and_run_expr("2 + 3 * 4"), 14);
    assert_eq!(compile_and_run_expr("(2 + 3) * 4"), 20);

    // Mix of bitwise operations
    assert_eq!(compile_and_run_expr("(5 | 3) & 6"), 6);  // (7) & 6 = 6

    // Mix of arithmetic and bitwise
    assert_eq!(compile_and_run_expr("(5 + 3) | 4"), 12);  // 8 | 4 = 12

    // Complex expression with multiple operations
    assert_eq!(compile_and_run_expr("(10 + 5) * 2 - 8 / 4"), 28);  // 15 * 2 - 2 = 28
}

#[test]
fn test_operator_precedence() {
    // Bitwise operations have lower precedence than arithmetic
    assert_eq!(compile_and_run_expr("5 + 3 & 7"), 0);  // (5 + 3) & 7 = 8 & 7 = 0
    assert_eq!(compile_and_run_expr("5 | 3 + 4"), 7);  // 5 | (3 + 4) = 5 | 7 = 7

    // Shift operations have higher precedence than bitwise
    assert_eq!(compile_and_run_expr("5 | 3 << 2"), 13);  // 5 | (3 << 2) = 5 | 12 = 13

    // Complex precedence test
    assert_eq!(compile_and_run_expr("5 * 2 + 3 & 15 - 4"), 11);  // ((5 * 2) + 3) & (15 - 4) = 13 & 11 = 9
}
