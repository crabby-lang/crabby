// Crabby's Memory Management and Safety
// By using lifetimes, Ownership and Borrowings. It makes Crabby memory safety as possible.
// However, Memory safeties aren't always perfect, and Crabby is still in early development.

use std::collections::HashMap;
use crate::utils::CrabbyError;
use crate::ast::{Expression, Statement, Program};

#[derive(Debug, Clone, PartialEq)]
enum Lifetime {
    Static,
    Local { scope_depth: u32 },
    Borrowed {
        from: String,
        scope_depth: u32,
        is_mutable: bool,
    },
}

#[derive(Debug, Clone)]
struct OwnershipInfo {
    lifetime: Lifetime,
    borrowed_count: u32,
    mut_borrowed: bool,
    initialized: bool,
}

pub struct MemoryChecker {
    ownership_map: HashMap<String, OwnershipInfo>,
    current_scope: u32,
    moved_variables: Vec<String>,
}

impl MemoryChecker {
    pub fn new() -> Self {
        Self {
            ownership_map: HashMap::new(),
            current_scope: 0,
            moved_variables: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for statement in &program.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), CrabbyError> {
        match stmt {
            Statement::Let { name, value } => {
                self.check_expression(value)?;

                self.ownership_map.insert(name.clone(), OwnershipInfo {
                    lifetime: Lifetime::Local { scope_depth: self.current_scope },
                    borrowed_count: 0,
                    mut_borrowed: false,
                    initialized: true,
                });
            }

            Statement::Var { name, value } => {
                self.check_expression(value)?;
                self.ownership_map.insert(name.clone(), OwnershipInfo {
                    lifetime: Lifetime::Local { scope_depth: self.current_scope },
                    borrowed_count: 0,
                    mut_borrowed: false,
                    initialized: true,
                });
            }

            Statement::Block(statements) => {
                self.current_scope += 1;
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.cleanup_scope(self.current_scope);
                self.current_scope -= 1;
            }

            Statement::FunctionDef { name: _, params, body, return_type: _, docstring: _ } => {
                self.current_scope += 1;

                for param in params {
                    self.ownership_map.insert(param.clone(), OwnershipInfo {
                        lifetime: Lifetime::Local { scope_depth: self.current_scope },
                        borrowed_count: 0,
                        mut_borrowed: false,
                        initialized: true,
                    });
                }

                self.check_statement(body)?;
                self.cleanup_scope(self.current_scope);
                self.current_scope -= 1;
            }

            Statement::Expression(expr) => {
                self.check_expression(expr)?;
            }

            Statement::ArrayAssign { array, index, value } => {
                if let Expression::Variable(name) = array {
                    self.check_mutable_access(name)?;
                }
                self.check_expression(index)?;
                self.check_expression(value)?;
            }

            Statement::While { condition, body } => {
                self.check_expression(condition)?;
                self.current_scope += 1;
                self.check_statement(body)?;
                self.cleanup_scope(self.current_scope);
                self.current_scope -= 1;
            }

            Statement::If { condition, then_branch, else_branch } => {
                self.check_expression(condition)?;

                self.current_scope += 1;
                self.check_statement(then_branch)?;
                self.cleanup_scope(self.current_scope);
                self.current_scope -= 1;

                if let Some(else_stmt) = else_branch {
                    self.current_scope += 1;
                    self.check_statement(else_stmt)?;
                    self.cleanup_scope(self.current_scope);
                    self.current_scope -= 1;
                }
            }

            Statement::Return(expr) => {
                self.check_expression(expr)?;
            }

            _ => {}
        }
        Ok(())
    }

    fn check_expression(&mut self, expr: &Expression) -> Result<(), CrabbyError> {
        match expr {
            Expression::Variable(name) => {
                if self.moved_variables.contains(name) {
                    return Err(CrabbyError::InterpreterError(
                        format!("Use of moved variable '{}'", name)
                    ));
                }

                if let Some(info) = self.ownership_map.get(name) {
                    if !info.initialized {
                        return Err(CrabbyError::InterpreterError(
                            format!("Use of uninitialized variable '{}'", name)
                        ));
                    }
                } else {
                    return Err(CrabbyError::InterpreterError(
                        format!("Use of undefined variable '{}'", name)
                    ));
                }
            }

            Expression::Binary { left, operator: _, right } => {
                self.check_expression(left)?;
                self.check_expression(right)?;
            }

            Expression::Call { function: _, arguments } => {
                for arg in arguments {
                    self.check_expression(arg)?;
                }
            }

            Expression::Index { array, index } => {
                self.check_expression(array)?;
                self.check_expression(index)?;
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression(elem)?;
                }
            }

            _ => {}
        }
        Ok(())
    }

    fn check_mutable_access(&self, var_name: &str) -> Result<(), CrabbyError> {
        if let Some(info) = self.ownership_map.get(var_name) {
            if info.borrowed_count > 0 {
                return Err(CrabbyError::InterpreterError(
                    format!("Cannot mutably access '{}' while borrowed", var_name)
                ));
            }
            if info.mut_borrowed {
                return Err(CrabbyError::InterpreterError(
                    format!("Cannot access '{}' while mutably borrowed", var_name)
                ));
            }
        }
        Ok(())
    }

    fn cleanup_scope(&mut self, scope: u32) {
        self.ownership_map.retain(|_, info| {
            match info.lifetime {
                Lifetime::Local { scope_depth } => scope_depth < scope,
                Lifetime::Borrowed { scope_depth, .. } => scope_depth < scope,
                Lifetime::Static => true,
            }
        });

        self.moved_variables.clear();
    }

    pub fn mark_moved(&mut self, var_name: &str) {
        self.moved_variables.push(var_name.to_string());
    }

    pub fn check_borrowable(&self, var_name: &str, mutable: bool) -> Result<(), CrabbyError> {
        if let Some(info) = self.ownership_map.get(var_name) {
            if mutable {
                if info.borrowed_count > 0 || info.mut_borrowed {
                    return Err(CrabbyError::InterpreterError(
                        format!("Cannot mutably borrow '{}' while already borrowed", var_name)
                    ));
                }
            } else if info.mut_borrowed {
                return Err(CrabbyError::InterpreterError(
                    format!("Cannot borrow '{}' while mutably borrowed", var_name)
                ));
            }
        }
        Ok(())
    }
}
