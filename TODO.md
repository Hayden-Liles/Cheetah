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
- [x] Fix LLVM dominance validation issues with nonlocal variables in deeply nested functions
- [x] Implement proper handling for variable shadowing in nested functions
- [x] Create an enhanced closure environment structure with support for nonlocal variables

## Current Focus (High Priority)

### Core Language Features
1. [x] Fix LLVM validation issues with nonlocal variables (COMPLETED)
   - [x] Implement a basic closure environment solution for nonlocal variables
   - [x] Create a closure environment structure to store nonlocal variables
   - [x] Pass the environment pointer to nested functions
   - [x] Update nonlocal variables in the environment
   - [x] Load nonlocal variables from the environment
   - [x] Pass nonlocal variables as parameters to nested functions
   - [x] Add nonlocal proxy system to avoid dominance validation issues
   - [x] Fix LLVM dominance validation issues with nonlocal variables in nested functions
   - [x] Fix LLVM validation issues with nonlocal variables in simple cases
   - [x] Fix LLVM validation issues with nonlocal variables in complex cases (loops, conditionals)
   - [x] Fix LLVM validation issues with nonlocal variables in nested functions with multiple levels
   - [x] Improve nonlocal variable lookup to properly handle variables in outer scopes
   - [x] Fix LLVM validation issues with nonlocal variables in shadowing cases
   - [x] Fix LLVM dominance validation issues with deeply nested functions
   - [x] Fix LLVM dominance validation issues with variable shadowing in nested functions
2. [x] Fix LLVM validation issues with function calls (COMPLETED)
   - [x] Fix len() function in nested function calls with parameters
   - [x] Fix LLVM validation issues with function calls
   - [x] Implement proper handling for function calls with len()
   - [x] Fix LLVM dominance validation issues with function calls
3. [x] Implement basic exception handling structure (COMPLETED)
   - [x] Define exception runtime structure
   - [x] Implement basic exception creation and raising
   - [x] Implement simple try-except blocks
   - [x] Add structure for finally blocks
   - [x] Add basic tests for exception handling
   - [x] Complete full exception handling implementation
4. [x] Complete exception handling implementation (COMPLETED)
   - [x] Implement proper exception propagation through the call stack
   - [x] Add support for nested exception handling
   - [x] Support exception variables (as in 'except Exception as e')
   - [x] Enable basic exception handling tests
   - [x] Support try-except-else-finally blocks
   - [x] Fix function call parameter type mismatch in exception tests
   - [x] Fix basic block termination issues with return statements in try-except blocks
   - [x] Add simplified tests for raise and catch functionality
   - [x] Add test for exception variable binding
5. [ ] Add support for modules and imports
   - [ ] Implement basic module loading
   - [ ] Support for import statements
   - [ ] Handle module-level variables and functions
   - [ ] Support relative imports
   - [ ] Add support for module-level functions and variables
   - [ ] Implement import caching to avoid duplicate imports
   - [ ] Support for importing specific symbols from modules
   - [ ] Add tests for module imports

### Completed High Priority Features
4. [x] Enhance dictionary support further
   - [x] Implement dictionary methods (keys, values, items)
   - [x] Add support for dictionary comprehensions
   - [x] Implement membership testing with 'in' operator for dictionaries
   - [x] Improve dictionary integration with functions
5. [x] Fix remaining dictionary integration issues
   - [x] Fix type mismatch issues with dictionary function parameters
   - [x] Resolve bitcast issues with dictionary methods in functions
   - [x] Improve type inference for nested dictionaries
   - [x] Enhance typechecker to better handle dictionary indexing

### Completed Core Language Features
4. [x] Implement basic tuple support (creation, access)
5. [x] Complete basic tuple support (function arguments, return values, unpacking)
   - [x] Fix variable registration for tuple unpacking with assignment
   - [x] Support direct variable creation from tuple unpacking
   - [x] Improve function parameter type inference for tuples
   - [x] Add support for nested tuple unpacking in functions
   - [x] Add support for multiple tuple parameters in functions
   - [x] Implement subscript access for tuples (e.g., tuple[0])
   - [x] Improve type inference for tuples in function parameters and returns
   - [x] Add support for inferring tuple types from function calls
   - [x] Add support for mixed-type tuples (elements of different types)
   - [x] Implement dynamic tuple indexing (using variables as indices)
   - [x] Complete implementation of advanced tuple features in ignored tests
      - [x] Implement nested tuple indexing (e.g., t[1][0])
      - [x] Improve function return type inference for tuples
      - [x] Implement advanced tuple function arguments
      - [x] Implement tuple unpacking in functions
      - [x] Support tuples with function calls
      - [x] Implement complex tuple scenarios
5. [x] Complete implementation of for loops
   - [x] Implement basic for loop structure
   - [x] Add support for range() function
   - [x] Implement break and continue statements
   - [x] Support for loop else clause
6. [x] Implement list operations
   - [x] Define list structure in LLVM
   - [x] Implement list creation (empty and with elements)
   - [x] Add list access (get item by index)
   - [x] Implement list binary operations (concatenation and repetition)
   - [x] Create comprehensive tests for list operations
   - [x] Fix advanced list operations:
     - [x] Fix list element assignment (numbers[0] = 100)
     - [x] Fix list operations in loops (for num in numbers)
     - [x] Fix list operations in functions (get_first, append_to_list)
