// Runtime checks feature for Crabby
// This avoids runtime errors like recursion and loop problems
// and more code-related errors that are or may be fragile.

// compile.rs - Handles executing a '.crab' file
// runtime.rs - manages functions, stacks, etc

use std::collections::HashMap;
use crate::fs;
use std::pin::Pin;
use std::future::Future;

use crate::value::{Value, Function};
use crate::utils::CrabbyError;
use crate::parser::{parse, Program, Statement, Expression};
use std::path::PathBuf;
use crate::lexer::tokenize;
use crate::modules::Module;

pub struct Runtime {
    variables: HashMap<String, Value>,
    awaiting: Vec<Pin<Box<dyn Future<Output = Value>>>>,
    call_stack: Vec<String>,
    module: Module,
    loaded_functions: HashMap<String, Function>,
    current_file: Option<PathBuf>,
}

impl Runtime {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let mut runtime = Self {
            variables: HashMap::new(),
            loaded_functions: HashMap::new(),
            awaiting: Vec::new(),
            call_stack: Vec::new(),
            module: Module {
                public_items: HashMap::new(),
                private_items: HashMap::new(),
                variable: HashMap::new()
            },
            current_file: file_path
        };

        runtime.loaded_functions.insert("print".to_string(), Function {
            params: vec!["value".to_string()],
            body: Box::new(Statement::Expression(Expression::Variable("value".to_string()))),
        });

