use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::fs;

use crate::utils::CrabbyError;
use crate::core::memory::MemoryChecker;
use crate::core::network::NetworkHandler;
use crate::core::network::NetworkEvent;
use crate::parser::{Parser, parse, Program, Statement, Expression, BinaryOp, PatternKind, MatchArm, NetworkOperation};
use crate::lexer::tokenize;
use crate::docgen::Documentation;

#[derive(Clone)]
struct NetworkState {
    handler: NetworkHandler,
    connections: HashMap<String, usize>,
}

pub struct Compiler {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    module: Module,
    current_file: Option<PathBuf>,
    network_state: Option<NetworkState>,
}

#[derive(Clone)]
enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Lambda(Function),
    Boolean(bool),
    Array(Vec<Value>),
    NetworkHandler(NetworkHandler),
    Void,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Lambda(_), Value::Lambda(_)) => false,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::NetworkHandler(_), Value::NetworkHandler(_)) => false,
            (Value::Void, Value::Void) => true,
            _ => false,
        }
    }
}

impl Value {
    fn to_string(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Lambda(_) => "<lambda>".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(elements) => {
                let elements_str: Vec<String> = elements.iter()
                    .map(|e| e.to_string())
                    .collect();
                format!("[{}]", elements_str.join(", "))
            },
            Value::NetworkHandler(_) => "<network handler>".to_string(),
            Value::Void => "void".to_string()
        }
    }

    fn matches(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        }
    }

    fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        }
    }

    fn get_index(&self, index: i64) -> Result<Value, CrabbyError> {
        match self {
            Value::Array(elements) => {
                if index < 0 || index >= elements.len() as i64 {
                    Err(CrabbyError::CompileError(format!(
                        "Array index out of bounds: {}", index
                    )))
                } else {
                    Ok(elements[index as usize].clone())
                }
            }
            _ => Err(CrabbyError::CompileError(
                "Cannot index non-array value".to_string()
            )),
        }
    }
}

