use std::fmt;

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
