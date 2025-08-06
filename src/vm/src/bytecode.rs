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

use crate::vm::Instructions;
use crate::value::ValueVM;
use std::fs::File;
use std::io::{Read, Write, BufWriter, BufReader};

// Bytecode Crabby File Format (.cby)
// Magic Number: "CRAB" (0x43524142)
// Version: 1 byte
// Constants count: 4 bytes (little endian)
// Constants section: VARIABLE LENGTH
// Instructions count: 4 bytes (little endian)
// Instructions section: VARIABLE LENGTH

pub struct BytecodeFile {
    pub instructions: Vec<Instructions>,
    pub constants: Vec<ValueVM>,
}

impl BytecodeFile {
    const MAGIC: &'static [u8] = b"CRAB";
    const VERSION: u8 = 1;

    pub fn new(instructions: Vec<Instructions>, constants: Vec<ValueVM>) -> Self {
        Self {
            instructions,
            constants,
        }
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        // Writes the Magic number in the VM
        writer.write_all(Self::MAGIC)?;

        // Writes the version
        writer.write_all(&[Self::VERSION])?;

        // Writes the constants version
        writer.write_all(&(self.constants.len() as u32).to_le_bytes())?;
        for constant in &self.constants {
            self.write_value(&mut writer, constant)?;
        }

        // Writes the instruction section
        writer.flush()?;
        println!("Bytecode file format:");
        println!("  Magic: {:?}", std::str::from_utf8(Self::MAGIC).unwrap());
        println!("  Version: {}", Self::VERSION);
        println!("  Constants: {} items", self.constants.len());
        println!("  Instructions: {} items", self.instructions.len());

        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn  std::error::Error>> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);

        // Reads and verifies the MAGIC number
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != Self::MAGIC {
            return Err("Invalid bytecode file: Wrong magic number!".into());
        }

        // Reads and verifies VERSION
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != Self::VERSION {
            return Err(format!("Unsupported bytecode version: {}", version[0]).into());
        }

        // Reads constants
        let mut constants_count = [0u8; 4];
        reader.read_exact(&mut constants_count)?;
        let contants_count = u32::from_le_bytes(constants_count);

        let mut constants = Vec::new();
        for _ in 0..constants_count {
            constants.push(self.read_value(&mut reader)?);
        }

        // Reads instructions
        let mut instructions_count = [0u8; 4];
        reader.read_exact(&mut instructions_count)?;
        let instructions_count = u32::from_le_bytes(instructions_count);

        let mut instructions = Vec::new();
        for _ in 0..instructions_count {
            instructions.push(self.read_instructions(&mut reader)?);
        }

        Ok(Self {
            instructions,
            constants,
        })

    }

    fn write_value(&self, writer: &mut BufWriter<File>, value: &ValueVM) -> Result<(), Box<dyn std::error::Error>> {
        match value {
            ValueVM::Number(n) => {
                writer.write_all(&[0x01])?;
                writer.write_all(&n.to_le_bytes())?;
            }
            ValueVM::String(s) => {
                writer.write_all(&[0x02])?;
                writer.write_all(&(s.len() as u32).to_le_bytes())?;
                writer.write_all(s.as_bytes())?;
            }
            ValueVM::Boolean(b) => {
                writer.write_all(&[0x03])?;
                writer.write_all(&[if *b { 1 } else { 0 }])?;
            }
            ValueVM::Nil => {
                writer.write_all(&[0x04])?; // returns nothing
            }
        }
        Ok(())
    }

    fn read_value(&self, reader: &mut BufReader<File>) -> Result<(), Box<dyn std::error::Error>> {
        let mut type_tag = [0u8; 1];
        reader.read_exact(&mut type_tag)?;

        match type_tag[0] {
            0x01 => {
                let mut bytes = [0u8; 8];
                reader.read_exact(&mut bytes)?;
                Ok(ValueVM::Number(f64::from_le_bytes(bytes)))
            }
            0x02 => {
                let mut len_bytes = [0u8; 4];
                reader.read_exact(&mut len_bytes)?;
                let len = u32::from_le_bytes(len_bytes) as usize;

                let mut string_bytes = vec![0u8; len];
                reader.read_exact(&mut string_bytes)?;
                Ok(ValueVM::String(String::from_utf8(string_bytes)?))
            }
            0x03 => {
                let mut bool_bytes = [0u8; 1];
                reader.read_exact(&mut bool_bytes)?;
                Ok(ValueVM::Boolean(bool_bytes[0] != 0))
            }
            0x04 => Ok(ValueVM::Nil),
            _ => Err(format!("Unknown value type tag: {}", type_tag[0]).into()),
        }
    }

    fn write_instruction(&self, writer: &mut BufWriter<File>, instruction: &Instructions) -> Result<(), Box<dyn std::error::Error>> {
        writer.write_all(&[instruction.to_opcode()])?;

        match instruction {
            Instructions::LoadConstant(index) => {
                writer.write_all(&(*index as u32).to_le_bytes())?;
            }
            Instructions::LoadVariable(name) | Instructions::StoreVariable(name) => {
                writer.write_all(&(name.len() as u32).to_le_bytes())?;
                writer.write_all(name.as_bytes())?;
            }
            _ => {} // No additional data
        }
        Ok(())
    }

    fn read_instruction(&self, reader: &mut BufReader<File>) -> Result<Instructions, Box<dyn std::error::Error>> {
        let mut opcode = [0u8; 1];
        reader.read_exact(&mut opcode)?;

        match opcode[0] {
            0x01 => {
                let mut index_bytes = [0u8; 4];
                reader.read_exact(&mut index_bytes)?;
                let index = u32::from_le_bytes(index_bytes);
                Ok(Instructions::LoadConstant(index))
            }
            0x02 => {
                let mut len_bytes = [0u8; 4];
                reader.read_exact(&mut len_bytes)?;
                let len = u32::from_le_bytes(len_bytes) as usize;

                let mut name_bytes = vec![0u8; len];
                reader.read_exact(&mut name_bytes)?;
                Ok(Instructions::LoadVariable(String::from_utf8(name_bytes)?))
            }
            0x03 => {
                let mut len_bytes = [0u8; 4];
                reader.read_exact(&mut len_bytes)?;
                let len = u32::from_le_bytes(len_bytes) as usize;

                let mut name_bytes = vec![0u8; len];
                reader.read_exact(&mut name_bytes)?;
                Ok(Instructions::StoreVariable(String::from_utf8(name_bytes)?))
            }
            0x10 => Ok(Instructions::Add),
            0x11 => Ok(Instructions::Subtract),
            0x12 => Ok(Instructions::Multiply),
            0x13 => Ok(Instructions::Divide),
            0x20 => Ok(Instructions::Print),
            0x30 => Ok(Instructions::Pop),
            0x31 => Ok(Instructions::Return),
            _ => Err(format!("Unknown Instructions OPCODE: {}", opcode[0]).into()),
        }
    }
}