        runtime
    }

    pub async fn runtime(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.runtime_statement(statement).await?;
        }
        Ok(())
    }

    async fn handle_print(&mut self, args: &[Expression]) -> Result<Value, CrabbyError> {
        if args.len() != 1 {
            return Err(CrabbyError::CompileError("print takes exactly one argument".to_string()));
        }

        let value = self.runtime_expression(&args[0]).await?; // Add await here
        println!("{}", value.to_string());
        Ok(Value::Integer(0))
    }

    async fn handle_lambda_call(&mut self, lambda: Function, arguments: &[Expression]) -> Result<Value, CrabbyError> {
        for (param, arg) in lambda.params.iter().zip(arguments) {
            let arg_value = self.runtime_expression(arg).await?;
            self.variables.insert(param.clone(), arg_value);
        }

        if let Some(value) = self.runtime_statement(&lambda.body).await? {
            Ok(value)
        } else {
            Ok(Value::Void)
        }
    }

    async fn load_and_import_module(&self, _import_name: &str, import_path: &str) -> Result<Module, CrabbyError> {
        let current_file = self.current_file.as_ref().map(|p| p.as_path()).unwrap_or_else(|| std::path::Path::new("."));
        let resolved_path = Module::resolve_path(current_file, import_path);
        let source_code = fs::read_to_string(&resolved_path).map_err(|e| {
            CrabbyError::CompileError(format!("Failed to read module '{}': {}", resolved_path.display(), e))
        })?;
        let tokens = tokenize(&source_code).await?;
        let ast = parse(tokens).await?;
        let mut module_runtime = Runtime::new(Some(resolved_path.clone()));
        module_runtime.runtime(&ast).await?;
        Ok(module_runtime.module)
    }

    pub async fn runtime_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, CrabbyError> {
        match stmt {
            Statement::Let { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.runtime_expression(value).await?;

                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                self.variables.insert(var_name, compiled_value);
                Ok(None)
            },
            Statement::Var { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.runtime_expression(value).await?;

                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                self.variables.insert(var_name, compiled_value);
                Ok(None)
            },
            Statement::Const { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.runtime_expression(value).await?;

                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                self.variables.insert(var_name, compiled_value);
                Ok(None)
            },
            Statement::Import { name, source } => {
                if let Some(source_path) = source {
                    let module = self.load_and_import_module(name, source_path).await?;
                    if let Some(value) = module.public_items.get(name) {
                        self.variables.insert(name.clone(), value.clone());
                        Ok(None)
                    } else if module.private_items.contains_key(name) {
                        Err(CrabbyError::CompileError(format!(
                            "Cannot import private item '{}' from module",
                            name
                        )))
                    } else {
                        Err(CrabbyError::CompileError(format!(
                            "Item '{}' not found in module",
                            name
                        )))
                    }
                } else {
                    Err(CrabbyError::CompileError("Standard library imports not yet implemented".to_string()))
                }
            },
            Statement::Macro { name, params, body } => {
                self.variables.insert(name.clone(), Value::Lambda(Function {
                    params: vec![params.clone()],
                    body: Box::new(Statement::Expression(*(*body).clone())),
                }));
                Ok(None)
            },
            Statement::ForIn { variable, iterator, body } => {
                let iter_value = self.runtime_expression(iterator).await?;
                if let Value::Integer(n) = iter_value {
                    for i in 0..n {
                        self.variables.insert(variable.clone(), Value::Integer(i));
                        self.runtime_statement(body).await?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::CompileError("Iterator must be a range".to_string()))
                }
            },
            Statement::Enum { name, variants: _variants, where_clause: _ } => {
                let value = Value::String(format!("enum {}", name));
                self.variables.insert(name.clone(), value);
                Ok(None)
            },
            Statement::Struct { name, fields: _fields, where_clause: _where_clause } => {
                let value = Value::String(format!("struct {}", name));
                self.variables.insert(name.clone(), value);
                Ok(None)
            },
            _ => Ok(None)
        }
    }

    pub async fn runtime_expression(&mut self, expr: &Expression) -> Result<Value, CrabbyError> {
        match expr {
            Expression::Variable(name) => {
                self.variables.get(name).cloned().ok_or_else(|| {
                    CrabbyError::CompileError(format!("Undefined variable: {}", name))
                })
            },
            Expression::Await { expr } => {
                let value = self.runtime_expression(expr).await?;
                self.awaiting.push(value);
                Ok(Value::Void)
            },
            Expression::Call { function, arguments } => {
                let mut compiled_args = Vec::new();

                if self.call_stack.contains(function) {
                    return Err(CrabbyError::CompileError(format!(
                        "Recursion is not allowed: function '{}' calls itself", function
                    )));
                }
                self.call_stack.push(function.clone());

                for arg in arguments {
                    compiled_args.push(self.runtime_expression(arg).await?);
                }

                if function == "print" {
                    return self.handle_print(arguments).await;
                }

                if let Some(Value::Lambda(lambda)) = self.variables.get(function) {
                    return self.handle_lambda_call(lambda.clone(), arguments).await;
                }

                let func = self.loaded_functions.get(function).cloned().ok_or_else(|| {
                    CrabbyError::CompileError(format!("Undefined function: {}", function))
                })?;

                if arguments.len() != func.params.len() {
                    return Err(CrabbyError::CompileError(format!(
                        "Function {} expects {} arguments, got {}",
                        function,
                        func.params.len(),
                        arguments.len()
                    )));
                }

                let result = if let Some(Value::Lambda(lambda)) = self.variables.get(function) {
                    self.handle_lambda_call(lambda.clone(), arguments).await
                } else {
                    let func = self.loaded_functions.get(function).cloned().ok_or_else(|| {
                        CrabbyError::CompileError(format!("Undefined function: {}", function))
                    })?;
                    if arguments.len() != func.params.len() {
                        return Err(CrabbyError::CompileError(format!(
                            "Function {} expects {} arguments, got {}",
                            function,
                            func.params.len(),
                            arguments.len()
                        )));
                    }
                    let mut new_runtime = Runtime::new(None);
                    for (param, arg) in func.params.iter().zip(arguments) {
                        let arg_value = self.runtime_expression(arg).await?;
                        new_runtime.variables.insert(param.clone(), arg_value);
                    }
                    match new_runtime.runtime_statement(&func.body).await? {
                        Some(value) => Ok(value),
                        None => Ok(Value::Integer(0)),
                    }
                };
                self.call_stack.pop();
                result;

                Ok(Value::Void)
            },
            Expression::Lambda { params, body } => {
                Ok(Value::Lambda(Function {
                    params: params.clone(),
                    body: body.clone(),
                }))
            },
            _ => Ok(Value::Void)
        }
    }
}
