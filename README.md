# Cheetah Programming Language

<div align="center">
  <img src="https://via.placeholder.com/200x200?text=Cheetah" alt="Cheetah Logo" width="200" height="200">
  <h3>A fast, Python-like programming language with AOT compilation</h3>
</div>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Cheetah is a programming language with Python-like syntax that compiles to native code using LLVM. It aims to provide the simplicity and readability of Python with the performance benefits of ahead-of-time (AOT) compilation.

## Features

- **Python-like Syntax**: Familiar and easy to learn for Python developers
- **AOT Compilation**: Compiles to native executables for maximum performance
- **JIT Compilation**: Supports Just-In-Time compilation for development and testing
- **REPL**: Interactive Read-Eval-Print Loop for quick experimentation
- **Type Checking**: Built-in type checking system
- **Exception Handling**: Comprehensive try-except-else-finally blocks
- **Core Language Features**: Functions, closures, loops, conditionals, and more

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/cheetah.git
cd cheetah

# Build and install
./install.sh
```

The installation script will:
1. Build the Cheetah CLI in release mode
2. Install the CLI binary to `/usr/local/bin/cheetah`
3. Install the runtime library for AOT linking to `/usr/local/lib/cheetah`

## Usage

### Running Cheetah Programs

Create a file with the `.ch` extension:

```python
# hello.ch
print("Hello, World!")
```

Run it using the Cheetah command:

```bash
cheetah hello.ch
```

### Building Executables

Compile a Cheetah program to an executable:

```bash
cheetah build hello.ch
```

This will create a native executable in the `.cheetah_build` directory.

### Interactive REPL

Start an interactive REPL session:

```bash
cheetah repl
```

Use the `-j` flag to enable JIT compilation in the REPL:

```bash
cheetah repl -j
```

### Additional Commands

- **Lexical Analysis**: `cheetah lex file.ch`
- **Parsing**: `cheetah parse file.ch`
- **Type Checking**: `cheetah check file.ch`
- **Code Formatting**: `cheetah format file.ch`
- **LLVM IR Generation**: `cheetah compile file.ch`

## Language Examples

### Hello World

```python
# hello.ch
print("Hello, World!")
```

### Functions

```python
# Simple function
def greet():
    return "Hello, World!"

# Function with parameters
def add(x, y):
    return x + y

# Function with default parameters
def greet_person(name, greeting="Hello"):
    return greeting + ", " + name + "!"

# Recursive function
def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)
```

### Control Flow

```python
# If statements
x = 10
if x > 5:
    print("x is greater than 5")
elif x == 5:
    print("x is equal to 5")
else:
    print("x is less than 5")

# While loops
count = 0
while count < 5:
    print(count)
    count = count + 1

# For loops
for i in range(5):
    print(i)
```

### Data Structures

```python
# Lists
numbers = [1, 2, 3, 4, 5]
print(numbers[0])  # Access first element
numbers[0] = 10    # Modify element
combined = numbers + [6, 7, 8]  # Concatenate lists

# Dictionaries
person = {"name": "Alice", "age": 30}
print(person["name"])  # Access value
person["email"] = "alice@example.com"  # Add new key-value pair

# List comprehensions
squares = [x * x for x in range(10)]
even_squares = [x * x for x in range(10) if x % 2 == 0]
```

### Exception Handling

```python
try:
    x = 10 / 0
except ZeroDivisionError:
    print("Cannot divide by zero")
except:
    print("Some other error occurred")
else:
    print("No error occurred")
finally:
    print("This always executes")
```

## Project Status

Cheetah is under active development and there is much left to be done. Current focus areas include:

- Performance optimizations
- Module support and imports
- Extending built-in functions
- Improving testing infrastructure

See the [TODO.md](TODO.md) file for a detailed list of planned features and improvements.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- LLVM for providing the compilation backend
- Python for inspiring the language syntax and semantics