7. [x] Implement slice operations for lists and strings
   - [x] Define slice syntax and semantics
   - [x] Update the parser to handle slice notation
   - [x] Implement slice operations for lists in the compiler
   - [x] Implement slice operations for strings in the compiler
   - [x] Create tests for slice operations
8. [x] Add support for list comprehensions
   - [x] Implement basic list comprehension syntax
   - [x] Support for list comprehensions with range
   - [x] Support for list comprehensions with conditions (if clauses)
   - [x] Support for list comprehensions with strings
   - [x] Support for list comprehensions with lists
   - [x] Support for nested list comprehensions
   - [x] Add comprehensive tests for list comprehensions
     - [x] Test various expression types in list comprehensions
     - [x] Test complex and deeply nested list comprehensions
     - [x] Test edge cases like empty list comprehensions
     - [x] Support for advanced features (string operations, arithmetic operations, membership operations)
     - [x] Support for more advanced features (tuple unpacking in comprehensions, multiple for clauses)
9. [x] Add dictionary support
   - [x] Implement basic dictionary structure in LLVM
   - [x] Add dictionary creation (empty and with key-value pairs)
   - [x] Implement dictionary access (get value by key)
   - [x] Add dictionary modification (set value for key)
   - [x] Implement dictionary operations (len)
   - [x] Create comprehensive tests for dictionary operations
   - [x] Add advanced dictionary tests
10. [x] Enhance dictionary support
   - [x] Implement nested dictionary access
   - [x] Add support for mixed key and value types
   - [x] Implement dictionary methods (keys, values, items)
   - [x] Add support for dictionary comprehensions
   - [x] Implement membership testing with 'in' operator for dictionaries
   - [x] Improve dictionary integration with functions
11. [x] Fix remaining dictionary integration issues
   - [x] Fix type mismatch issues with dictionary function parameters
   - [x] Resolve bitcast issues with dictionary methods in functions
   - [x] Improve type inference for nested dictionaries
   - [x] Enhance typechecker to better handle dictionary indexing
12. [x] Implement a basic closure environment solution for nonlocal variables
    - [x] Create a closure environment structure to store nonlocal variables
    - [x] Pass the environment pointer to nested functions
    - [x] Update nonlocal variables in the environment
    - [x] Load nonlocal variables from the environment
    - [x] Pass nonlocal variables as parameters to nested functions

### Essential Built-ins
13. [x] Support for the range() built-in function
14. [x] Implement basic built-in functions (print, len, etc.)
    - [x] Implement print() function
    - [x] Implement input() function
    - [x] Implement len() function for strings and lists
      - [x] Basic len() function implementation
      - [x] Support for len() in expressions and control flow
      - [x] Fix len() function in nested function calls with parameters
      - [x] Fix LLVM dominance validation issues with function calls
      - [x] Extend len() function to support dictionaries and other collections
    - [x] Implement type conversion functions (int(), float(), bool(), str())
    - [ ] Implement other common built-in functions (min, max, etc.)
15. [ ] Implement string manipulation functions

### Testing Improvements
16. [x] Add comprehensive tests for dictionary operations
17. [x] Fix failing tests for nonlocal variables in nested functions
    - [x] Resolve LLVM dominance validation issues in simple tests
    - [x] Add test cases for basic nonlocal scenarios
    - [x] Ensure basic nonlocal variable tests pass
    - [x] Fix complex nonlocal variable tests (loops, conditionals, shadowing)
    - [x] Add more test cases for complex nonlocal scenarios
    - [x] Fix LLVM dominance validation issues in deeply nested functions
    - [x] Implement proper handling for variable shadowing in nested functions
18. [x] Fix failing tests for len() function
    - [x] Fix len() function in nested function calls with parameters
    - [x] Fix LLVM validation issues with function calls
    - [x] Implement proper handling for function calls with len()
19. [x] Implement exception handling structure (COMPLETED)
    - [x] Define exception runtime structure
    - [x] Implement basic exception creation and raising
    - [x] Implement simple try-except blocks
    - [x] Add structure for finally blocks
    - [x] Add basic tests for exception handling
    - [x] Implement proper exception propagation through the call stack
    - [x] Add support for nested exception handling
    - [x] Support exception variables (as in 'except Exception as e')
    - [x] Support try-except-else-finally blocks
    - [x] Fix function call parameter type mismatch in exception tests
    - [x] Fix basic block termination issues with return statements in try-except blocks
    - [x] Add simplified tests for raise and catch functionality
20. [x] Add more comprehensive test cases for new features
    - [x] Enable previously ignored tests for list comprehensions with string operations
    - [x] Enable previously ignored tests for list comprehensions with arithmetic operations
    - [x] Enable previously ignored tests for list comprehensions with membership operations
21. [ ] Add tests for edge cases in type conversions
22. [ ] Create a test suite for comparing compiled output with CPython execution
23. [ ] Add tests for error handling and recovery

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

- Research performance improvements for numeric operations
- Explore Symbol from JS and how it could be used in Cheetah.
- ✅ Implemented a robust solution for nonlocal variables using default values and proper dominance validation to handle LLVM's requirements
- ✅ Created an enhanced closure environment structure with support for nonlocal variables in deeply nested functions
- ✅ Implemented special handling for variable shadowing in nested functions
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
- Research how other compilers handle closure environments and variable capture
- Consider implementing a static analysis pass to identify all nonlocal variables before code generation
- Look into how Python's exception handling is implemented in CPython for inspiration
