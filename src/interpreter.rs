use std::collections::HashMap;
use crate::fs;
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;

use crate::utils::CrabbyError;
use crate::ast::{Program, Statement, Expression, BinaryOp, PatternKind, MatchArm};
use crate::value::{Value, Function};
use crate::parser::*;
use crate::lexer::*;
use crate::modules::Module;

pub struct Interpreter {
    variables: HashMap<String, Value>,
    awaiting: Vec<Pin<Box<dyn Future<Output = Value>>>>,
    function_definitions: HashMap<String, Function>,
    call_stack: Vec<String>,
    module: Module,
    current_file: Option<PathBuf>,
}

impl Interpreter {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let mut interpreter = Self {
            variables: HashMap::new(),
            awaiting: Vec::new(),
            call_stack: Vec::new(),
            function_definitions: HashMap::new(),
            module: Module {
                public_items: HashMap::new(),
                private_items: HashMap::new(),
                variable: HashMap::new()
            },
            current_file: file_path
        };

        interpreter.function_definitions.insert("print".to_string(), Function {
            params: vec!["value".to_string()],
            body: Box::new(Statement::Expression(Expression::Variable("value".to_string()))),
        });

        interpreter
    }

    fn new_module() -> Module {
        Module {
            public_items: HashMap::new(),
            private_items: HashMap::new(),
            variable: HashMap::new()
        }
    }

    pub fn interpret_function_def(&mut self, name: &str, params: &[String], body: &Statement) -> Result<(), CrabbyError> {
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

    pub async fn handle_lambda_call(&mut self, lambda: Function, arguments: &[Expression]) -> Result<Value, CrabbyError> {
        for (param, arg) in lambda.params.iter().zip(arguments) {
            let arg_value = self.interpret_expression(arg).await?;
            self.variables.insert(param.clone(), arg_value);
        }

        if let Some(value) = self.interpret_statement(&lambda.body).await? {
            Ok(value)
        } else {
            Ok(Value::Void)
        }
    }

    pub async fn interpret_let_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
        let interpreted_value = self.interpret_expression(value).await?;
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        if is_public {
            self.module.public_items.insert(var_name, interpreted_value);
        } else {
            self.module.private_items.insert(var_name, interpreted_value);
        }

        Ok(())
    }

    pub async fn interpret_var_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
        let interpreted_value = self.interpret_expression(value).await?;
        let is_public = name.starts_with("pub ");
        let var_name = if is_public {
            name.trim_start_matches("pub ").to_string()
        } else {
            name.to_string()
        };

        if is_public {
            self.module.public_items.insert(var_name, interpreted_value);
        } else {
            self.module.private_items.insert(var_name, interpreted_value);
        }

        Ok(())
    }

    pub async fn handle_print(&mut self, args: &[Expression]) -> Result<Value, CrabbyError> {
        if args.len() != 1 {
            return Err(CrabbyError::InterpreterError("print takes exactly one argument".to_string()));
        }

        let value = self.interpret_expression(&args[0]).await?;
        println!("{}", value.to_string());
        Ok(Value::Integer(0))
    }

    pub async fn interpret(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.interpret_statement(statement).await?;
        }
        Ok(())
    }

    pub async fn interpret_match(&mut self, value: &Expression, arms: &[MatchArm]) -> Result<Option<Value>, CrabbyError> {
        let match_value = self.interpret_expression(value).await?;

        for arm in arms {
            if self.pattern_matches(&match_value, &arm.pattern).await? {
                return Ok(Some(self.interpret_expression(&arm.body).await?));
            }
        }

        Ok(None)
    }

    async fn pattern_matches(&mut self, value: &Value, pattern: &Expression) -> Result<bool, CrabbyError> {
        match pattern {
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => {
                    let interpreted = self.interpret_expression(expr).await?;
                    Ok(value == &interpreted)
                },
                PatternKind::Variable(_) => Ok(true),
                PatternKind::Wildcard => Ok(true),
            },
            _ => Ok(false),
        }
    }

    pub async fn interpret_where(&mut self, expr: &Expression, condition: &Expression, _body: &Statement) -> Result<Value, CrabbyError> {
        let cond_value = self.interpret_expression(condition).await?;
        if let Value::Boolean(true) = cond_value {
            self.interpret_expression(expr).await
        } else {
            Ok(Value::Void)
        }
    }

    pub async fn load_and_import_module(&self, _import_name: &str, import_path: &str) -> Result<Module, CrabbyError> {
        let current_file = std::path::Path::new(".");
        let resolved_path = Module::resolve_path(current_file, import_path);
        let source_code = fs::read_to_string(&resolved_path).map_err(|e| {
            CrabbyError::InterpreterError(format!("Failed to read module '{}': {}", resolved_path.display(), e))
        })?;
        let tokens = tokenize(&source_code).await?;
        let ast = parse(tokens).await?;
        let mut module_interpreter = Interpreter::new(Some(resolved_path.clone()));
        module_interpreter.interpret(&ast).await?;
        Ok(module_interpreter.module)
    }

    pub fn interpret_statement<'a>(&'a mut self, stmt: &'a Statement) -> Pin<Box<dyn Future<Output = Result<Option<Value>, CrabbyError>> + 'a>> {
        Box::pin(async move {
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

                    self.function_definitions.insert(func_name, function);

                    Ok(None)
                },
                Statement::Let { name, value } => {
                    let is_public = name.starts_with("pub ");
                    let var_name = if is_public {
                        name.trim_start_matches("pub ").to_string()
                    } else {
                        name.to_string()
                    };

                    let interpreted_value = self.interpret_expression(value).await?;

                    if is_public {
                        self.module.public_items.insert(var_name.clone(), interpreted_value.clone());
                    } else {
                        self.module.private_items.insert(var_name.clone(), interpreted_value.clone());
                    }

                    self.variables.insert(var_name, interpreted_value);
                    Ok(None)
                },
                Statement::Var { name, value } => {
                    let is_public = name.starts_with("pub ");
                    let var_name = if is_public {
                        name.trim_start_matches("pub ").to_string()
                    } else {
                        name.to_string()
                    };

                    let interpreted_value = self.interpret_expression(value).await?;

                    if is_public {
                        self.module.public_items.insert(var_name.clone(), interpreted_value.clone());
                    } else {
                        self.module.private_items.insert(var_name.clone(), interpreted_value.clone());
                    }

                    self.variables.insert(var_name, interpreted_value);
                    Ok(None)
                },
                Statement::Const { name, value } => {
                    let is_public = name.starts_with("pub ");
                    let var_name = if is_public {
                        name.trim_start_matches("pub ").to_string()
                    } else {
                        name.to_string()
                    };

                    let interpreted_value = self.interpret_expression(value).await?;

                    if is_public {
                        self.module.public_items.insert(var_name.clone(), interpreted_value.clone());
                    } else {
                        self.module.private_items.insert(var_name.clone(), interpreted_value.clone());
                    }

                    self.variables.insert(var_name, interpreted_value);
                    Ok(None)
                },
                Statement::AsyncFunction { .. } => Ok(None),
                Statement::And { left, right } => {
                    let left_val = Value::String(left.clone());
                    let right_val = Value::String(right.clone());
                    Ok(Some(Value::Boolean(left_val == right_val)))
                },
                Statement::ArrayAssign { array, index, value } => {
                    let array_val = self.interpret_expression(array).await?;
                    let index_val = self.interpret_expression(index).await?;
                    let new_val = self.interpret_expression(value).await?;

                    if let (Value::Array(mut elements), Value::Integer(i)) = (array_val, index_val) {
                        if i < 0 || i >= elements.len() as i64 {
                            return Err(CrabbyError::InterpreterError(
                                "Array index out of bounds".to_string()
                            ));
                        }
                        elements[i as usize] = new_val;
                        Ok(None)
                    } else {
                        Err(CrabbyError::InterpreterError(
                            "Invalid array assignment".to_string()
                        ))
                    }
                },
                Statement::Match { value, arms } => self.interpret_match(value, arms).await,
                Statement::Return(expr) => {
                    let value = self.interpret_expression(expr).await?;
                    Ok(Some(value))
                },
                Statement::Loop { count, body } => {
                    let count_value = self.interpret_expression(count).await?;
                    if let Value::Integer(n) = count_value {
                        for _ in 0..n {
                            self.interpret_statement(body).await?;
                        }
                        Ok(None)
                    } else {
                        Err(CrabbyError::InterpreterError("Loop count must be an integer".to_string()))
                    }
                },
                Statement::If { condition, then_branch, else_branch } => {
                    let cond_value = self.interpret_expression(condition).await?;
                    match cond_value {
                        Value::Boolean(true) => Ok(self.interpret_statement(then_branch).await?),
                        Value::Boolean(false) => {
                            if let Some(else_branch) = else_branch {
                                Ok(self.interpret_statement(else_branch).await?)
                            } else {
                                Ok(None)
                            }
                        },
                        _ => return Err(CrabbyError::InterpreterError("Condition must be boolean".into()))
                    }
                },
                Statement::While { condition, body } => {
                    loop {
                        let condition_value = self.interpret_expression(condition).await?;
                        match condition_value {
                            Value::Integer(0) => break,
                            _ => {
                                if let Some(Value::Integer(-1)) = self.interpret_statement(body).await? {
                                    break;
                                }
                            }
                        }
                    }
                }
                Statement::Block(statements) => {
                    for stmt in statements {
                        self.interpret_statement(stmt).await?;
                    }
                    Ok(None)
                },
                Statement::Expression(expr) => {
                    let value = self.interpret_expression(expr).await?;
                    Ok(Some(value))
                },
                Statement::Import { name, source } => {
                    if let Some(source_path) = source {
                        let module = self.load_and_import_module(name, source_path).await?;
                        if let Some(value) = module.public_items.get(name) {
                            self.module.variable.insert(name.clone(), value.clone());
                            Ok(None)
                        } else if module.private_items.contains_key(name) {
                            Err(CrabbyError::InterpreterError(format!(
                                "Cannot import private item '{}' from module",
                                name
                            )))
                        } else {
                            Err(CrabbyError::InterpreterError(format!(
                                "Item '{}' not found in module",
                                name
                            )))
                        }
                    } else {
                        Err(CrabbyError::InterpreterError("Standard library imports not yet implemented".to_string()))
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
                    let iter_value = self.interpret_expression(iterator).await?;
                    if let Value::Integer(n) = iter_value {
                        for i in 0..n {
                            self.variables.insert(variable.clone(), Value::Integer(i));
                            self.interpret_statement(body).await?;
                        }
                        Ok(None)
                    } else {
                        Err(CrabbyError::InterpreterError("Iterator must be a range".to_string()))
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
        })
    }

    pub fn interpret_expression<'a>(&'a mut self, expr: &'a Expression) -> Pin<Box<dyn Future<Output = Result<Value, CrabbyError>> + 'a>> {
        Box::pin(async move {
            match expr {
                Expression::Integer(n) => Ok(Value::Integer(*n)),
                Expression::Float(f) => Ok(Value::Float(*f)),
                Expression::String(s) => Ok(Value::String(s.clone())),
                Expression::Boolean(value) => Ok(Value::Integer(if *value { 1 } else { 0 })),
                Expression::Variable(name) => {
                    self.variables.get(name).cloned().ok_or_else(|| {
                        CrabbyError::InterpreterError(format!("Undefined variable: {}", name))
                    })
                },
                Expression::Await { expr } => {
                    let value = self.interpret_expression(expr).await?;
                    Ok(value)
                },
                Expression::Call { function, arguments } => {
                    let mut interpreted_args = Vec::new();

                    if self.call_stack.contains(function) {
                        return Err(CrabbyError::InterpreterError(format!(
                            "Recursion is not allowed: function '{}' calls itself", function
                        )));
                    }
                    self.call_stack.push(function.clone());

                    for arg in arguments {
                        interpreted_args.push(self.interpret_expression(arg).await?);
                    }

                    if function == "print" {
                        return self.handle_print(arguments).await;
                    }

                    if let Some(Value::Lambda(lambda)) = self.variables.get(function) {
                        return self.handle_lambda_call(lambda.clone(), arguments).await;
                    }

                    let func = self.function_definitions.get(function).cloned().ok_or_else(|| {
                        CrabbyError::InterpreterError(format!("Undefined function: {}", function))
                    })?;

                    if arguments.len() != func.params.len() {
                        return Err(CrabbyError::InterpreterError(format!(
                            "Function {} expects {} arguments, got {}",
                            function,
                            func.params.len(),
                            arguments.len()
                        )));
                    }

                    let result = if let Some(Value::Lambda(lambda)) = self.variables.get(function) {
                        self.handle_lambda_call(lambda.clone(), arguments).await
                    } else {
                        let func = self.function_definitions.get(function).cloned().ok_or_else(|| {
                            CrabbyError::InterpreterError(format!("Undefined function: {}", function))
                        })?;
                        if arguments.len() != func.params.len() {
                            return Err(CrabbyError::InterpreterError(format!(
                                "Function {} expects {} arguments, got {}",
                                function,
                                func.params.len(),
                                arguments.len()
                            )));
                        }
                        let mut new_interpret = Interpreter::new(None);
                        for (param, arg) in func.params.iter().zip(arguments) {
                            let arg_value = self.interpret_expression(arg).await?;
                            new_interpret.variables.insert(param.clone(), arg_value);
                        }
                        match new_interpret.interpret_statement(&func.body).await? {
                            Some(value) => Ok(value),
                            None => Ok(Value::Integer(0)),
                        }
                    };
                    self.call_stack.pop();
                    result;

                    Ok(Value::Void)
                },
                Expression::Where { expr, condition, body } => {
                    let cond_value = self.interpret_expression(condition).await?;
                    match cond_value {
                        Value::Boolean(true) => {
                            self.interpret_statement(body);
                            Ok(self.interpret_expression(expr).await?)
                        },
                        _ => Ok(Value::Boolean(false)),
                    }
                },
                Expression::Range(count) => {
                    let count_value = self.interpret_expression(count).await?;
                    if let Value::Integer(n) = count_value {
                        Ok(Value::Integer(n))
                    } else {
                        Err(CrabbyError::InterpreterError("Range argument must be an integer".to_string()))
                    }
                },
                Expression::Array(elements) => {
                    let mut values = Vec::new();
                    for elem in elements {
                        values.push(self.interpret_expression(elem).await?);
                    }
                    Ok(Value::Array(values))
                },
                Expression::Index { array, index } => {
                    let array_value = self.interpret_expression(array).await?;
                    let index_value = self.interpret_expression(index).await?;

                    match index_value {
                        Value::Integer(i) => array_value.get_index(i),
                        _ => Err(CrabbyError::InterpreterError(
                            "Array index must be an integer".to_string()
                        )),
                    }
                },
                Expression::FString { template, expressions } => {
                    let result = template.clone();
                    let mut expr_values = Vec::new();

                    // Evaluate all expressions
                    for expr in expressions {
                        let value = self.interpret_expression(expr).await?;
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
                    PatternKind::Literal(expr) => {
                        self.interpret_expression(expr).await
                    },
                    PatternKind::Variable(name) => Ok(Value::String(name.clone())),
                    PatternKind::Wildcard => Ok(Value::Void),
                },
                Expression::Lambda { params, body } => {
                    Ok(Value::Lambda(Function {
                        params: params.clone(),
                        body: body.clone(),
                    }))
                },
                Expression::Binary { left, operator, right } => {
                    let left_val = self.interpret_expression(left).await?;
                    let right_val = self.interpret_expression(right).await?;

                    match (left_val, operator, right_val) {
                        // Integer operations
                        (Value::Integer(l), BinaryOp::Add, Value::Integer(r)) => Ok(Value::Integer(l + r)),
                        (Value::Integer(l), BinaryOp::Sub, Value::Integer(r)) => Ok(Value::Integer(l - r)),
                        (Value::Integer(l), BinaryOp::Mul, Value::Integer(r)) => Ok(Value::Integer(l * r)),
                        (Value::Integer(l), BinaryOp::Div, Value::Integer(r)) => {
                            if r == 0 {
                                return Err(CrabbyError::InterpreterError("Division by zero".to_string()));
                            }
                            return Ok(Value::Integer(l / r));
                        }

                        // Float operations
                        (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Ok(Value::Float(l + r)),
                        (Value::Float(l), BinaryOp::Sub, Value::Float(r)) => Ok(Value::Float(l - r)),
                        (Value::Float(l), BinaryOp::Mul, Value::Float(r)) => Ok(Value::Float(l * r)),
                        (Value::Float(l), BinaryOp::Div, Value::Float(r)) => {
                            if r == 0.0 {
                                return Err(CrabbyError::InterpreterError("Division by zero".to_string()));
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
                                        return Err(CrabbyError::InterpreterError("Division by zero".to_string()));
                                    }
                                    return Ok(Value::Float(l / r));
                                }
                                BinaryOp::Eq => Ok(Value::Integer(if (l - r).abs() < f64::EPSILON { 1 } else { 0 })),
                                BinaryOp::MatchOp => Ok(Value::Boolean((*left).matches(&*right))),
                                BinaryOp::Dot => Err(CrabbyError::InterpreterError("Cannot use dot operator with numbers".to_string())),
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
                                        return Err(CrabbyError::InterpreterError("Division by zero".to_string()));
                                    }
                                    return Ok(Value::Float(l / r));
                                }
                                BinaryOp::Eq => Ok(Value::Integer(if (l - r).abs() < f64::EPSILON { 1 } else { 0 })),
                                BinaryOp::MatchOp => Err(CrabbyError::InterpreterError("Cannot use match operator with numbers".to_string())),
                                BinaryOp::Dot => Err(CrabbyError::InterpreterError("Cannot use dot operator with numbers".to_string())),
                            }
                        }

                        // String operations
                        (Value::String(l), BinaryOp::Add, Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                        (Value::String(l), BinaryOp::Dot, Value::String(r)) => Ok(Value::String(format!("{}.{}", l, r))),
                        (Value::String(l), BinaryOp::Add, r) => Ok(Value::String(format!("{}{}", l, r.to_string()))),
                        (l, BinaryOp::Add, Value::String(r)) => Ok(Value::String(format!("{}{}", l.to_string(), r))),

                        _ => return Err(CrabbyError::InterpreterError("Invalid operation".to_string())),
                    }?;
                    Ok(Value::Void)
                },
                _ => Ok(Value::Void)
            }
        })
    }
}
