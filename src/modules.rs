// Module handler for Crabby's import && export system

use std::collections::HashMap;
use crate::fs;
use std::path::{Path, PathBuf};

use crate::value::Value;
use crate::utils::CrabbyError;
use crate::parse;
use crate::compile::Compiler;
use crate::lexer::tokenize;

#[derive(Clone)]
pub struct Module {
    pub variable: HashMap<String, Value>,
    pub public_items: HashMap<String, Value>,
    pub private_items: HashMap<String, Value>
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
            Err(CrabbyError::CompileError(format!(
                "Cannot import private item '{}' from module",
                item_name
            )))
        } else {
            Err(CrabbyError::CompileError(format!(
                "Item '{}' not found in module",
                item_name
            )))
        }
    }

    pub fn resolve_path(&self, current_file: &Path, import_path: &str) -> PathBuf {
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

    pub async fn load_module(&mut self, current_file: &Path, _name: &str, source: &str) -> Result<(), CrabbyError> {
        let resolved_path = self.resolve_path(current_file, source);
        let source_code = fs::read_to_string(&resolved_path)?;
        let tokens = tokenize(&source_code).await?;
        let ast = parse(tokens).await?;
        let mut module_compiler = Compiler::new(Some(resolved_path));
        module_compiler.compile(&ast).await?;
        Ok(())
    }
}
