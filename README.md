# Crowbar Rust Code Editor with AST Manipulation
![image](https://github.com/user-attachments/assets/0aca50ad-b57f-4924-a874-1e32b192ea17)


Crowbar is a lightweight Rust code editor with a built-in Abstract Syntax Tree (AST) manipulation tool. The editor allows you to load, modify, and run Rust code directly within the interface. It also provides real-time feedback on variable values through a visual inspector, making it easier to understand and debug Rust programs.

## Features

- **Code Editor**: Write and edit Rust code with syntax highlighting.
- **AST Manipulation**: Analyze and modify code using the AST. The editor allows you to update variable values dynamically before running the code.
- **Variable Inspector**: Displays variables of basic types (integers, floats, booleans, strings) and allows you to modify their values.
- **Code Execution**: Compile and run the Rust code directly within the editor. The output of the code execution is displayed in real-time.
- **Support for Constants and Strings**: The editor can handle and display constants and strings in the variable inspector.

## Usage

1. **Load a File**: Click on "Load File" to open a Rust file that you want to edit and run.
2. **Edit Code**: Modify the code directly in the editor. You can change variable values or add new lines of code.
3. **Inspect Variables**: The variable inspector below the code editor shows all variables in your code. You can modify their values and see the changes reflected immediately.
4. **Run Code**: Click "Run Code" to compile and execute the Rust code. The output will be displayed in the "Output" section below the variable inspector.

## Example

`sample.rs` is used as an example Rust file to demonstrate the features of Crowbar. The file contains a simple Rust program that calculates the sum of two numbers and prints the result. You can load this file into Crowbar to see how the editor works.

## Requirements

Rust: Make sure you have Rust installed on your machine to compile and run the code.
