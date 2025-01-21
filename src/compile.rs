use crate::utils::CrabbyError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::fs;
pub use crate::parser::ast::*;
pub use crate::parser::parser::*;
use crate::lexer::*;

pub struct Compiler {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    module: Module,
    current_file: Option<PathBuf>,
}

#[derive(Clone)]
enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Lambda(Function),
    Boolean(bool),
}

impl Value {
    fn to_string(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Lambda(_) => "<lambda>".to_string(),
            Value::Boolean(b) => b.to_string(),
        }
    }
}

#[derive(Clone)]
struct Function {
    params: Vec<String>,
    body: Box<Statement>,
}

#[derive(Clone)]
struct Module {
    public_items: HashMap<String, Value>,
    private_items: HashMap<String, Value>,
}

impl Compiler {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let mut compiler = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            module: Module {
                public_items: HashMap::new(),
                private_items: HashMap::new(),
            },
            current_file: file_path,
        };

        // Add built-in print function
        compiler.functions.insert("print".to_string(), Function {
            params: vec!["value".to_string()],
            body: Box::new(Statement::Expression(Expression::Variable("value".to_string()))),
        });

        compiler
    }

    fn new_module() -> Module {
        Module {
            public_items: HashMap::new(),
            private_items: HashMap::new(),
        }
    }

    fn compile_function_def(&mut self, name: &str, params: &[String], body: &Statement) -> Result<(), CrabbyError> {
        let is_public = name.starts_with("pub ");
        let func_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        let function = Function {
            params: params.to_vec(),
            body: Box::new(body.clone()),
        };

        if is_public {
            self.module.public_items.insert(func_name, Value::Lambda(function));
        } else {
            self.module.private_items.insert(func_name, Value::Lambda(function));
        }

        Ok(())
    }

    fn compile_let_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        let compiled_value = self.compile_expression(value)?;

        if is_public {
            self.module.public_items.insert(var_name, compiled_value);
        } else {
            self.module.private_items.insert(var_name, compiled_value);
        }

        Ok(())
    }

    fn import_item(&mut self, module: &Module, item_name: &str) -> Result<(), CrabbyError> {
        if let Some(value) = module.public_items.get(item_name) {
            self.variables.insert(item_name.to_string(), value.clone());
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

    fn handle_lambda_call(&mut self, lambda: Function, arguments: &[Expression]) -> Result<Value, CrabbyError> {
        if arguments.len() != lambda.params.len() {
            return Err(CrabbyError::CompileError(format!(
                "Lambda expects {} arguments, got {}",
                lambda.params.len(),
                arguments.len()
            )));
        }

        for (param, arg) in lambda.params.iter().zip(arguments) {
            let arg_value = self.compile_expression(arg)?;
            self.variables.insert(param.clone(), arg_value);
        }

        match self.compile_statement(&lambda.body)? {
            Some(value) => Ok(value),
            None => Ok(Value::Integer(0)),
        }
    }

    fn handle_print(&mut self, args: &[Expression]) -> Result<Value, CrabbyError> {
        if args.len() != 1 {
            return Err(CrabbyError::CompileError("print takes exactly one argument".to_string()));
        }

        let value = self.compile_expression(&args[0])?;
        println!("{}", value.to_string());
        Ok(Value::Integer(0))
    }

    pub fn compile(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_path(&self, current_file: &Path, import_path: &str) -> PathBuf {
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

    fn load_module(&mut self, current_file: &Path, _name: &str, source: &str) -> Result<(), CrabbyError> {
        let resolved_path = self.resolve_path(current_file, source);

        println!("Trying to load: {}", resolved_path.display());
        
        // Try to read the source file
        let source_code = fs::read_to_string(&resolved_path).map_err(|e| {
            CrabbyError::CompileError(format!(
                "Failed to read module '{}': {} (resolved path: {})", 
                source, 
                e,
                resolved_path.display()
            ))
        })?;

        // Tokenizes and parses the imported file
        let tokens = tokenize(&source_code)?;
        let ast = parse(tokens)?;

        // Creates a new compiler instance for the module
        let mut module_compiler = Compiler::new(Some(resolved_path));
        module_compiler.compile(&ast)?;
    
        // Only exposes public functions
        for (name, value) in module_compiler.module.public_items {
            self.variables.insert(name, value);
        }

        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<Option<Value>, CrabbyError> {
        match statement {
            Statement::FunctionDef { name, params, body } => {
                let is_public = name.starts_with("pub ");
                let func_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let function = Function {
                    params: params.clone(),
                    body: body.clone(),
                };

                if is_public {
                    self.module.public_items.insert(func_name.clone(), Value::Lambda(function.clone()));
                } else {
                    self.module.private_items.insert(func_name.clone(), Value::Lambda(function.clone()));
                }

                self.functions.insert(func_name, function);

                Ok(None)
            }
            Statement::Let { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.compile_expression(value)?;

                // Store in both places - module for exports and variables for local use
                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                // Always add to variables for local use
                self.variables.insert(var_name, compiled_value);

                Ok(None)
            }
            Statement::Return(expr) => {
                let value = self.compile_expression(expr)?;
                Ok(Some(value))
            }
            Statement::Loop { count, body } => {
                let count_value = self.compile_expression(count)?;
                if let Value::Integer(n) = count_value {
                    for _ in 0..n {
                        self.compile_statement(body)?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::CompileError("Loop count must be an integer".to_string()))
                }
            }
            Statement::ForIn { variable, iterator, body } => {
                let iter_value = self.compile_expression(iterator)?;
                if let Value::Integer(n) = iter_value {
                    for i in 0..n {
                        self.variables.insert(variable.clone(), Value::Integer(i));
                        self.compile_statement(body)?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::CompileError("Iterator must be a range".to_string()))
                }
            }
            Statement::Enum { name, variants, where_clause } => {
                // For now, just store enum definition in variables
                let value = Value::String(format!("enum {}", name));
                self.variables.insert(name.clone(), value);
                Ok(None)
            }
            Statement::Struct { name, fields, where_clause } => {
                // For now, just store struct definition in variables
                let value = Value::String(format!("struct {}", name));
                self.variables.insert(name.clone(), value);
                Ok(None)
            }
            Statement::Import { name, source } => {
                if let Some(source_path) = source {
                    // Create a new compiler instance for the imported module
                    let mut module_compiler = Compiler::new(Some(PathBuf::from(source_path)));

                    // Load and compile the module
                    let path = Path::new(source_path);
                    let source_code = fs::read_to_string(path).map_err(|e| {
                        CrabbyError::CompileError(format!("Failed to read module '{}': {}", source_path, e))
                    })?;

                    let tokens = tokenize(&source_code)?;
                    let ast = parse(tokens)?;
                    module_compiler.compile(&ast)?;

                    // Try to import the requested item
                    if let Some(value) = module_compiler.module.public_items.get(name) {
                        self.variables.insert(name.clone(), value.clone());
                        Ok(None)
                    } else if module_compiler.module.private_items.contains_key(name) {
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
            }
            Statement::If { condition, then_branch, else_branch } => {
                let condition_value = self.compile_expression(condition)?;
                match condition_value {
                    Value::Integer(0) => {
                        if let Some(else_branch) = else_branch {
                            self.compile_statement(else_branch)
                        } else {
                            Ok(None)
                        }
                    }
                    _ => self.compile_statement(then_branch),
                }
            }
            Statement::While { condition, body } => {
                loop {
                    let condition_value = self.compile_expression(condition)?;
                    match condition_value {
                        Value::Integer(0) => break,
                        _ => {
                            if let Some(Value::Integer(-1)) = self.compile_statement(body)? {
                                break;
                            }
                        }
                    }
                }
                Ok(None)
            }
            Statement::Block(statements) => {
                let mut last_value = None;
                for stmt in statements {
                    last_value = self.compile_statement(stmt)?;
                }
                Ok(last_value)
            }
            Statement::Expression(expr) => {
                let value = self.compile_expression(expr)?;
                Ok(Some(value))
            }
        }
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<Value, CrabbyError> {
        match expression {
            Expression::Integer(n) => Ok(Value::Integer(*n)),
            Expression::Float(f) => Ok(Value::Float(*f)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Variable(name) => {
                self.variables.get(name).cloned().ok_or_else(|| {
                    CrabbyError::CompileError(format!("Undefined variable: {}", name))
                })
            },
            Expression::Boolean(value) => Ok(Value::Integer(if *value { 1 } else { 0 })),
            Expression::Where { expr, condition } => {
                // For now, evaluate the condition and return the expression if true
                let cond_value = self.compile_expression(condition)?;
                match cond_value {
                    Value::Integer(0) => Ok(Value::Integer(0)),
                    _ => self.compile_expression(expr),
                }
            },
            Expression::Range(count) => {
                let count_value = self.compile_expression(count)?;
                if let Value::Integer(n) = count_value {
                    Ok(Value::Integer(n))
                } else {
                    Err(CrabbyError::CompileError("Range argument must be an integer".to_string()))
                }
            },
            Expression::Call { function, arguments } => {
                if function == "print" {
                    return self.handle_print(arguments);
                }

                if let Some(Value::Lambda(lambda)) = self.variables.get(function) {
                    return self.handle_lambda_call(lambda.clone(), arguments);
                }

                let func = self.functions.get(function).cloned().ok_or_else(|| {
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

                let mut new_compiler = Compiler::new(None);
                for (param, arg) in func.params.iter().zip(arguments) {
                    let arg_value = self.compile_expression(arg)?;
                    new_compiler.variables.insert(param.clone(), arg_value);
                }

                match new_compiler.compile_statement(&func.body)? {
                    Some(value) => Ok(value),
                    None => Ok(Value::Integer(0)),
                }
            },
            Expression::Lambda { params, body } => {
                Ok(Value::Lambda(Function {
                    params: params.clone(),
                    body: body.clone(),
                }))
            },
            Expression::Binary { left, operator, right } => {
                let left_val = self.compile_expression(left)?;
                let right_val = self.compile_expression(right)?;

                match (left_val, operator, right_val) {
                    // Integer operations
                    (Value::Integer(l), BinaryOp::Add, Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    (Value::Integer(l), BinaryOp::Sub, Value::Integer(r)) => Ok(Value::Integer(l - r)),
                    (Value::Integer(l), BinaryOp::Mul, Value::Integer(r)) => Ok(Value::Integer(l * r)),
                    (Value::Integer(l), BinaryOp::Div, Value::Integer(r)) => {
                        if r == 0 {
                            return Err(CrabbyError::CompileError("Division by zero".to_string()));
                        }
                        Ok(Value::Integer(l / r))
                    }

                    // Float operations
                    (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::Float(l), BinaryOp::Sub, Value::Float(r)) => Ok(Value::Float(l - r)),
                    (Value::Float(l), BinaryOp::Mul, Value::Float(r)) => Ok(Value::Float(l * r)),
                    (Value::Float(l), BinaryOp::Div, Value::Float(r)) => {
                        if r == 0.0 {
                            return Err(CrabbyError::CompileError("Division by zero".to_string()));
                        }
                        Ok(Value::Float(l / r))
                    }

                    // Mixed Integer and Float operations
                    (Value::Integer(l), op, Value::Float(r)) => {
                        let l = l as f64;
                        match op {
                            BinaryOp::Add => Ok(Value::Float(l + r)),
                            BinaryOp::Sub => Ok(Value::Float(l - r)),
                            BinaryOp::Mul => Ok(Value::Float(l * r)),
                            BinaryOp::Div => {
                                if r == 0.0 {
                                    return Err(CrabbyError::CompileError("Division by zero".to_string()));
                                }
                                Ok(Value::Float(l / r))
                            }
                            BinaryOp::Eq => Ok(Value::Integer(if (l - r).abs() < f64::EPSILON { 1 } else { 0 })),
                            BinaryOp::Dot => Err(CrabbyError::CompileError("Cannot use dot operator with numbers".to_string())),
                        }
                    }

                    (Value::Float(l), op, Value::Integer(r)) => {
                        let r = r as f64;
                        match op {
                            BinaryOp::Add => Ok(Value::Float(l + r)),
                            BinaryOp::Sub => Ok(Value::Float(l - r)),
                            BinaryOp::Mul => Ok(Value::Float(l * r)),
                            BinaryOp::Div => {
                                if r == 0.0 {
                                    return Err(CrabbyError::CompileError("Division by zero".to_string()));
                                }
                                Ok(Value::Float(l / r))
                            }
                            BinaryOp::Eq => Ok(Value::Integer(if (l - r).abs() < f64::EPSILON { 1 } else { 0 })),
                            BinaryOp::Dot => Err(CrabbyError::CompileError("Cannot use dot operator with numbers".to_string())),
                        }
                    }

                    // String operations
                    (Value::String(l), BinaryOp::Add, Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::String(l), BinaryOp::Dot, Value::String(r)) => Ok(Value::String(format!("{}.{}", l, r))),
                    (Value::String(l), BinaryOp::Add, r) => Ok(Value::String(format!("{}{}", l, r.to_string()))),
                    (l, BinaryOp::Add, Value::String(r)) => Ok(Value::String(format!("{}{}", l.to_string(), r))),

                    _ => Err(CrabbyError::CompileError("Invalid operation".to_string())),
                }
            }
        }
    }
}
