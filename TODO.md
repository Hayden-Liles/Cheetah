# Cheetah Project To-Do List

This document tracks tasks, features, and improvements for the Cheetah Python compiler project.

## How to Use This List
- Completed tasks are marked with `[x]`
- Incomplete tasks are marked with `[ ]`
- Tasks are organized by priority and category
- Add new tasks at the bottom of the appropriate section

## High Priority

### Parser Improvements
- [x] Fix list comprehension parsing issues
- [x] Enable large input parser tests
- [x] Improve error messages for syntax errors
- [x] Fix nested parentheses parsing in expressions
- [ ] Add better recovery from parsing errors

### Compiler Core
- [ ] Implement proper type checking system
- [ ] Add support for all binary operations
- [ ] Implement string concatenation
- [ ] Support for recursive function calls
- [ ] Implement proper variable scoping

### Language Features
- [ ] Complete implementation of for loops
- [ ] Support for the range() built-in function
- [ ] Implement list operations
- [ ] Add dictionary support
- [ ] Implement basic built-in functions (print, str, int, etc.)

## Medium Priority

### Optimizations
- [ ] Implement constant folding
- [ ] Add dead code elimination
- [ ] Optimize numeric operations
- [ ] Improve memory management

### Testing
- [ ] Add more comprehensive test cases
- [ ] Create benchmarks for performance testing
- [ ] Implement integration tests with real Python code
- [ ] Add tests for edge cases in type conversions

### Documentation
- [ ] Document the compiler architecture
- [ ] Create user guide for the language features
- [ ] Add inline documentation for key functions
- [ ] Create examples of supported syntax

## Low Priority

### Additional Features
- [ ] Support for classes and objects
- [ ] Implement exception handling
- [ ] Add support for modules and imports
- [ ] Implement more advanced Python features (generators, decorators)
- [ ] Add support for Python standard library modules

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

## Recently Completed
- [x] Fix comprehension_conditions5 test
- [x] Enable previously ignored large input parser tests
- [x] Create this TODO list

