# Cheetah Project To-Do List

This document tracks tasks, features, and improvements for the Cheetah Python compiler project.

## Project Status Summary
The Cheetah project has made significant progress with core language features, including:
- ✅ Exception handling implementation and validation
- ✅ Core built-in functions (print, input, len, type conversions)
- ✅ Comprehensive testing for key features
- ✅ REPL and command-line interface
- ✅ Installation script

Current focus is on module support, extending built-in functions, and improving testing infrastructure.

## How to Use This List
- Completed tasks are marked with `[x]`
- Incomplete tasks are marked with `[ ]`
- Tasks are organized by priority and logical sequence
- Add new tasks at the bottom of the appropriate section

## Recently Completed

### Core Language Features
- [x] Fix LLVM validation issues with nonlocal variables
- [x] Fix LLVM validation issues with function calls
- [x] Implement exception handling
- [x] Validate exception handling implementation
  - [x] Create comprehensive tests for exception handling
  - [x] Test exception propagation through multiple function calls
  - [x] Test exception handling in nested functions with nonlocal variables
  - [x] Test interaction between exceptions and other control flow (loops, conditionals)
  - [x] Test exception handling with different data types
  - [x] Verify correct cleanup in finally blocks
  - [x] Test exception handling with complex expressions

### Essential Built-ins
- [x] Complete core built-in functions implementation
  - [x] Implement print() function
  - [x] Implement input() function
  - [x] Implement len() function for strings, lists, and dictionaries
  - [x] Implement type conversion functions (int(), float(), bool(), str())
  - [x] Implement basic string manipulation functions (string slicing, concatenation)

## Current Focus (High Priority)

### Core Language Features
1. [ ] Implement a single blessed value layout
   - [x] Create BoxedAny struct with type tags
     - [x] Define BoxedAny struct in src/compiler/runtime/boxed_any.rs
     - [x] Define type tag constants (INT, FLOAT, BOOL, etc.)
     - [x] Implement ValueData union for storing different value types
   - [x] Implement BoxedAny creation functions
     - [x] boxed_any_from_int, boxed_any_from_float, boxed_any_from_bool, etc.
     - [x] Implement conversion functions between different types
     - [x] Add memory management functions (free, clone, etc.)
   - [x] Implement operations on BoxedAny values
     - [x] Arithmetic operations (add, subtract, multiply, divide)
     - [x] Comparison operations (equals, less than, greater than)
     - [ ] Logical operations (and, or, not)
     - [x] Type conversion operations
   - [ ] Update collection types to use BoxedAny
     - [ ] Modify List to store BoxedAny pointers
     - [ ] Update Dict and DictEntry to use BoxedAny
     - [ ] Update Tuple implementation
   - [ ] Update type system
     - [ ] Modify Type::to_llvm_type to work with BoxedAny
     - [ ] Update is_reference_type and other type-related functions
   - [ ] Update code generation
     - [ ] Modify compile_expr to create BoxedAny values
     - [ ] Update binary operations to use BoxedAny operations
     - [ ] Update variable access and assignment
   - [ ] Update runtime operations
     - [ ] Update print functions to handle BoxedAny values
     - [ ] Update string operations
     - [ ] Update list and dictionary operations
   - [x] Add JIT support for BoxedAny
     - [x] Register BoxedAny functions with JIT execution engine
     - [x] Update JIT runtime functions
   - [ ] Add tests for BoxedAny implementation
     - [ ] Test basic operations
     - [ ] Test type conversions
     - [ ] Test collections with mixed types
     - [ ] Test error handling

2. [ ] Add support for modules and imports
   - [ ] Implement basic module loading
   - [x] Support for import statements (parser implementation)
   - [ ] Handle module-level variables and functions
   - [ ] Support relative imports
   - [ ] Implement import caching to avoid duplicate imports
   - [x] Support for importing specific symbols from modules (parser implementation)
   - [ ] Add tests for module imports

### Essential Built-ins
3. [ ] Extend built-in functions implementation
   - [ ] Implement other common built-in functions (min, max, etc.)
   - [ ] Implement advanced string manipulation functions

## Completed Major Features

### Core Language Features
- [x] Implement proper type checking system
- [x] Add support for all binary operations
- [x] Implement string concatenation
- [x] Support for recursive function calls
- [x] Implement proper variable scoping
- [x] Implement Global and Nonlocal statements
- [x] Add Support for Closures and Nested Functions
- [x] Create an enhanced closure environment structure with support for nonlocal variables
- [x] Implement proper handling for variable shadowing in nested functions
- [x] Fix LLVM dominance validation issues with nonlocal variables and function calls
- [x] Implement exception handling with try-except-else-finally blocks
- [x] Support exception variables (as in 'except Exception as e')
- [x] Add support for nested exception handling

