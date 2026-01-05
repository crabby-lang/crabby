// Module handler for Crabby's import && export system

use std::collections::HashMap;
use crate::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::value::Value;
use crate::utils::CrabbyError;
use crate::parser::*;
use crate::interpreter::Interpreter;
use crate::lexer::tokenize;

pub struct ModuleCache {
    loaded: HashMap<PathBuf, Arc<Module>>,
    loading: HashSet<PathBuf>,
}

#[derive(Clone)]
pub struct Module {
    // pub variable: HashMap<String, Value>,
    pub public_items: HashMap<String, Value>,
    // pub private_items: HashMap<String, Value>
    pub exports: HashMap<String, Value>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            public_items: HashMap::new(),
            private_items: HashMap::new(),
            variable: HashMap::new()
        }
    }

    pub fn import_item(&mut self, module: &Module, item_name: &str) -> Result<(), CrabbyError> {
        if let Some(value) = module.public_items.get(item_name) {
            self.variable.insert(item_name.to_string(), value.clone());
            Ok(())
        } else if module.private_items.contains_key(item_name) {
            Err(CrabbyError::InterpreterError(format!(
                "Cannot import private item '{}' from module",
                item_name
            )))
        } else {
            Err(CrabbyError::InterpreterError(format!(
                "Item '{}' not found in module",
                item_name
            )))
        }
    }

    pub fn resolve_path(current_file: &Path, import_path: &str) -> PathBuf {
        if let Some(current_dir) = current_file.parent() {
            if import_path.starts_with("./") {
                // Handle explicit relative path
                current_dir.join(&import_path[2..])
            } else if import_path.starts_with("../") {
                // Handle parent directory reference
                current_dir.join(import_path)
            } else {
                // Handle implicit relative path
                current_dir.join(import_path)
            }
        } else {
            // Fallback to current directory if no parent
            PathBuf::from(import_path)
        }
    }

    pub async fn load_module(cache: &mut ModuleCache, current_file: &Path, source: &str) -> Result<Arc<Module>, CrabbyError> {
        let resolved_path = Module::resolve_path(current_file, source);

        if let Some(module) = cache.loaded.get(&resolved_path) {
            return Ok(module.clone());
        }

        if cache.loading.contains(&resolved_path) {
            return Err(CrabbyError::InterpreterError("Cyclic module import detected!").to_string());
        }

        cache.loading.insert(resolved_path.clone());
    }
}