#[derive(Clone, PartialEq)]
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
            network_state: None,
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

    fn compile_network_call(&mut self, _object: &Expression, method: &str, args: &[Expression]) -> Result<Value, CrabbyError> {
        let evaluated_args: Vec<Value> = args.iter()
            .map(|arg| self.compile_expression(arg))
            .collect::<Result<Vec<_>, _>>()?;

        if self.network_state.is_none() {
            self.network_state = Some(NetworkState {
                handler: NetworkHandler::new(),
                connections: HashMap::new(),
            });
        }

        let network_state = self.network_state.as_mut().unwrap();

        match method {
            "listen" => {
                if evaluated_args.len() != 2 {
                    return Err(CrabbyError::CompileError("listen() requires address and port".into()));
                }
                match (&evaluated_args[0], &evaluated_args[1]) {
                    (Value::String(addr), Value::Integer(port)) => {
                        network_state.handler.listen(addr, *port as u16)?;
                        return Ok(Value::NetworkHandler(network_state.handler.clone()))
                    }
                    _ => Err(CrabbyError::CompileError("listen() requires address and port".into()))
                };
            }
            "connect" => {
                if evaluated_args.len() != 2 {
                    return Err(CrabbyError::CompileError("connect() requires address and port".into()));
                }
                match (&evaluated_args[0], &evaluated_args[1]) {
                    (Value::String(addr), Value::Integer(port)) => {
                        network_state.handler.connect(addr, *port as u16)?;
                        return Ok(Value::NetworkHandler(network_state.handler.clone()));
                    }
                    _ => Err(CrabbyError::CompileError("Invalid arguments for connect()".into()))
                };
            },
            "send" => {
                if evaluated_args.len() != 1 {
                    return Err(CrabbyError::CompileError("send() requires one argument".into()));
                }
                match &evaluated_args[0] {
                    Value::String(data) => {
                        network_state.handler.send(data.as_bytes(), 0)?;
                        return Ok(Value::Void)
                    }
                    _ => Err(CrabbyError::CompileError("send() requires string data".into()))
                }
            },
            "receive" => {
                return Ok(Value::NetworkHandler(network_state.handler.clone()))
            },
            _ => Err(CrabbyError::CompileError(format!("Unknown network method: {}", method))),
        }

        Ok(Value::Integer(0))
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

    fn compile_var_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
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

    fn compile_match(&mut self, value: &Expression, arms: &[MatchArm]) -> Result<Option<Value>, CrabbyError> {
        let match_value = self.compile_expression(value)?;

        for arm in arms {
            if self.pattern_matches(&match_value, &arm.pattern)? {
                return Ok(Some(self.compile_expression(&arm.body)?));
            }
        }

        Ok(None)
    }

    fn pattern_matches(&mut self, value: &Value, pattern: &Expression) -> Result<bool, CrabbyError> {
        match pattern {
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => Ok(value == &self.compile_expression(expr)?),
                PatternKind::Variable(_) => Ok(true),
                PatternKind::Wildcard => Ok(true),
            },
            _ => Ok(false),
        }
    }

    fn compile_where(&mut self, expr: &Expression, condition: &Expression, body: &Statement) -> Result<Value, CrabbyError> {
        let cond_value = self.compile_expression(condition)?;
        if let Value::Boolean(true) = cond_value {
            self.compile_expression(expr)
        } else {
            Ok(Value::Boolean(false))
        }
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<Option<Value>, CrabbyError> {
        match statement {
            Statement::FunctionDef { name, params, body, return_type, docstring } => {
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
            Statement::Async { condition, body } => {
                let _condition = self.compile_expression(condition)?;
                self.compile_statement(body)
            }
            Statement::Await { condition, body } => {
                let _condition = self.compile_expression(condition)?;
                self.compile_statement(body)
            }
            Statement::And { left, right } => {
                let left_val = Value::String(left.clone());
                let right_val = Value::String(right.clone());
                Ok(Some(Value::Boolean(left_val == right_val)))
            }
            Statement::Macro { name, params, body } => {
                self.variables.insert(name.clone(), Value::Lambda(Function {
                    params: vec![params.clone()],
                    body: Box::new(Statement::Expression(*(*body).clone())),
                }));
                Ok(None)
            }
            Statement::Network { kind, address, port, body } => {
                let addr = self.compile_expression(address)?;
                let port_val = self.compile_expression(port)?;

                let mut handler = NetworkHandler::new();

                match kind {
                    NetworkOperation::Listen { addr: _, port: _ } => {
                        if let (Value::String(addr_str), Value::Integer(port_num)) = (addr, port_val) {
                            handler.listen(&addr_str)?;
                            if let Some(body) = body {
                                self.compile_statement(body)?;
                            }
                        } else {
                            return Err(CrabbyError::CompileError(
                                "Listen expects string address and integer port".to_string()
                            ));
                        }
                    },
                    NetworkOperation::Connect { addr: _, port: _ } => {
                        if let (Value::String(addr_str), Value::Integer(port_num)) = (addr, port_val) {
                            handler.connect(&addr_str, port_num as u16)?;
                            if let Some(body) = body {
                                self.compile_statement(body)?;
                            }
                        } else {
                            return Err(CrabbyError::CompileError(
                                "Connect expects string address and integer port".to_string()
                            ));
                        }
                    },
                    NetworkOperation::Send { data: _, conn_index: _ } => {
                        let data_val = self.compile_expression(data)?;
                        if let Value::String(data_str) = data_val {
                            handler.send(data_str.as_bytes(), 0)?;
                        } else {
                            return Err(CrabbyError::CompileError(
                                "Send expects string data".to_string()
                            ));
                        }
                    },
                    NetworkOperation::Receive => {
                        match handler.receive()? {
                            NetworkEvent::Received(data) => {
                                if let Ok(s) = String::from_utf8(data) {
                                    return Ok(Some(Value::String(s)));
                                }
                            },
                            _ => return Ok(None),
                        }
                    },
                    NetworkOperation::Bind { addr: _, port: _ } => {
                        if let (Value::String(addr_str), Value::Integer(port_num)) = (addr, port_val) {
                            handler.bind(&addr_str, port_num as u16)?;
                            if let Some(body) = body {
                                self.compile_statement(body)?;
                            }
                        } else {
                            return Err(CrabbyError::CompileError(
                                "Bind expects string address and integer port".to_string()
                            ));
                        }
                    },
                }

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
            Statement::ArrayAssign { array, index, value } => {
                let array_val = self.compile_expression(array)?;
                let index_val = self.compile_expression(index)?;
                let new_val = self.compile_expression(value)?;

                if let (Value::Array(mut elements), Value::Integer(i)) = (array_val, index_val) {
                    if i < 0 || i >= elements.len() as i64 {
                        return Err(CrabbyError::CompileError(
                            "Array index out of bounds".to_string()
                        ));
                    }
                    elements[i as usize] = new_val;
                    Ok(None)
                } else {
                    Err(CrabbyError::CompileError(
                        "Invalid array assignment".to_string()
                    ))
                }
            }
            Statement::Var { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.compile_expression(value)?;

                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                self.variables.insert(var_name, compiled_value);

                Ok(None)
            }
            Statement::Match { value, arms } => self.compile_match(value, arms),
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
            Expression::Where { expr, condition, body } => {
                let cond_value = self.compile_expression(condition)?;
                match cond_value {
                    Value::Boolean(true) => {
                        self.compile_statement(body)?;
                        self.compile_expression(expr)
                    },
                    _ => Ok(Value::Boolean(false)),
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
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.compile_expression(elem)?);
                }
                Ok(Value::Array(values))
            },
            Expression::Index { array, index } => {
                let array_value = self.compile_expression(array)?;
                let index_value = self.compile_expression(index)?;

                match index_value {
                    Value::Integer(i) => array_value.get_index(i),
                    _ => Err(CrabbyError::CompileError(
                        "Array index must be an integer".to_string()
                    )),
                }
            },
            Expression::FString { template, expressions } => {
                let result = template.clone();
                let mut expr_values = Vec::new();

                // Evaluate all expressions
                for expr in expressions {
                    let value = self.compile_expression(expr)?;
                    expr_values.push(value);
                }

                // Replace placeholders with values
                let mut curr_pos = 0;
                let mut final_string = String::new();

                for (_i, value) in expr_values.iter().enumerate() {
                    if let Some(start) = result[curr_pos..].find('{') {
                        if let Some(end) = result[curr_pos + start..].find('}') {
                            final_string.push_str(&result[curr_pos..curr_pos + start]);
                            final_string.push_str(&value.to_string());
                            curr_pos = curr_pos + start + end + 1;
                        }
                    }
                }

                final_string.push_str(&result[curr_pos..]);
                Ok(Value::String(final_string))
            },
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => self.compile_expression(expr),
                PatternKind::Variable(name) => Ok(self.variables.get(name)
                    .ok_or_else(|| CrabbyError::CompileError(format!("Undefined variable: {}", name)))?
                    .clone()),
                PatternKind::Wildcard => Ok(Value::Boolean(true)),
            },
            Expression::Network { operation, handler } => {
                match operation {
                    NetworkOperation::Listen { addr, port } => {
                        let handler = self.ensure_network()?;
                        handler.handler.listen(addr, *port)?;
                        Ok(Value::NetworkHandler(handler.handler.clone()))
                    }
                    NetworkOperation::Connect { addr, port } => {
                        let handler = self.ensure_network()?;
                        handler.handler.connect(addr, *port)?;
                        Ok(Value::NetworkHandler(handler.handler.clone()))
                    }
                    NetworkOperation::Send { data, conn_index } => {
                        let handler = self.ensure_network()?;
                        let data_val = self.compile_expression(data)?;
                        if let Value::String(s) = data_val {
                            handler.handler.send(s.as_bytes(), *conn_index)?;
                            Ok(Value::Void)
                        } else {
                            Err(CrabbyError::CompileError("send() requires string data".into()))
                        }
                    }
                    NetworkOperation::Receive => {
                        let handler = self.ensure_network()?;
                        Ok(Value::NetworkHandler(handler.handler.clone()))
                    }
                    NetworkOperation::Bind { addr, port } => {
                        let handler = self.ensure_network()?;
                        handler.handler.bind(addr, *port)?;
                        Ok(Value::NetworkHandler(handler.handler.clone()))
                    }
                }
            },
            Expression::Call { function, arguments } => {
                if function == "print" {
                    return self.handle_print(arguments);
                }

                if function == "network" {
                    if let Some(Expression::Call { function: method, arguments: args }) = arguments.get(0) {
                        return self.compile_network_call(&Expression::Variable(function.clone()), method, args);
                    }
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
                            BinaryOp::MatchOp => Ok(Value::Boolean((*left).matches(&*right))),
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
                            BinaryOp::MatchOp => Err(CrabbyError::CompileError("Cannot use match operator with numbers".to_string())),
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

    fn ensure_network(&mut self) -> Result<&mut NetworkState, CrabbyError> {
        if self.network_state.is_none() {
            self.network_state = Some(NetworkState {
                handler: NetworkHandler::new(),
                connections: HashMap::new(),
            });
        }
        Ok(self.network_state.as_mut().unwrap())
    }
}
