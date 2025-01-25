use std::fmt;
use crate::parser::ast::BinaryOp;
use crate::parser::ast::Expression;
use crate::parser::ast::Statement;

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

#[derive(Debug, thiserror::Error)]
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

    #[error("Match operation error: {0}")]
    MatchError(String),

    #[error("Missing match arm: {0}")]
    MissingMatchArm(String),

    #[error("Missing case in match: {0}")]
    MissingCaseKeyword(String),

    #[error("Compilation error: {0}")]
    CompileError(String),
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

impl Expression {
    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::Integer(a), Expression::Integer(b)) => a == b,
            (Expression::Float(a), Expression::Float(b)) => a == b,
            (Expression::String(a), Expression::String(b)) => a == b,
            (Expression::Variable(a), Expression::Variable(b)) => a == b,
            (Expression::Boolean(a), Expression::Boolean(b)) => a == b,
            _ => false,
        }
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