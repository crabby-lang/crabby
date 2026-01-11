use crate::fs;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use crate::ast::{BinaryOp, Expression, MatchArm, PatternKind, Program, Statement, Visibility};
use crate::lexer::*;
use crate::modules::Module;
use crate::parser::*;
use crate::utils::CrabbyError;
use crate::value::{Function, Value};

// use crate::core::ffi::{FFIManager, FFIValue};

// Used for limiting the recursion Crabby receives
// to avoid stack overflow at runtime interpretation
const MAX_RECURSION_DEPTH: usize = 1000;

pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

pub struct Interpreter {
    pub env: Environment,
    awaiting: Vec<Pin<Box<dyn Future<Output = Value>>>>,
    function_definitions: HashMap<String, Function>,
    call_stack: Vec<String>,
    pub module: Module,
    current_file: Option<PathBuf>,
    recursion_depth: usize,
    // pub ffi_manager: FFIManager,
    // add_builtin: HashMap<String, Box<dyn Fn(Vec<Value>) -> Result<Value, CrabbyError> + Send + Sync>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    pub fn insert(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
}

impl Interpreter {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let env = Environment::new();

        let mut interpreter = Self {
            env,
            awaiting: Vec::new(),
            call_stack: Vec::new(),
            function_definitions: HashMap::new(),
            module: Module {
                public_items: HashMap::new(),
                private_items: HashMap::new(),
                variable: HashMap::new(),
                exports: HashMap::new(),
            },
            current_file: file_path,
            recursion_depth: 0,
            // ffi_manager: FFIManager::new(),
            // add_builtin: HashMap::new(),
        };

        interpreter.function_definitions.insert(
            "print".into(),
            Function {
                params: vec!["value".into()],
                body: Box::new(Statement::Expression(Expression::Variable("value".into()))),
            },
        );

