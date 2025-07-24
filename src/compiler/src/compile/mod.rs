use crate::value::Value;
use crate::vm::{Instruction, VM};

pub struct Compiler {
    constants: Vec<Value>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
        }
    }

        
}

