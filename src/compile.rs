use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::future::Future;
use crate::fs;

use crate::utils::CrabbyError;
use crate::core::network::NetworkHandler;
use crate::parser::{parse, Program, Statement, Expression, BinaryOp, PatternKind, MatchArm, NetworkOperation};
use crate::lexer::tokenize;

#[derive(Clone)]
pub struct NetworkState {
    pub handler: NetworkHandler,
    pub connections: HashMap<String, usize>,
}

pub struct Compiler {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    module: Module,
    current_file: Option<PathBuf>,
    network_state: Option<NetworkState>,
    handler: NetworkHandler,
    awaiting: Vec<Pin<Box<dyn Future<Output = Value>>>>,
}

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Lambda(Function),
    Boolean(bool),
    Array(Vec<Value>),
    NetworkHandler(NetworkHandler),
    Void,
}

#[derive(Clone, PartialEq)]
pub struct Function {
    params: Vec<String>,
    body: Box<Statement>,
}

#[derive(Clone)]
pub struct Module {
    pub public_items: HashMap<String, Value>,
    pub private_items: HashMap<String, Value>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            public_items: HashMap::new(),
            private_items: HashMap::new(),
        }
    }
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

impl NetworkState {
    pub fn new() -> Self {
        Self {
            handler: NetworkHandler::new(),
            connections: HashMap::new(),
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

impl Compiler {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let mut compiler = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            module: Module {
                public_items: HashMap::new(),
                private_items: HashMap::new()
            },
            current_file: file_path,
            network_state: None,
            handler: NetworkHandler::new(),
            awaiting: Vec::new(),
        };

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

    pub async fn compile_network_call(&mut self, _object: &Expression, method: &str, args: &[Expression]) -> Result<Value, CrabbyError> {
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.compile_expression(arg).await?);
        }

        let network_state = self.ensure_network()?;

        match method {
            "listen" => {
                if evaluated_args.len() != 2 {
                    return Err(CrabbyError::CompileError("listen() requires address and port".into()));
                }
                match (&evaluated_args[0], &evaluated_args[1]) {
                    (Value::String(addr), Value::Integer(port)) => {
                        network_state.handler.listen(addr, *port as u16).await?;
                        Ok(Value::NetworkHandler(network_state.handler.clone()))
                    }
                    _ => Err(CrabbyError::CompileError("Invalid arguments for listen()".into()))
                }
            },
            "send" => {
                if evaluated_args.len() != 1 {
                    return Err(CrabbyError::CompileError("send() requires data argument".into()));
                }
                match &evaluated_args[0] {
                    Value::String(data) => {
                        network_state.handler.send(data.as_bytes(), 0).await?;
                        Ok(Value::Void)
                    }
                    _ => Err(CrabbyError::CompileError("send() requires string data".into()))
                }
            },
            "receive" => {
                Ok(Value::NetworkHandler(network_state.handler.clone()))
            },
            _ => Err(CrabbyError::CompileError(format!("Unknown network method: {}", method))),
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

    fn ensure_network(&mut self) -> Result<&mut NetworkState, CrabbyError> {
        if self.network_state.is_none() {
            self.network_state = Some(NetworkState {
                handler: NetworkHandler::new(),
                connections: HashMap::new(),
            });
        }
        Ok(self.network_state.as_mut().unwrap())
    }

    pub async fn compile_let_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
        let compiled_value = self.compile_expression(value).await?;
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        if is_public {
            self.module.public_items.insert(var_name, compiled_value);
        } else {
            self.module.private_items.insert(var_name, compiled_value);
        }

        Ok(())
    }

    pub async fn compile_var_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
        let compiled_value = self.compile_expression(value).await;
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        if is_public {
            self.module.public_items.insert(var_name, compiled_value?);
        } else {
            self.module.private_items.insert(var_name, compiled_value?);
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

    async fn handle_lambda_call(&mut self, lambda: Function, arguments: &[Expression]) -> Result<Value, CrabbyError> {
        for (param, arg) in lambda.params.iter().zip(arguments) {
            let arg_value = self.compile_expression(arg).await?;
            self.variables.insert(param.clone(), arg_value);
        }

        if let Some(value) = self.compile_statement(&lambda.body).await? {
            Ok(value)
        } else {
            Ok(Value::Void)
        }
    }

    async fn handle_print(&mut self, args: &[Expression]) -> Result<Value, CrabbyError> {
        if args.len() != 1 {
            return Err(CrabbyError::CompileError("print takes exactly one argument".to_string()));
        }

        let value = self.compile_expression(&args[0]).await?; // Add await here
        println!("{}", value.to_string());
        Ok(Value::Integer(0))
    }

    pub async fn compile(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.compile_statement(statement).await?;
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

    pub async fn load_module(&mut self, current_file: &Path, _name: &str, source: &str) -> Result<(), CrabbyError> {
        let resolved_path = self.resolve_path(current_file, source);
        let source_code = fs::read_to_string(&resolved_path)?;
        let tokens = tokenize(&source_code).await?;
        let ast = parse(tokens).await?;
        let mut module_compiler = Compiler::new(Some(resolved_path));
        module_compiler.compile(&ast).await?;
        Ok(())
    }

    async fn compile_match(&mut self, value: &Expression, arms: &[MatchArm]) -> Result<Option<Value>, CrabbyError> {
        let match_value = self.compile_expression(value).await?;

        for arm in arms {
            if self.pattern_matches(&match_value, &arm.pattern).await? {
                return Ok(Some(self.compile_expression(&arm.body).await?));
            }
        }

        Ok(None)
    }

    async fn pattern_matches(&mut self, value: &Value, pattern: &Expression) -> Result<bool, CrabbyError> {
        match pattern {
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => {
                    let compiled = self.compile_expression(expr).await?;
                    Ok(value == &compiled)
                },
                PatternKind::Variable(_) => Ok(true),
                PatternKind::Wildcard => Ok(true),
            },
            _ => Ok(false),
        }
    }

    pub async fn compile_where(&mut self, expr: &Expression, condition: &Expression, _body: &Statement) -> Result<Value, CrabbyError> {
        let cond_value = self.compile_expression(condition).await?;
        if let Value::Boolean(true) = cond_value {
            self.compile_expression(expr).await
        } else {
            Ok(Value::Void)
        }
    }

    pub async fn compile_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, CrabbyError> {
        match stmt {
            Statement::FunctionDef { name, params, body, return_type: _, docstring: _ } => {
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
            },
            Statement::AsyncFunction { name, params, body, return_type: _ } => {

            },
            Statement::And { left, right } => {
                let left_val = Value::String(left.clone());
                let right_val = Value::String(right.clone());
                Ok(Some(Value::Boolean(left_val == right_val)))
            },
            Statement::Macro { name, params, body } => {
                self.variables.insert(name.clone(), Value::Lambda(Function {
                    params: vec![params.clone()],
                    body: Box::new(Statement::Expression(*(*body).clone())),
                }));
                Ok(None)
            },
            Statement::Network { kind, address, port, body } => {
                let addr = self.compile_expression(address).await;
                let port_val = self.compile_expression(port).await;

                let addr = match addr {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                let port_val = match port_val {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                match (addr, port_val) {
                    (Value::String(addr_str), Value::Integer(port_num)) => {
                        let mut net_state = self.ensure_network()?.clone();
                        match kind {
                            NetworkOperation::Listen { .. } => {
                                net_state.handler.listen(&addr_str, port_num as u16).await?;
                            },
                            NetworkOperation::Connect { .. } => {
                                net_state.handler.connect(&addr_str, port_num as u16).await?;
                            },
                            &NetworkOperation::Send { ref data, ref conn_index } => {
                                let data_val = self.compile_expression(data).await?;
                                if let Value::String(s) = data_val {
                                    net_state.handler.send(s.as_bytes(), *conn_index).await?;
                                }
                            },
                            NetworkOperation::Receive { conn_index } => {
                                let data = self.handler.receive(*conn_index).await?;
                                return if let Ok(s) = String::from_utf8(data) {
                                    Ok(Some(Value::String(s)))
                                } else {
                                    Err(CrabbyError::CompileError("Invalid UTF-8 data".into()))
                                };
                            },
                        }
                        if let Some(body) = body {
                            self.compile_statement(body).await?;
                        }
                        Ok(None)
                    },
                    _ => Err(CrabbyError::CompileError("Invalid network operation arguments".into()))
                }
            },
            Statement::Let { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.compile_expression(value).await?;

                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                self.variables.insert(var_name, compiled_value);
                Ok(None)
            },
            Statement::ArrayAssign { array, index, value } => {
                let array_val = self.compile_expression(array).await?;
                let index_val = self.compile_expression(index).await?;
                let new_val = self.compile_expression(value).await?;

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
            },
            Statement::Var { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let compiled_value = self.compile_expression(value).await?;

                if is_public {
                    self.module.public_items.insert(var_name.clone(), compiled_value.clone());
                } else {
                    self.module.private_items.insert(var_name.clone(), compiled_value.clone());
                }

                self.variables.insert(var_name, compiled_value);
                Ok(None)
            },
            Statement::Match { value, arms } => self.compile_match(value, arms).await,
            Statement::Return(expr) => {
                let value = self.compile_expression(expr).await?;
                Ok(Some(value))
            },
            Statement::Loop { count, body } => {
                let count_value = self.compile_expression(count).await?;
                if let Value::Integer(n) = count_value {
                    for _ in 0..n {
                        self.compile_statement(body).await?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::CompileError("Loop count must be an integer".to_string()))
                }
            },
            Statement::ForIn { variable, iterator, body } => {
                let iter_value = self.compile_expression(iterator).await?;
                if let Value::Integer(n) = iter_value {
                    for i in 0..n {
                        self.variables.insert(variable.clone(), Value::Integer(i));
                        self.compile_statement(body).await?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::CompileError("Iterator must be a range".to_string()))
                }
            },
            Statement::Enum { name, variants: _variants, where_clause } => {
                let value = Value::String(format!("enum {}", name));
                self.variables.insert(name.clone(), value);
                Ok(None)
            },
            Statement::Struct { name, fields: _fields, where_clause: _where_clause } => {
                let value = Value::String(format!("struct {}", name));
                self.variables.insert(name.clone(), value);
                Ok(None)
            },
            Statement::Import { name, source } => {
                if let Some(source_path) = source {
                    let mut module_compiler = Compiler::new(Some(PathBuf::from(source_path)));

                    let path = Path::new(source_path);
                    let source_code = fs::read_to_string(path).map_err(|e| {
                        CrabbyError::CompileError(format!("Failed to read module '{}': {}", source_path, e))
                    })?;

                    let tokens = tokenize(&source_code).await?;
                    let ast = parse(tokens).await?;
                    module_compiler.compile(&ast).await?;

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
            },
            Statement::If { condition, then_branch, else_branch } => {
                let cond_value = self.compile_expression(condition).await?;
                match cond_value {
                    Value::Boolean(true) => self.compile_statement(then_branch).await,
                    Value::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            self.compile_statement(else_branch).await
                        } else {
                            Ok(None)
                        }
                    },
                    _ => Err(CrabbyError::CompileError("Condition must be boolean".into()))
                }
            },
            Statement::While { condition, body } => {
                loop {
                    let condition_value = self.compile_expression(condition).await?;
                    match condition_value {
                        Value::Integer(0) => break,
                        _ => {
                            if let Some(Value::Integer(-1)) = self.compile_statement(body).await? {
                                break;
                            }
                        }
                    }
                }
                Ok(None)
            },
            Statement::Block(statements) => {
                for stmt in statements {
                    self.compile_statement(stmt).await?;
                }
                Ok(None)
            },
            Statement::Expression(expr) => {
                let value = self.compile_expression(expr).await?;
                Ok(Some(value))
            },
            _ => Ok(None)
        }
    }

    pub async fn compile_expression(&mut self, expr: &Expression) -> Result<Value, CrabbyError> {
        match expr {
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
                let cond_value = self.compile_expression(condition).await?;
                match cond_value {
                    Value::Boolean(true) => {
                        self.compile_statement(body).await?;
                        Ok(self.compile_expression(expr).await?)
                    },
                    _ => Ok(Value::Boolean(false)),
                }
            },
            Expression::Range(count) => {
                let count_value = self.compile_expression(count).await?;
                if let Value::Integer(n) = count_value {
                    Ok(Value::Integer(n))
                } else {
                    Err(CrabbyError::CompileError("Range argument must be an integer".to_string()))
                }
            },
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.compile_expression(elem).await?);
                }
                Ok(Value::Array(values))
            },
            Expression::Index { array, index } => {
                let array_value = self.compile_expression(array).await?;
                let index_value = self.compile_expression(index).await?;

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
                    let value = self.compile_expression(expr).await?;
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
            Expression::Await { expr } => {
                let value = self.compile_expression(expr).await?;
                self.awaiting.push(value);
                Ok(Value::Void)
            },
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => {
                    self.compile_expression(expr).await
                },
                PatternKind::Variable(name) => Ok(Value::String(name.clone())),
                PatternKind::Wildcard => Ok(Value::Void),
            },
            Expression::Network { operation, handler } => {
                let network_state = self.ensure_network()?;
                Ok(Value::NetworkHandler(network_state.handler.clone())).unwrap();

                match operation {
                    NetworkOperation::Listen { addr, port } => {
                        let handler = self.ensure_network();
                        handler.unwrap().handler.listen(addr, *port).await?;
                        Ok(Value::NetworkHandler(handler.unwrap().handler.clone()))
                    }
                    NetworkOperation::Connect { addr, port } => {
                        let handler = self.ensure_network();
                        handler.unwrap().handler.connect(addr, *port).await?;
                        Ok(Value::NetworkHandler(handler.unwrap().handler.clone()))
                    }
                    NetworkOperation::Send { data, conn_index } => {
                        let handler = self.ensure_network();
                        let data_val = self.compile_expression(data).await?;
                        if let Value::String(s) = data_val {
                            handler.unwrap().handler.send(s.as_bytes(), *conn_index).await?;
                            Ok(Value::Void)
                        } else {
                            Err(CrabbyError::CompileError("send() requires string data".into()))
                        }
                    }
                    NetworkOperation::Receive { conn_index: _ } => {
                        let handler = self.ensure_network();
                        Ok(Value::NetworkHandler(handler.unwrap().handler.clone()))
                    }
                }
            },
            Expression::Call { function, arguments } => {
                let mut compiled_args = Vec::new();
                for arg in arguments {
                    compiled_args.push(self.compile_expression(arg).await?);
                }

                if function == "print" {
                    return self.handle_print(arguments).await;
                }

                if function == "network" {
                    if let Some(Expression::Call { function: method, arguments: args }) = arguments.get(0) {
                        return self.compile_network_call(&Expression::Variable(function.clone()), method, args).await;
                    }
                }

                if let Some(Value::Lambda(lambda)) = self.variables.get(function) {
                    return self.handle_lambda_call(lambda.clone(), arguments).await;
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
                    let arg_value = self.compile_expression(arg).await?;
                    new_compiler.variables.insert(param.clone(), arg_value);
                }

                match new_compiler.compile_statement(&func.body).await? {
                    Some(value) => Ok(value),
                    None => Ok(Value::Integer(0)),
                }.unwrap();

                Ok(Value::Void)
            },
            Expression::Lambda { params, body } => {
                Ok(Value::Lambda(Function {
                    params: params.clone(),
                    body: body.clone(),
                }))
            },
            Expression::Binary { left, operator, right } => {
                let left_val = self.compile_expression(left).await?;
                let right_val = self.compile_expression(right).await?;

                match (left_val, operator, right_val) {
                    // Integer operations
                    (Value::Integer(l), BinaryOp::Add, Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    (Value::Integer(l), BinaryOp::Sub, Value::Integer(r)) => Ok(Value::Integer(l - r)),
                    (Value::Integer(l), BinaryOp::Mul, Value::Integer(r)) => Ok(Value::Integer(l * r)),
                    (Value::Integer(l), BinaryOp::Div, Value::Integer(r)) => {
                        if r == 0 {
                            return Err(CrabbyError::CompileError("Division by zero".to_string()));
                        }
                        return Ok(Value::Integer(l / r));
                    }

                    // Float operations
                    (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::Float(l), BinaryOp::Sub, Value::Float(r)) => Ok(Value::Float(l - r)),
                    (Value::Float(l), BinaryOp::Mul, Value::Float(r)) => Ok(Value::Float(l * r)),
                    (Value::Float(l), BinaryOp::Div, Value::Float(r)) => {
                        if r == 0.0 {
                            return Err(CrabbyError::CompileError("Division by zero".to_string()));
                        }
                        return Ok(Value::Float(l / r));
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
                                return Ok(Value::Float(l / r));
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
                                return Ok(Value::Float(l / r));
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

                    _ => return Err(CrabbyError::CompileError("Invalid operation".to_string())),
                }?;
                Ok(Value::Void)
            }
            _ => Ok(Value::Void)
        }
    }
}