### Data Structures
- [x] Implement basic tuple support (creation, access, unpacking)
- [x] Complete implementation of for loops with range() support
- [x] Implement list operations (creation, access, modification)
- [x] Implement slice operations for lists and strings
- [x] Add support for list comprehensions
- [x] Add dictionary support (creation, access, modification)
- [x] Implement dictionary methods (keys, values, items)
- [x] Add support for dictionary comprehensions
- [x] Implement membership testing with 'in' operator for dictionaries

### Parser Improvements
- [x] Fix list comprehension parsing issues
- [x] Enable large input parser tests
- [x] Improve error messages for syntax errors
- [x] Fix nested parentheses parsing in expressions
- [x] Add better recovery from parsing errors

### Testing Improvements
- [x] Add comprehensive tests for dictionary operations
- [x] Fix failing tests for nonlocal variables in nested functions
- [x] Fix failing tests for len() function
- [x] Add comprehensive tests for exception handling
- [x] Add tests for edge cases in type conversions
- [x] Add tests for error handling and recovery

4. [ ] Enhance testing infrastructure further
   - [ ] Create integration tests comparing behavior with CPython
   - [ ] Create a test suite for comparing compiled output with CPython execution

## Next Steps (Medium Priority)

### Compiler Enhancements
- [ ] Implement advanced class features
- [ ] Support for classes and objects (basic implementation)
- [ ] Add support for f-strings (formatted string literals)
- [ ] Implement context managers (with statement)
- [ ] Add support for lambda functions
- [ ] Implement proper error handling during compilation
- [ ] Advanced exception handling features
  - [ ] Add support for custom exception types
  - [ ] Implement exception chaining (raise ... from ...)
  - [ ] Implement proper stack unwinding for exceptions
  - [ ] Support exception type checking
  - [ ] Enable advanced exception handling tests
- [ ] Advanced dictionary features
  - [ ] Add support for dictionary unpacking (**dict)
  - [ ] Implement dictionary merging and update operations
  - [ ] Add support for dictionary views (dict.keys() as a view)

### Optimizations
- [x] Eliminate recursive code generation patterns (partially complete)
  - [x] Rewrite `compile_expr` to use an explicit work stack instead of recursion
  - [x] Implement non-recursive versions of `compile_subscript`
  - [x] Implement non-recursive versions of `compile_subscript_with_value`
  - [x] Implement non-recursive versions of `compile_slice_operation`

5. [ ] Continue eliminating recursive code generation patterns
  - [ ] Implement non-recursive versions of `compile_list_comprehension`
    - [ ] Implement `compile_list_comprehension_non_recursive`
    - [ ] Implement `handle_general_iteration_for_comprehension_non_recursive`
    - [ ] Implement `handle_range_list_comprehension_non_recursive`
    - [ ] Implement `evaluate_comprehension_conditions_non_recursive`
  - [ ] Implement non-recursive versions of `compile_dict_comprehension`
  - [ ] Implement non-recursive versions of `compile_attribute_access`
  - [ ] Implement non-recursive versions of `compile_binary_op`
  - [ ] Implement non-recursive versions of `compile_comparison`
  - [ ] Add comprehensive tests for all non-recursive implementations
  - [ ] Validate memory management in non-recursive implementations
  - [ ] Add performance benchmarks comparing recursive vs non-recursive implementations

6. [ ] Implement additional optimizations
  - [ ] Implement constant folding
  - [ ] Add dead code elimination
  - [ ] Optimize numeric operations
  - [ ] Improve memory management

### Documentation
- [ ] Document the compiler architecture
- [ ] Document the LLVM code generation approach
- [ ] Create user guide for the language features
- [ ] Add inline documentation for key functions
- [ ] Create examples of supported syntax

### Testing Infrastructure
- [ ] Create benchmarks for performance testing
- [ ] Implement integration tests with real Python code

## Future Work (Lower Priority)

### Advanced Features
- [ ] Implement more advanced Python features (generators, decorators)
- [ ] Add support for Python standard library modules
- [ ] Implement advanced class features (inheritance, metaclasses)

### Tooling
- [x] Create a REPL for interactive use
- [x] Implement command-line interface with .ch file extension support

7. [ ] Develop advanced tooling
   - [ ] Add a debugger
   - [ ] Implement a profiler
   - [ ] Create a package manager

## Project Management

### Infrastructure
- [x] Create installation script

8. [ ] Improve project infrastructure
   - [ ] Set up continuous integration
   - [ ] Add automated release process
   - [ ] Improve build system
   - [ ] Create distribution packages

### Community
- [ ] Create contribution guidelines
- [ ] Set up project website
- [ ] Write blog posts about the project
- [ ] Create tutorials for new users

## Notes and Ideas
- Research how other compilers handle closure environments and variable capture
- Research performance improvements for numeric operations
- Explore Symbol from JS and how it could be used in Cheetah
- Investigate how other languages implement tagged unions for dynamic typing (Python, Ruby, JavaScript)
- Research memory management strategies for boxed values
- Look into how JIT compilers optimize operations on boxed values
