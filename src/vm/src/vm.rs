/*
* MIT License
*
* Copyright (c) 2024 Kazooki123
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*
*/

use crate::value::ValueVM;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Instructions {
    LoadConstant(usize),
    LoadVariable(String),
    StoreVariable(String),
    Print,
    Add,
    Subtract,
    Multiply,
    Divide,
    Pop,
    Return,
}

// A set of Instructions
impl Instructions {
    pub fn to_opcode(&self) -> u8 {
        match self {
            Instructions::LoadConstant(_) => 0x01,
            Instructions::LoadVariable(_) => 0x02,
            Instructions::StoreVariable(_) => 0x03,
            Instructions::Add => 0x10,
            Instructions::Subtract => 0x11,
            Instructions::Multiply => 0x12,
            Instructions::Divide => 0x13,
            Instructions::Print => 0x20,
            Instructions::Pop => 0x30,
            Instructions::Return => 0x31,
        }
    }

    pub fn opcode_name(&self) -> &'static str {
        match self {
            Instructions::LoadConstant(_) => "ldc",
            Instructions::LoadVariable(_) => "aload",
            Instructions::StoreVariable(_) => "astore",
            Instructions::Add => "iadd",
            Instructions::Subtract => "isub",
            Instructions::Multiply => "imul",
            Instructions::Divide => "idiv",
            Instructions::Print => "invokevirtual",
            Instructions::Pop => "pop",
            Instructions::Return => "return",
        }
    }
}

pub struct VM {
    pub stack: Vec<ValueVM>,
    pub constants: Vec<ValueVM>,
    pub variables: HashMap<String, ValueVM>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            constants: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn print_bytecode(&self, instructions: &[Instructions]) {
        println!("=== Bytecode ===");
        println!("  Code:");
        for (i, instruction) in instructions.iter().enumerate() {
            println!("   {}:", i);
            match instruction {
                Instructions::LoadConstant(index) => {
                    println!("{} #{}", instruction.opcode_name(), index);
                }
                Instructions::LoadVariable(name) => {
                    println!("{} #{}", instruction.opcode_name(), name);
                }
                Instructions::StoreVariable(name) => {
                    println!("{} #{}", instruction.opcode_name(), name);
                }
                Instructions::Print => {
                    println!(
                        "{} #crab/lib/std/Print.print:(Lcrab/lang/Object;)V",
                        instruction.opcode_name()
                    );
                }
                _ => {
                    println!("{}", instruction.opcode_name());
                }
            }
        }

        if !self.constants.is_empty() {
            println!("   Constant pool:");
            for (i, constant) in self.constants.iter().enumerate() {
                println!("    #{} = {}", i, constant.to_string());
            }
        }
        println!();
    }

    pub fn to_raw_bytecode(&self, instructions: &[Instructions]) -> Vec<u8> {
        let mut bytecode = Vec::new();

        for instruction in instructions {
            bytecode.push(instruction.to_opcode());

            match instruction {
                Instructions::LoadConstant(index) => {
                    bytecode.push(*index as u8);
                }
                Instructions::LoadVariable(name) | Instructions::StoreVariable(name) => {
                    bytecode.push(name.len() as u8);
                    bytecode.extend(name.as_bytes());
                }
                _ => {}
            }
        }

        bytecode
    }

    pub fn execute(&mut self, instructions: &[Instructions]) -> Option<ValueVM> {
        for instruction in instructions {
            match instruction {
                Instructions::LoadConstant(index) => {
                    if let Some(value) = self.constants.get(*index) {
                        self.stack.push(value.clone());
                    } else {
                        panic!("Invalid constant index: {}", index);
                    }
                }
                Instructions::LoadVariable(name) => {
                    if let Some(value) = self.variables.get(name) {
                        self.stack.push(value.clone());
                    } else {
                        panic!("Undefined variable: {}", name);
                    }
                }
                Instructions::StoreVariable(name) => {
                    if let Some(value) = self.stack.pop() {
                        self.variables.insert(name.to_string(), value);
                    } else {
                        panic!("Stack underflow when storing variable");
                    }
                }
                Instructions::Add => {
                    let b = self.stack.pop().expect("Stack overflow!");
                    let a = self.stack.pop().expect("Stack overflow!");

                    match (a, b) {
                        (ValueVM::Number(a), ValueVM::Number(b)) => {
                            self.stack.push(ValueVM::Number(a + b));
                        }
                        (ValueVM::String(a), ValueVM::String(b)) => {
                            self.stack.push(ValueVM::String(format!("{}{}", a, b)));
                        }
                        _ => panic!("Cannot add these types"),
                    }
                }
                Instructions::Subtract => {
                    let b = self.stack.pop().expect("Stack overflow!");
                    let a = self.stack.pop().expect("Stack overflow!");

                    if let (ValueVM::Number(a), ValueVM::Number(b)) = (a, b) {
                        self.stack.push(ValueVM::Number(a - b));
                    } else {
                        panic!("Cannot subtract non-numbers!!");
                    }
                }
                Instructions::Multiply => {
                    let b = self.stack.pop().expect("Stack overflow!");
                    let a = self.stack.pop().expect("Stack overflow!");

                    if let (ValueVM::Number(a), ValueVM::Number(b)) = (a, b) {
                        self.stack.push(ValueVM::Number(a * b));
                    } else {
                        panic!("Cannot multiply non-numbers!")
                    }
                }
                Instructions::Divide => {
                    let b = self.stack.pop().expect("Stack underflow");
                    let a = self.stack.pop().expect("Stack underflow");

                    if let (ValueVM::Number(a), ValueVM::Number(b)) = (a, b) {
                        if b != 0.0 {
                            self.stack.push(ValueVM::Number(a / b));
                        } else {
                            panic!("Division by zero");
                        }
                    } else {
                        panic!("Cannot divide non-numbers!");
                    }
                }
                Instructions::Print => {
                    if let Some(value) = self.stack.pop() {
                        println!("{}", value.to_string());
                    } else {
                        panic!("Stack underflow when printing");
                    }
                }
                Instructions::Pop => {
                    self.stack.pop();
                }
                Instructions::Return => {
                    return self.stack.pop();
                }
            }
        }

        // If no explicit returns, return the top of the stack
        self.stack.pop()
    }

    pub fn add_constant(&mut self, value: ValueVM) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
