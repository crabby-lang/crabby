use std::collections::HashMap;
use std::path::PathBuf;

use crate::utils::CrabbyError;
use crate::parser::{Program, Statement, Expression, BinaryOp, PatternKind, MatchArm};
use crate::value::{Value, Function};
use crate::modules::Module;

pub struct Compiler {
    function_definitions: HashMap<String, Function>,
    module: Module,
}

impl Compiler {
    pub fn new(_file_path: Option<PathBuf>) -> Self {
        let mut compiler = Self {
            function_definitions: HashMap::new(),
            module: Module {
                public_items: HashMap::new(),
                private_items: HashMap::new(),
                variable: HashMap::new()
            }
        };

        compiler.function_definitions.insert("print".to_string(), Function {
            params: vec!["value".to_string()],
            body: Box::new(Statement::Expression(Expression::Variable("value".to_string()))),
        });

        compiler
    }

    fn new_module() -> Module {
        Module {
            public_items: HashMap::new(),
            private_items: HashMap::new(),
            variable: HashMap::new()
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

    pub async fn compile_let_statement(&mut self, name: &str, value: &Expression) -> Result<(), CrabbyError> {
        let compiled_value = self.compile_expression(value)?;
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
        let compiled_value = self.compile_expression(value);
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

    async fn handle_print(&mut self, args: &[Expression]) -> Result<Value, CrabbyError> {
        if args.len() != 1 {
            return Err(CrabbyError::CompileError("print takes exactly one argument".to_string()));
        }

        let value = self.compile_expression(&args[0])?;
        println!("{}", value.to_string());
        Ok(Value::Integer(0))
    }

    pub async fn compile(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    pub fn compile_match(&mut self, value: &Expression, arms: &[MatchArm]) -> Result<Option<Value>, CrabbyError> {
        let match_value = self.compile_expression(value)?;

        for arm in arms {
            if self.pattern_matches(&match_value, &arm.pattern)? {
                return Ok(Some(self.compile_expression(&arm.body)?));
            }
        }

        Ok(None)
    }

    async fn pattern_matches(&mut self, value: &Value, pattern: &Expression) -> Result<bool, CrabbyError> {
        match pattern {
            Expression::Pattern(pattern_kind) => match &**pattern_kind {
                PatternKind::Literal(expr) => {
                    let compiled = self.compile_expression(expr)?;
                    Ok(value == &compiled)
                },
                PatternKind::Variable(_) => Ok(true),
                PatternKind::Wildcard => Ok(true),
            },
            _ => Ok(false),
        }
    }

    pub async fn compile_where(&mut self, expr: &Expression, condition: &Expression, _body: &Statement) -> Result<Value, CrabbyError> {
        let cond_value = self.compile_expression(condition)?;
        if let Value::Boolean(true) = cond_value {
            self.compile_expression(expr)
        } else {
            Ok(Value::Void)
        }
    }

    pub async fn load_and_import_module(&self, _import_name: &str, import_path: &str) -> Result<Module, CrabbyError> {
        let current_file = std::path::Path::new(".");
        let resolved_path = Module::resolve_path(current_file, import_path);
        let source_code = crate::fs::read_to_string(&resolved_path).map_err(|e| {
            CrabbyError::CompileError(format!("Failed to read module '{}': {}", resolved_path.display(), e))
        })?;
        let tokens = crate::lexer::tokenize(&source_code).await?;
        let ast = crate::parser::parse(tokens).await?;
        let mut module_compiler = Compiler::new(Some(resolved_path.clone()));
        module_compiler.compile(&ast).await?;
        Ok(module_compiler.module)
    }

    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, CrabbyError> {
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
            Statement::AsyncFunction { .. } => Ok(None),
            Statement::And { left, right } => {
                let left_val = Value::String(left.clone());
                let right_val = Value::String(right.clone());
                Ok(Some(Value::Boolean(left_val == right_val)))
            },
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
            },
            Statement::Match { value, arms } => self.compile_match(value, arms),
            Statement::Return(expr) => {
                let value = self.compile_expression(expr)?;
                Ok(Some(value))
            },
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
            },
            Statement::If { condition, then_branch, else_branch } => {
                let cond_value = self.compile_expression(condition)?;
                match cond_value {
                    Value::Boolean(true) => self.compile_statement(then_branch),
                    Value::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            self.compile_statement(else_branch)
                        } else {
                            Ok(None)
                        }
                    },
                    _ => Err(CrabbyError::CompileError("Condition must be boolean".into()))
                }
            },
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
            },
            Statement::Block(statements) => {
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
                Ok(None)
            },
            Statement::Expression(expr) => {
                let value = self.compile_expression(expr)?;
                Ok(Some(value))
            },
            Statement::Import { name, source } => {
                if let Some(source_path) = source {
                    let module = self.load_and_import_module(name, source_path);
                    if let Some(value) = module.public_items.get(name) {
                        self.module.variable.insert(name.clone(), value.clone());
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
            _ => Ok(None)
        }
    }

    pub fn compile_expression(&mut self, expr: &Expression) -> Result<Value, CrabbyError> {
        match expr {
            Expression::Integer(n) => Ok(Value::Integer(*n)),
            Expression::Float(f) => Ok(Value::Float(*f)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(value) => Ok(Value::Integer(if *value { 1 } else { 0 })),
            Expression::Where { expr, condition, body } => {
                let cond_value = self.compile_expression(condition)?;
                match cond_value {
                    Value::Boolean(true) => {
                        self.compile_statement(body);
                        Ok(self.compile_expression(expr)?)
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
                PatternKind::Literal(expr) => {
                    self.compile_expression(expr)
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
