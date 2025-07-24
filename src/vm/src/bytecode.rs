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

use crate::vm::Instruction;
use crate::value::Value;
use std::fs::File;
use std::io::{Read, Write, BufWriter, BufReader};

// Bytecode Crabby File Format (.cby)
// Magic Number: "CRAB" (0x43524142)
// Version: 1 byte
// Constants count: 4 bytes (little endian)
// Constants section: VARIABLE LENGTH
// Instruction count: 4 bytes (little endian)
// Instruction section: VARIABLE LENGTH

pub struct BytecodeFile {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
}

impl BytecodeFile {
    const MAGIC: &'static [u8] = b"CRAB";
    const VERSION: u8 = 1;

    pub fn new(instructions: Vec<Instruction>, constants: Vec<Value>) -> Self {
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



    }

}
