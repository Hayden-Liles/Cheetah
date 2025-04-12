# Cheetah Project To-Do List

This document tracks tasks, features, and improvements for the Cheetah Python compiler project.

## How to Use This List
- Completed tasks are marked with `[x]`
- Incomplete tasks are marked with `[ ]`
- Tasks are organized by priority and logical sequence
- Add new tasks at the bottom of the appropriate section

## Completed Tasks

### Parser Improvements
- [x] Fix list comprehension parsing issues
- [x] Enable large input parser tests
- [x] Improve error messages for syntax errors
- [x] Fix nested parentheses parsing in expressions
- [x] Add better recovery from parsing errors

### Compiler Core
- [x] Implement proper type checking system
- [x] Add support for all binary operations
- [x] Implement string concatenation
- [x] Support for recursive function calls
- [x] Implement proper variable scoping
- [x] Implement Global
- [x] Add Support for Closures (basic infrastructure)
- [x] Add Support for Nested Functions
- [x] Implement Nonlocal Statements (basic support)
- [x] Improve Global Variable Access from nested scopes
- [x] Implement a solution for nonlocal variables in nested functions (using global variables and unique variable names)
- [x] Fix basic LLVM validation issues with nonlocal variables in nested functions
- [x] Improve handling of nonlocal variables in complex scenarios (conditionals, loops)
- [x] Fix remaining LLVM validation issues with nonlocal variables in complex scenarios (shadowing, nested nonlocals)
- [x] Implement a more robust solution for nonlocal variables using global variables with unique names
- [x] Add tests for closure support

## Current Focus (High Priority)

### Core Language Features
1. [x] Implement basic tuple support (creation, access)
2. [x] Complete tuple support (function arguments, return values, unpacking)
   - [x] Fix variable registration for tuple unpacking with assignment
   - [x] Support direct variable creation from tuple unpacking
   - [x] Improve function parameter type inference for tuples
   - [x] Add support for nested tuple unpacking in functions
   - [x] Add support for multiple tuple parameters in functions
3. [ ] Complete implementation of for loops
4. [ ] Implement list operations
5. [ ] Implement slice operations for lists and strings
6. [ ] Add support for list comprehensions
7. [ ] Implement a proper closure environment solution for nonlocal variables

### Essential Built-ins
8. [ ] Support for the range() built-in function
9. [ ] Implement basic built-in functions (print, len, etc.)
10. [ ] Add dictionary support
11. [ ] Implement string manipulation functions

### Testing Improvements
12. [ ] Add more comprehensive test cases for new features
13. [ ] Add tests for edge cases in type conversions
14. [ ] Create a test suite for comparing compiled output with CPython execution
15. [ ] Add tests for error handling and recovery

## Next Steps (Medium Priority)

### Compiler Enhancements
- [ ] Implement exception handling (try/except/finally)
- [ ] Add support for modules and imports
- [ ] Support for classes and objects (basic implementation)
- [ ] Add support for f-strings (formatted string literals)
- [ ] Implement context managers (with statement)
- [ ] Add support for lambda functions
- [ ] Implement proper error handling during compilation

### Optimizations
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
- [ ] Create a REPL for interactive use
- [ ] Add a debugger
- [ ] Implement a profiler
- [ ] Create a package manager

## Project Management

### Infrastructure
- [ ] Set up continuous integration
- [ ] Add automated release process
- [ ] Improve build system
- [ ] Create installation packages

### Community
- [ ] Create contribution guidelines
- [ ] Set up project website
- [ ] Write blog posts about the project
- [ ] Create tutorials for new users

## Notes and Ideas

- Consider implementing a JIT compilation option
- Explore WASM as a target platform
- Look into Python 3.10+ features to support
- Research performance improvements for numeric operations

