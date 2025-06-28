use std::fmt;

use crate::parser::{BinaryOp, Expression, Statement, NetworkOperation};
use crate::deadcode::DeadCodeWarning;
// use periphery::sys::gpio::Value;
use crate::compile::Value;

#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CrabbyError {
    #[error("Lexer error at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Parser error at line {line}, column {column}: {message}")]
    ParserError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Invalid match pattern: {0}")]
    InvalidMatchPattern(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Match operation error: {0}")]
    MatchError(String),

    #[error("Missing match arm: {0}")]
    MissingMatchArm(String),

    #[error("Missing case in match: {0}")]
    MissingCaseKeyword(String),

    #[error("Class error: {0}")]
    ClassError(String),

    #[error("Trait error: {0}")]
    TraitError(String),

    #[error("Implementation error: {0}")]
    ImplError(String),

    #[error("Async/Await error: {0}")]
    AsyncError(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Visibility error: {0}")]
    VisibilityError(String),

    #[error("Compilation error: {0}")]
    CompileError(String),
}

impl From<std::io::Error> for CrabbyError {
    fn from(error: std::io::Error) -> Self {
        CrabbyError::NetworkError(error.to_string())
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {} column {} (bytes {}-{})",
            self.line, self.column, self.start, self.end
        )
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Expression {
    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::Integer(a), Expression::Integer(b)) => a == b,
            (Expression::Float(a), Expression::Float(b)) => a == b,
            (Expression::String(a), Expression::String(b)) => a == b,
            (Expression::Variable(a), Expression::Variable(b)) => a == b,
            (Expression::Boolean(a), Expression::Boolean(b)) => a == b,
            (Expression::Array(a), Expression::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.matches(y))
            },
            (Expression::Index { array: a1, index: i1 },
             Expression::Index { array: a2, index: i2 }) => {
                a1.matches(a2) && i1.matches(i2)
            },
            _ => false,
        }
    }
}

impl fmt::Display for DeadCodeWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Integer(n) => write!(f, "{}", n),
            Expression::Float(f_val) => write!(f, "{}", f_val),
            Expression::String(s) => write!(f, "{}", s),
            Expression::Variable(name) => write!(f, "{}", name),
            Expression::Boolean(b) => write!(f, "{}", b),
            Expression::Range(count) => write!(f, "range({})", count),
            Expression::Pattern(pattern) => write!(f, "{:?}", pattern),
            Expression::Where { expr, condition, body } => {
                write!(f, "{} where {} {}", expr, condition, body)
            },
            Expression::Binary { left, operator, right } => {
                write!(f, "({} {} {})", left, operator, right)
            },
            Expression::FString { template, expressions: _ } => {
                write!(f, "f\"{}\"", template)
            },
            Expression::Await { expr } => {
                write!(f, "await {}", expr)
            },
            Expression::Network { operation, handler } => {
                write!(f, "Network({}, {:?})", operation, handler)
            },
            Expression::Call { function, arguments } => {
                write!(f, "{}({})", function,
                    arguments.iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            Expression::Lambda { params, body } => {
                write!(f, "lambda({}) {}",
                    params.join(", "),
                    body
                )
            },
            Expression::Array(elements) => {
                write!(f, "[{}]",
                    elements.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            Expression::Index { array, index } => {
                write!(f, "{}[{}]", array, index)
            },
        }
    }
}

impl std::fmt::Display for NetworkOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkOperation::Listen { addr, port } =>
                write!(f, "Listen({}:{})", addr, port),
            NetworkOperation::Connect { addr, port } =>
                write!(f, "Connect({}:{})", addr, port),
            NetworkOperation::Send { data, conn_index } =>
                write!(f, "Send({:?}, {})", data, conn_index),
            NetworkOperation::Receive { conn_index } =>
                write!(f, "Receive({})", conn_index),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Dot => write!(f, "."),
            BinaryOp::MatchOp => write!(f, "=>"),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Block(stmts) => {
                write!(f, "{{ {} }}",
                    stmts.iter()
                        .map(|stmt| stmt.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            },
            Statement::Expression(expr) => write!(f, "{}", expr),
            _ => write!(f, "{:?}", self),
        }
    }
}
