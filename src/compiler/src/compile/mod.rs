use crate::value::Value;
use crate::vm::{Instructions, VM};

pub struct Compiler {
    constants: Vec<Value>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
        }
    }

    pub fn get_constants(&self) -> Vec<Value> {
        self.constants.clone()
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn compile()
}
