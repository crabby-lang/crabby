// Crabby scans crab code then checks if it's a dead/unused code or not

use crate::ast::{Expression, Program, Statement};
use crate::utils::CrabbyError;
use std::collections::{HashMap, HashSet};

pub struct DeadCodeAnalyzer {
    defined_symbols: HashMap<String, SymbolInfo>,
    used_symbols: HashSet<String>,
    pub_exports: HashSet<String>,
}

#[derive(Debug)]
pub struct SymbolInfo {
    kind: SymbolKind,
    line: usize,
    column: usize,
}

#[derive(Debug)]
pub struct DeadCodeWarning {
    pub symbol: String,
    pub kind: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum SymbolKind {
    Function,
    Variable,
    Struct,
    Enum,
    Macro,
}

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self {
            defined_symbols: HashMap::new(),
            used_symbols: HashSet::new(),
            pub_exports: HashSet::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<Vec<DeadCodeWarning>, CrabbyError> {
        self.collect_definitions(program)?;
        self.collect_usage(program)?;

        // Generate warnings for unused symbols
        let mut warnings = Vec::new();

        for (name, info) in &self.defined_symbols {
            // Skips if symbol is public or used
            if self.pub_exports.contains(name) || self.used_symbols.contains(name) {
                continue;
            }

            warnings.push(DeadCodeWarning {
                symbol: name.clone(),
                kind: info.kind.to_string(),
                line: info.line,
                column: info.column,
            });
        }

        Ok(warnings)
    }

    fn collect_definitions(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for stmt in &program.statements {
            match stmt {
                Statement::FunctionDef {
                    name,
                    params: _,
                    body: _,
                    return_type: _,
                    docstring: _,
                    visibility: _,
                } => {
                    if name.starts_with("pub ") {
                        let clean_name = name.trim_start_matches("pub ").to_string();
                        self.pub_exports.insert(clean_name.clone());
                        self.add_symbol(clean_name, SymbolKind::Function, 0, 0);
                    } else {
                        self.add_symbol(name.clone(), SymbolKind::Function, 0, 0);
                    }
                }
                Statement::Let { name, value: _ } => {
                    if name.starts_with("pub ") {
                        let clean_name = name.trim_start_matches("pub ").to_string();
                        self.pub_exports.insert(clean_name.clone());
                        self.add_symbol(clean_name, SymbolKind::Variable, 0, 0);
                    } else {
                        self.add_symbol(name.clone(), SymbolKind::Variable, 0, 0);
                    }
                }
                Statement::Struct {
                    name,
                    fields: _,
                    where_clause: _,
                } => {
                    self.add_symbol(name.clone(), SymbolKind::Struct, 0, 0);
                }
                Statement::Enum {
                    name,
                    variants: _,
                    where_clause: _,
                } => {
                    self.add_symbol(name.clone(), SymbolKind::Enum, 0, 0);
                }
                // Statement::Macro {
                //     name,
                //     params: _,
                //     body: _,
                // } => {
                //     self.add_symbol(name.clone(), SymbolKind::Macro, 0, 0);
                // }
                _ => {}
            }
        }
        Ok(())
    }

    fn collect_usage(&mut self, program: &Program) -> Result<(), CrabbyError> {
        for stmt in &program.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), CrabbyError> {
        match stmt {
            Statement::Expression(expr) => self.analyze_expression(expr)?,
            Statement::Block(statements) => {
                for stmt in statements {
                    self.analyze_statement(stmt)?;
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.analyze_statement(else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(body)?;
            }
            Statement::ForIn {
                variable: _,
                iterator,
                body,
            } => {
                self.analyze_expression(iterator)?;
                self.analyze_statement(body)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<(), CrabbyError> {
        match expr {
            Expression::Variable(name) => {
                self.used_symbols.insert(name.clone());
            }
            Expression::Call {
                function,
                arguments,
            } => {
                self.used_symbols.insert(function.clone());
                for arg in arguments {
                    self.analyze_expression(arg)?;
                }
            }
            Expression::Binary {
                left,
                operator: _,
                right,
            } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)?;
            }
            Expression::Where {
                expr,
                condition,
                body,
            } => {
                self.analyze_expression(expr)?;
                self.analyze_expression(condition)?;
                self.analyze_statement(body)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn add_symbol(&mut self, name: String, kind: SymbolKind, line: usize, column: usize) {
        self.defined_symbols
            .insert(name, SymbolInfo { kind, line, column });
    }
}

impl SymbolKind {
    fn to_string(&self) -> String {
        match self {
            SymbolKind::Function => "function".to_string(),
            SymbolKind::Variable => "variable".to_string(),
            SymbolKind::Struct => "struct".to_string(),
            SymbolKind::Enum => "enum".to_string(),
            SymbolKind::Macro => "macro".to_string(),
        }
    }
}
