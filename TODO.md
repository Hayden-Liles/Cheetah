# Cheetah Project To-Do List

This document tracks tasks, features, and improvements for the Cheetah Python compiler project.

## How to Use This List
- Completed tasks are marked with `[x]`
- Incomplete tasks are marked with `[ ]`
- Tasks are organized by priority and logical sequence
- Add new tasks at the bottom of the appropriate section

## Current Focus (High Priority)

### Core Language Features
1. [x] Fix LLVM validation issues with nonlocal variables (COMPLETED)
2. [x] Fix LLVM validation issues with function calls (COMPLETED)
3. [x] Implement exception handling (COMPLETED)
4. [x] Validate exception handling implementation (COMPLETED)
   - [x] Create comprehensive tests for exception handling
   - [x] Test exception propagation through multiple function calls
   - [x] Test exception handling in nested functions with nonlocal variables
   - [x] Test interaction between exceptions and other control flow (loops, conditionals)
   - [x] Test exception handling with different data types
   - [x] Verify correct cleanup in finally blocks
   - [x] Test exception handling with complex expressions
5. [ ] Add support for modules and imports
   - [ ] Implement basic module loading
   - [ ] Support for import statements
   - [ ] Handle module-level variables and functions
   - [ ] Support relative imports
   - [ ] Implement import caching to avoid duplicate imports
   - [ ] Support for importing specific symbols from modules
   - [ ] Add tests for module imports

### Essential Built-ins
6. [ ] Complete built-in functions implementation
   - [x] Implement print() function
   - [x] Implement input() function
   - [x] Implement len() function for strings, lists, and dictionaries
   - [x] Implement type conversion functions (int(), float(), bool(), str())
   - [ ] Implement other common built-in functions (min, max, etc.)
   - [ ] Implement string manipulation functions

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
7. [ ] Enhance testing infrastructure
   - [x] Add comprehensive tests for dictionary operations
   - [x] Fix failing tests for nonlocal variables in nested functions
   - [x] Fix failing tests for len() function
   - [x] Add comprehensive tests for exception handling
   - [ ] Create integration tests comparing behavior with CPython
   - [ ] Add tests for edge cases in type conversions
   - [ ] Create a test suite for comparing compiled output with CPython execution
   - [ ] Add tests for error handling and recovery

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
- [ ] Add a debugger
- [ ] Implement a profiler
- [ ] Create a package manager

## Project Management

### Infrastructure
- [ ] Set up continuous integration
- [ ] Add automated release process
- [ ] Improve build system
- [x] Create installation script
- [ ] Create distribution packages

### Community
- [ ] Create contribution guidelines
- [ ] Set up project website
- [ ] Write blog posts about the project
- [ ] Create tutorials for new users

## Notes and Ideas

- ✅ Implemented a robust solution for nonlocal variables using closure environments
- ✅ Fixed LLVM validation issues with function calls and len() function in nested functions
- ✅ Implemented enhanced exception handling with try/except/else/finally blocks
  - ✅ Added runtime support for exceptions (creation, raising, checking)
  - ✅ Implemented try-except blocks with proper control flow
  - ✅ Added support for else and finally blocks
  - ✅ Implemented exception variable binding (as in 'except Exception as e')
  - ✅ Added support for nested try-except blocks
  - ✅ Created comprehensive tests for exception handling functionality
  - ✅ Fixed basic block termination issues with return statements in try-except blocks
  - ✅ Added simplified tests for raise and catch functionality
  - ✅ Added test for exception variable binding
  - ⏳ Advanced exception handling features moved to medium priority
  - ℹ️ Suggested test cases for validating exception handling:
    - Test exception propagation through multiple function calls
    - Test exception handling in nested functions with nonlocal variables
    - Test interaction between exceptions and loops/conditionals
    - Test resource cleanup in finally blocks with various exit scenarios
    - Test exception handling with different data types (int, string, list, etc.)
    - Test memory management during exception handling
    - Compare behavior with CPython for compatibility
- ✅ Implemented command-line interface with .ch file extension support
  - ✅ Added support for running files directly with `cheetah main.ch`
  - ✅ Standardized on .ch file extension for Cheetah source files
  - ✅ Created installation script for system-wide availability
  - ✅ Added example files with .ch extension
- ✅ Implemented print() function
  - ✅ Added support for printing different data types (string, int, float, bool)
  - ✅ Implemented proper newline handling
  - ✅ Added support for printing multiple arguments
  - ✅ Created test examples for print functionality
- Research how other compilers handle closure environments and variable capture
- Consider implementing a static analysis pass to identify all nonlocal variables before code generation
- Look into how Python's exception handling is implemented in CPython for inspiration
- Research performance improvements for numeric operations
- Explore Symbol from JS and how it could be used in Cheetah
