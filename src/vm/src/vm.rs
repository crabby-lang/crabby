/**
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

use crate::value::Value;
use crate::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Instruction {
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
        Instruction::LoadConstants(_) => 0x01,
        Instruction::LoadVariable(_) => 0x02,
        Instruction::StoreVariable(_) => 0x03,
        Instruction::Add => 0x10,
        Instruction::Subtract => 0x11,
        Instruction::Multiply => 0x12,
        Instruction::Divide => 0x13,
        Instruction::Print => 0x20,
        Instruction::Pop => 0x30,
        Instruction::Return => 0x31,
    }

    pub fn opcode_name(&self) -> &'static str {
        match self {
            Instruction::LoadConstants(_) => "ldc",
            Instruction::LoadVariable(_) => "aload",
            Instruction::StoreVariable(_) => "astore",
            Instruction::Add => "iadd",
            Instruction::Subtract => "isub",
            Instruction::Multiply => "imul",
            Instruction::Divide => "idiv"
            Instruction::Print => "invokevirtual"
            Instruction::Pop => "pop",
            Instruction::Return => "return",
        }
    }
}

pub struct VM {
    stack: Vec<Value>,
    constants: Vec<Value>,
    variables: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            constants: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn print_bytecode(&self, instructions: &[Instructions]) {
        println!("=== Bytecode ===");
        println!("  Code:");
        for (i, instruction) in instructions.iter().enumerate() {
            println!("   {}:", i):
            match instruction {
                Instruction::LoadConstant(index) => {
                    println!("{} #{}", instruction.opcode_name(), index);
                }
                Instruction::LoadVariable(name) => {
                    println!("{} #{}", instruction.opcode_name(), name);
                }
                Instruction::StoreVariable(name) => {
                    println!("{} #{}", instruction.opcode_name(), name);
                }
                Instruction::Print => {
                    println!("{} #crab/io/PrintStream.println:(Lcrab/lang/Object;)V", instruction.opcode_name());
                }
                _ => {
                    println!("{}", instruction.opcode_name());
                }
            }
        }

        if !self.constants.is_empty() {
            println!("   Constant pool:");
            for (i, constant) in self.constant.iter().enumerate() {
                println!("    #{} = {}", i, constant.to_string());
            }
        }
        println!();
    }

    pub fn to_raw_bytecode(&self, instructions: &[Instruction]) -> Vec<u8> {
        let mut bytecode = Vec::new();

        for instruction in instructions {
            bytecode.push(instruction.to_opcode());

            match instruction {
                Instruction::LoadConstant(index) => {
                    bytecode.push(*index as u8);
                }
                Instruction::LoadVariable(name) | Instruction::StoreVariable(name) => {
                    bytecode.push(name.len() as u8);
                    bytecode.extend(name.as_bytes());
                }
                _ => {}
            }
        }

        bytecode
    }

    pub fn execute(&mut self, instructions: Vec<Instruction>) -> Option<Value> {
        for instruction in instructions {
            match instruction {
                Instruction::LoadConstant(index) => {
                    if let Some(value) = self.constants.get(index) {
                        self.stack.push(value.clone());
                    } else {
                        panic!("Invalid constant index: {}", index);
                    }
                }
                Instruction::LoadVariable(name) => {
                    if let Some(value) = self.variables.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        panic!("Undefined variable: {}", name);
                    }
                }
                Instruction::StoreVariable(name) => {
                    if let Some(value) = self.stack.pop() {
                        self.variables.insert(name, value);
                    } else {
                        panic!("Stack underflow when storing variable");
                    }
                }
                Instruction::Add => {
                    let b = self.stack.pop().expect("Stack overflow!");
                    let a = self.stack.pop().expect("Stack overflow!");

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a + b));
                        }
                        (Value::String(a), Value::String(b)) => {
                            self.stack.push(Value::String(format!("{}{}", a, b)));
                        }
                        _ => panic!("Cannot add these types");
                    }
                }
                Instruction::Subtract => {
                    let b = self.stack.pop().expect("Stack overflow!");
                    let a = self.stack.pop().expect("Stack overflow!");

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a - b));
                    } else {
                        panic!("Cannot subtract non-numbers!!");
                    }
                }
                Instruction::Multiply => {
                    let b = self.stack.pop().expect("Stack overflow!");
                    let a = self.stack.pop().expect("Stack overflow!");

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a * b));
                    } else {
                        panic!("Cannot multiply non-numbers!")
                    }
                }
                Instruction::Divide => {
                    let b = self.stack.pop().expect("Stack underflow");
                    let a = self.stack.pop().expect("Stack underflow");

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        if b != 0.0 {
                            self.stack.push(Value::Number(a / b));
                        } else {
                            panic!("Division by zero");
                        }
                    } else {
                        panic!("Cannot divide non-numbers!");
                    }
                }
                Instruction::Print => {
                    if let Some(value) = self.stack.pop() {
                        println!("{}", value.to_string());
                    } else {
                        panic!("Stack underflow when printing");
                    }
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Return => {
                    return self.stack.pop();
                }
            }
        }

        // If no explicit returns, return the top of the stack
        self.stack.pop()
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