        interpreter
    }

    fn new_module() -> Module {
        Module {
            public_items: HashMap::new(),
            private_items: HashMap::new(),
            variable: HashMap::new(),
            exports: HashMap::new(),
        }
    }

    // pub fn add_builtin<F>(&mut self, name: &str, func: F)
    // where
    //     F: Fn(Vec<Value>) -> Result<Value, CrabbyError> + Send + Sync + 'static,
    // {
    //     self.add_builtin.insert(name.to_string(), Box::new(func));
    // }

    // pub fn call_ffi_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, CrabbyError> {
    //     let ffi_args = args.into_iter()
    //         .map(|arg| Ok(match arg {
    //             Value::Integer(i) => FFIValue::Int(i as i32),
    //             Value::Float(f) => FFIValue::Float(f),
    //             Value::String(s) => FFIValue::String(std::ffi::CString::new(s)
    //                 .map_err(|e| CrabbyError::InterpreterError(format!("Invalid string for FFI: {}", e)))?),
    //             _ => return Err(CrabbyError::InterpreterError("Unsupported FFI argument type".into())),
    //         }))
    //         .collect::<Result<Vec<_>, _>>()?;

    //     let result = self.ffi_manager.call_function(name, ffi_args)?;

    //     Ok(match result {
    //         FFIValue::Int(i) => Value::Integer(i as i64),
    //         FFIValue::Float(f) => Value::Float(f),
    //         FFIValue::String(s) => Value::String(s.to_string_lossy().into_owned()),
    //        FFIValue::Void => Value::Void,
    //        FFIValue::Pointer(_) => Value::Void, // Handle pointers appropriately
    //     })
    // }

    pub async fn interpret_function_def(
        &mut self,
        name: &str,
        params: &[String],
        body: &Statement,
    ) -> Result<(), CrabbyError> {
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
            self.module
                .public_items
                .insert(func_name, Value::Lambda(function));
        } else {
            self.module
                .private_items
                .insert(func_name, Value::Lambda(function));
        }

        Ok(())
    }

    pub fn interpret_async(&mut self, program: &Program) -> Result<(), CrabbyError> {
        let mut futures = Vec::new();

        for statement in &program.statements {
            match statement {
                Statement::AsyncFunction {
                    name,
                    params,
                    body,
                    return_type,
                } => {
                    let future =
                        self.handle_async_function(name, params, body, return_type.clone())?;
                    futures.push(future);
                }
                _ => {
                    self.interpret_statement(statement)?;
                }
            }
        }

        // Wait for all async operations to complete
        futures::future::join_all(futures);
        Ok(())
    }

    pub fn handle_async_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &Statement,
        _return_type: Option<String>,
    ) -> Result<Pin<Box<dyn Future<Output = Result<Value, CrabbyError>>>>, CrabbyError> {
        let function = Function {
            params: params.to_vec(),
            body: Box::new(body.clone()),
        };

        self.function_definitions.insert(name.to_string(), function);
        Ok(Box::pin(async move { Ok(Value::Void) }))
    }

    pub fn handle_function_call(
        &mut self,
        function: &str,
        arguments: &[Expression],
    ) -> Result<Value, CrabbyError> {
        // Checks recursion depth
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            return Err(CrabbyError::InterpreterError(format!(
                "Maximum recursion depth ({}) exceeded",
                MAX_RECURSION_DEPTH
            )));
        }

        self.recursion_depth += 1;
        let lambda_opt = self.env.get(function);
        let result = match lambda_opt {
            Some(Value::Lambda(lambda)) => self.handle_lambda_call(lambda.clone(), arguments),
            _ => {
                if let Some(func) = self.function_definitions.get(function).cloned() {
                    self.handle_lambda_call(func, arguments)
                } else {
                    Err(CrabbyError::InterpreterError(format!(
                        "Undefined function: {}",
                        function
                    )))
                }
            }
        };
        self.recursion_depth -= 1;
        result
    }

    pub fn handle_lambda_call(
        &mut self,
        lambda: Function,
        arguments: &[Expression],
    ) -> Result<Value, CrabbyError> {
        for (param, arg) in lambda.params.iter().zip(arguments) {
            let arg_value = self.interpret_expression(arg)?;
            self.env.insert(param.clone(), arg_value);
        }

        if let Some(value) = self.interpret_statement(&lambda.body)? {
            Ok(value)
        } else {
            Ok(Value::Void)
        }
    }

    pub fn interpret_let_statement(
        &mut self,
        name: &str,
        value: &Expression,
    ) -> Result<(), CrabbyError> {
        let interpreted_value = self.interpret_expression(value)?;
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        if is_public {
            self.module.public_items.insert(var_name, interpreted_value);
        } else {
            self.module
                .private_items
                .insert(var_name, interpreted_value);
        }

        Ok(())
    }

    pub fn interpret_var_statement(
        &mut self,
        name: &str,
        value: &Expression,
    ) -> Result<(), CrabbyError> {
        let interpreted_value = self.interpret_expression(value)?;
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        if is_public {
            self.module.public_items.insert(var_name, interpreted_value);
        } else {
            self.module
                .private_items
                .insert(var_name, interpreted_value);
        }

        Ok(())
    }

    pub fn handle_print(&mut self, args: &[Expression]) -> Result<Value, CrabbyError> {
        if args.len() != 1 {
            return Err(CrabbyError::InterpreterError(
                "print takes exactly one argument".to_string(),
            ));
        }

        let value = self.interpret_expression(&args[0])?;
        println!("{}", value.to_string());
        Ok(Value::Integer(0))
    }

    pub fn interpret(mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.interpret_statement(statement)?;
        }
        Ok(())
    }

    pub fn interpret_match(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<Option<Value>, CrabbyError> {
        let match_value = self.interpret_expression(value)?;

        for arm in arms {
            if self.pattern_matches(&match_value, &arm.pattern)? {
                return Ok(Some(self.interpret_expression(&arm.body)?));
            }
        }

        Ok(None)
    }

    pub fn pattern_matches(
        &mut self,
        value: &Value,
        pattern: &Expression,
    ) -> Result<bool, CrabbyError> {
        match pattern {
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => {
                    let interpreted = self.interpret_expression(expr)?;
                    Ok(value == &interpreted)
                }
                PatternKind::Variable(_) => Ok(true),
                PatternKind::Wildcard => Ok(true),
            },
            _ => Ok(false),
        }
    }

    pub fn interpret_where(
        mut self,
        expr: &Expression,
        condition: &Expression,
        _body: &Statement,
    ) -> Result<Value, CrabbyError> {
        let cond_value = self.interpret_expression(condition)?;
        if let Value::Boolean(true) = cond_value {
            self.interpret_expression(expr)
        } else {
            Ok(Value::Void)
        }
    }

    pub fn load_and_import_module(
        self,
        _import_name: &str,
        import_path: &str,
    ) -> Result<Module, CrabbyError> {
        let current_file = std::path::Path::new(".");
        let resolved_path = Module::resolve_path(current_file, import_path);
        let source_code = fs::read_to_string(&resolved_path).map_err(|e| {
            CrabbyError::InterpreterError(format!(
                "Failed to read module '{}': {}",
                resolved_path.display(),
                e
            ))
        })?;
        let tokens = TokenStream::tokenize(source_code)?;
        let ast = parse(tokens)?;
        let mut module_interpreter = Interpreter::new(Some(resolved_path.clone()));
        for statement in &ast.statements {
            module_interpreter.interpret_statement(statement)?;
        }
        Ok(module_interpreter.module.clone())
    }

    pub fn interpret_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, CrabbyError> {
        match stmt {
            Statement::FunctionDef {
                name,
                params,
                body,
                return_type: _,
                docstring: _,
                visibility,
            } => {
                // let is_public = name.starts_with("pub ");
                // let func_name = if is_public {
                //     name.trim_start_matches("pub ").to_string()
                // } else {
                //    name.to_string()
                // };

                let function = Function {
                    params: params.clone(),
                    body: body.clone(),
                };

                self.function_definitions
                    .insert(name.to_string(), function.clone());

                match visibility {
                    Visibility::Public => {
                        self.module
                            .public_items
                            .insert(name.to_string(), Value::Lambda(function));
                    }
                    Visibility::Private => {
                        self.module
                            .private_items
                            .insert(name.to_string(), Value::Lambda(function));
                    }
                    _ => {}
                }
                // if is_public {
                //    self.module.public_items.insert(func_name.clone(), Value::Lambda(function.clone()));
                // } else {
                //    self.module.private_items.insert(func_name.clone(), Value::Lambda(function.clone()));
                // }

                // self.function_definitions.insert(func_name, function);

                Ok(None)
            }
            Statement::FunctionFun {
                name,
                params,
                body,
                return_type: _,
                docstring: _,
                visibility,
            } => {
                // let is_public = name.starts_with("pub ");
                // let func_name = if is_public {
                //     name.trim_start_matches("pub ").to_string()
                // } else {
                //    name.to_string()
                // };

                let function = Function {
                    params: params.clone(),
                    body: body.clone(),
                };

                self.function_definitions
                    .insert(name.clone(), function.clone());

                match visibility {
                    Visibility::Public => {
                        self.module
                            .public_items
                            .insert(name.to_string(), Value::Lambda(function));
                    }
                    Visibility::Private => {
                        self.module
                            .private_items
                            .insert(name.to_string(), Value::Lambda(function));
                    }
                    _ => {}
                }
                // if is_public {
                //    self.module.public_items.insert(func_name.clone(), Value::Lambda(function.clone()));
                // } else {
                //    self.module.private_items.insert(func_name.clone(), Value::Lambda(function.clone()));
                // }

                // self.function_definitions.insert(func_name, function);

                Ok(None)
            }
            Statement::Let { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let interpreted_value = self.interpret_expression(value)?;

                if is_public {
                    self.module
                        .public_items
                        .insert(var_name.clone(), interpreted_value.clone());
                } else {
                    self.module
                        .private_items
                        .insert(var_name.clone(), interpreted_value.clone());
                }

                self.env.insert(var_name, interpreted_value);
                Ok(None)
            }
            Statement::Var { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let interpreted_value = self.interpret_expression(value)?;

                if is_public {
                    self.module
                        .public_items
                        .insert(var_name.clone(), interpreted_value.clone());
                } else {
                    self.module
                        .private_items
                        .insert(var_name.clone(), interpreted_value.clone());
                }

                self.env.insert(var_name, interpreted_value);
                Ok(None)
            }
            Statement::Const { name, value } => {
                let is_public = name.starts_with("pub ");
                let var_name = if is_public {
                    name.trim_start_matches("pub ").to_string()
                } else {
                    name.to_string()
                };

                let interpreted_value = self.interpret_expression(value)?;

                if is_public {
                    self.module
                        .public_items
                        .insert(var_name.clone(), interpreted_value.clone());
                } else {
                    self.module
                        .private_items
                        .insert(var_name.clone(), interpreted_value.clone());
                }

                self.env.insert(var_name, interpreted_value);
                Ok(None)
            }
            Statement::AsyncFunction { .. } => Ok(None),
            Statement::And { left, right } => {
                let left_val = Value::String(left.clone());
                let right_val = Value::String(right.clone());
                Ok(Some(Value::Boolean(left_val == right_val)))
            }
            Statement::ArrayAssign {
                array,
                index,
                value,
            } => {
                let array_val = self.interpret_expression(array)?;
                let index_val = self.interpret_expression(index)?;
                let new_val = self.interpret_expression(value)?;

                if let (Value::Array(mut elements), Value::Integer(i)) = (array_val, index_val) {
                    if i < 0 || i >= elements.len() as i64 {
                        return Err(CrabbyError::InterpreterError(
                            "Array index out of bounds".to_string(),
                        ));
                    }
                    elements[i as usize] = new_val;
                    Ok(None)
                } else {
                    Err(CrabbyError::InterpreterError(
                        "Invalid array assignment".to_string(),
                    ))
                }
            }
            Statement::Match { value, arms } => self.interpret_match(&value, &arms),
            Statement::Return(expr) => {
                let value = self.interpret_expression(expr)?;
                Ok(Some(value))
            }
            Statement::Loop { count, body } => {
                let count_value = self.interpret_expression(count)?;
                if let Value::Integer(n) = count_value {
                    for _ in 0..n {
                        self.interpret_statement(body)?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::InterpreterError(
                        "Loop count must be an integer".to_string(),
                    ))
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.interpret_expression(condition)?;
                match cond_value {
                    Value::Boolean(true) => Ok(self.interpret_statement(then_branch)?),
                    Value::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            Ok(self.interpret_statement(else_branch)?)
                        } else {
                            Ok(None)
                        }
                    }
                    _ => {
                        return Err(CrabbyError::InterpreterError(
                            "Condition must be boolean".into(),
                        ));
                    }
                }
            }
            Statement::While { condition, body } => {
                loop {
                    let condition_value = self.interpret_expression(condition)?;
                    match condition_value {
                        Value::Integer(0) => break,
                        _ => {
                            if let Some(Value::Integer(-1)) = self.interpret_statement(body)? {
                                break;
                            }
                        }
                    }
                }
                Ok(None)
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.interpret_statement(stmt)?;
                }
                Ok(None)
            }
            Statement::Expression(expr) => {
                let value = self.interpret_expression(expr)?;
                Ok(Some(value))
            }
            Statement::Import { name: _, source: _ } => {
                // let module = self
                //     .module_loader
                //     .load(self.current_file.as_ref().unwrap(), &source.unwrap())?;

                // let value = module.exports.get(&name).ok_or_else(|| {
                //     CrabbyError::InterpreterError(format!("Item '{}' not found in module!", name))
                // })?;

                // self.env.define(name, value.clone());
                // Ok(None)
                unimplemented!(/* I just want it to compile... */)
            }
            // Statement::Macro { name, params, body } => {
            //    self.env.insert(name.clone(), Value::Lambda(Function {
            //        params: vec![params.clone()],
            //        body: Box::new(Statement::Expression(*(*body).clone())),
            //    }));
            //    Ok(None)
            // },
            Statement::ForIn {
                variable,
                iterator,
                body,
            } => {
                let iter_value = self.interpret_expression(iterator)?;
                if let Value::Integer(n) = iter_value {
                    for i in 0..n {
                        self.env.insert(variable.clone(), Value::Integer(i));
                        self.interpret_statement(body)?;
                    }
                    Ok(None)
                } else {
                    Err(CrabbyError::InterpreterError(
                        "Iterator must be a range".to_string(),
                    ))
                }
            }
            Statement::Enum {
                name,
                variants: _variants,
                where_clause: _,
            } => {
                let value = Value::String(format!("enum {}", name));
                self.env.insert(name.clone(), value);
                Ok(None)
            }
            Statement::Struct {
                name,
                fields: _fields,
                where_clause: _where_clause,
            } => {
                let value = Value::String(format!("struct {}", name));
                self.env.insert(name.clone(), value);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn interpret_expression(&mut self, expr: &Expression) -> Result<Value, CrabbyError> {
        #[allow(unreachable_patterns)]
        match expr {
            Expression::Integer(n) => Ok(Value::Integer(*n)),
            Expression::Float(f) => Ok(Value::Float(*f)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(value) => Ok(Value::Integer(if *value { 1 } else { 0 })),
            Expression::Variable(name) => self.env.get(&name).ok_or_else(|| {
                CrabbyError::InterpreterError(format!("Undefined variable: {}", name))
            }),
            Expression::Await { expr } => self.interpret_expression(expr),
            Expression::Call {
                function,
                arguments,
            } => {
                if self.call_stack.contains(&function) {
                    return Err(CrabbyError::InterpreterError(format!(
                        "Recursion is not allowed: function '{}' calls itself",
                        function
                    )));
                }
                self.call_stack.push(function.clone());

                if function == "print" {
                    self.call_stack.pop();
                    return self.handle_print(&arguments);
                }

                let lambda_opt = self.env.get(&function);
                if let Some(Value::Lambda(lambda)) = lambda_opt {
                    let result = self.handle_lambda_call(lambda.clone(), &arguments);
                    self.call_stack.pop();
                    return result;
                }

                let func = self
                    .function_definitions
                    .get(function)
                    .cloned()
                    .ok_or_else(|| {
                        CrabbyError::InterpreterError(format!("Undefined function: {}", function))
                    })?;

                if arguments.len() != func.params.len() {
                    self.call_stack.pop();
                    return Err(CrabbyError::InterpreterError(format!(
                        "Function {} expects {} arguments, got {}",
                        function,
                        func.params.len(),
                        arguments.len()
                    )));
                }

                let mut new_interpret = Interpreter::new(None);
                for (param, arg) in func.params.iter().zip(arguments) {
                    let arg_value = self.interpret_expression(arg)?;
                    new_interpret.env.insert(param.clone(), arg_value);
                }

                let result = match new_interpret.interpret_statement(&func.body) {
                    Ok(Some(value)) => Ok(value),
                    Ok(None) => Ok(Value::Integer(0)),
                    Err(e) => Err(e),
                    _ => Ok(Value::Void),
                };

                self.call_stack.pop();
                result
            }
            Expression::Where {
                expr,
                condition,
                body,
            } => {
                let cond_value = self.interpret_expression(condition)?;
                match cond_value {
                    Value::Boolean(true) => {
                        self.interpret_statement(body)?;
                        Ok(self.interpret_expression(expr)?)
                    }
                    _ => Ok(Value::Boolean(false)),
                }
            }
            Expression::Range(count) => {
                let count_value = self.interpret_expression(count)?;
                if let Value::Integer(n) = count_value {
                    Ok(Value::Integer(n))
                } else {
                    Err(CrabbyError::InterpreterError(
                        "Range argument must be an integer".to_string(),
                    ))
                }
            }
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.interpret_expression(elem)?);
                }
                Ok(Value::Array(values))
            }
            Expression::Index { array, index } => {
                let array_value = self.interpret_expression(array)?;
                let index_value = self.interpret_expression(index)?;

                match index_value {
                    Value::Integer(i) => array_value.get_index(i),
                    _ => Err(CrabbyError::InterpreterError(
                        "Array index must be an integer".to_string(),
                    )),
                }
            }
            Expression::FString {
                template,
                expressions,
            } => {
                let result = template.clone();
                let mut expr_values = Vec::new();

                // Evaluate all expressions
                for expr in expressions {
                    let value = self.interpret_expression(expr)?;
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
            }
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => self.interpret_expression(&expr),
                PatternKind::Variable(name) => Ok(Value::String(name.clone())),
                PatternKind::Wildcard => Ok(Value::Void),
            },
            Expression::Lambda { params, body } => Ok(Value::Lambda(Function {
                params: params.clone(),
                body: body.clone(),
            })),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_clone = left.clone();
                let right_clone = right.clone();
                let left_val = self.interpret_expression(left)?;
                let right_val = self.interpret_expression(right)?;

                match (left_val, operator, right_val) {
                    // Integer operations
                    (Value::Integer(l), BinaryOp::Add, Value::Integer(r)) => {
                        Ok(Value::Integer(l + r))
                    }
                    (Value::Integer(l), BinaryOp::Sub, Value::Integer(r)) => {
                        Ok(Value::Integer(l - r))
                    }
                    (Value::Integer(l), BinaryOp::Mul, Value::Integer(r)) => {
                        Ok(Value::Integer(l * r))
                    }
                    (Value::Integer(l), BinaryOp::Div, Value::Integer(r)) => {
                        if r == 0 {
                            return Err(CrabbyError::InterpreterError(
                                "Division by zero".to_string(),
                            ));
                        }
                        Ok(Value::Integer(l / r))
                    }

                    // Float operations
                    (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::Float(l), BinaryOp::Sub, Value::Float(r)) => Ok(Value::Float(l - r)),
                    (Value::Float(l), BinaryOp::Mul, Value::Float(r)) => Ok(Value::Float(l * r)),
                    (Value::Float(l), BinaryOp::Div, Value::Float(r)) => {
                        if r == 0.0 {
                            return Err(CrabbyError::InterpreterError(
                                "Division by zero".to_string(),
                            ));
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
                                    return Err(CrabbyError::InterpreterError(
                                        "Division by zero".to_string(),
                                    ));
                                }
                                Ok(Value::Float(l / r))
                            }
                            BinaryOp::Eq => Ok(Value::Integer(if (l - r).abs() < f64::EPSILON {
                                1
                            } else {
                                0
                            })),
                            BinaryOp::MatchOp => {
                                let left_expr = left_clone.as_ref();
                                let right_expr = right_clone.as_ref();
                                Ok(Value::Boolean(left_expr.matches(right_expr)))
                            }
                            BinaryOp::Dot => Err(CrabbyError::InterpreterError(
                                "Cannot use dot operator with numbers".to_string(),
                            )),
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
                                    return Err(CrabbyError::InterpreterError(
                                        "Division by zero".to_string(),
                                    ));
                                }
                                Ok(Value::Float(l / r))
                            }
                            BinaryOp::Eq => Ok(Value::Integer(if (l - r).abs() < f64::EPSILON {
                                1
                            } else {
                                0
                            })),
                            BinaryOp::MatchOp => Err(CrabbyError::InterpreterError(
                                "Cannot use match operator with numbers".to_string(),
                            )),
                            BinaryOp::Dot => Err(CrabbyError::InterpreterError(
                                "Cannot use dot operator with numbers".to_string(),
                            )),
                        }
                    }

                    // String operations
                    (Value::String(l), BinaryOp::Add, Value::String(r)) => {
                        Ok(Value::String(format!("{}{}", l, r)))
                    }
                    (Value::String(l), BinaryOp::Dot, Value::String(r)) => {
                        Ok(Value::String(format!("{}.{}", l, r)))
                    }
                    (Value::String(l), BinaryOp::Add, r) => {
                        Ok(Value::String(format!("{}{}", l, r.to_string())))
                    }
                    (l, BinaryOp::Add, Value::String(r)) => {
                        Ok(Value::String(format!("{}{}", l.to_string(), r)))
                    }

                    _ => Err(CrabbyError::InterpreterError(
                        "Invalid operation".to_string(),
                    )),
                }
            }
            _ => Ok(Value::Void),
        }
    }
}
