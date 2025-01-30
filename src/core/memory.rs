// Crabby's Memory Management and Safety

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

use crate::parser::ast::{Expression, Statement};

#[derive(Debug, Clone)]
pub struct Memory {
    pub globals: HashMap<String, Value>,
    pub stack: Vec<Value>,
    pub scopes: Vec<Scope>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            stack: Vec::new(),
            scopes: Vec::new(),
        }
    }

    fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    fn get_global_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.globals.get_mut(name)
    }
}