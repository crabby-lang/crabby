use logos::Logos;
use crate::utils::{CrabbyError, Span};

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    #[token("def")]
    Def,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("let")]
    Let,
    #[token("lambda")]
    Lambda,
    #[token("loop")]
    Loop,
    #[token("for")]
    For,
    #[token("as")]
    As,
    #[token("and")]
    And,
    #[token("with")]
    With,
    #[token("in")]
    In,
    #[token("where")]
    Where,
    #[token("range")]
    Range,
    #[token("macro")]
    Macro,
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("elseif")]
    Elseif,
    #[token("pub")]
    Public,
    #[token("private")] // You can have a function have private or not, if not it'll still treat it private
    Private,
    #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("mut")]
    Mutable,
    #[token("const")]
    Constant,
    #[token("class")]
    Class,
    #[token("extends")]
    Extends,
    #[token("except")]
    Except,
    #[token("expect")]
    Expect,
    #[token("throw")]
    Throw,
    #[token("new")]
    New,
    #[token("impl")]
    Implement,
    #[token("trait")]
    Trait,
    #[token("override")]
    Override,
    #[token("module")]
    Module,
    #[token("global")] // Acts like 'pub', you can just use 2 options depending on your needs
    Global,
    #[token("namespace")]
    Namespace,
    #[token("static")]
    Static,
    #[token("var")]
    Variable,
    #[token("do")] // Does looping (now there are 3 options)
    Do,

    // Imports
    #[token("import")]
    Import,
    #[token("from")]
    From,

    // Literals
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r#""[^"]*""#, |lex| Some(lex.slice().trim_matches('"').to_string()))]
    String(String),

    #[regex(r#"f'[^']*'"#, |lex| Some(lex.slice()[2..lex.slice().len()-1].to_string()))]
    #[regex(r#"f"[^"]*""#, |lex| Some(lex.slice()[2..lex.slice().len()-1].to_string()))]
    FString(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Some(lex.slice().to_string()))]
    Identifier(String),

    #[token("true")]
    True,

    #[token("false")]
    False,

    // Operators and delimiters
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("=")]
    Equals,
    #[token("$")]
    DollarSign,
    #[token("?")]
    QuestionMark,
    #[token("_")]
    Underscore,
    #[token("!=")]
    NotEquals,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("<=")]
    LessThanOrEqual,
    #[token(">=")]
    GreaterThanOrEqual,
    #[token("|>")]
    Pipe,
    #[token("||")]
    Or,
    #[token("=>")]
    Arrow,
    #[token("->")]
    CoolerArrow,
    #[token("!")]
    Not,
    #[token("@")]
    Decorator,
    #[token("==")]
    DoubleEquals,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    #[regex(r"[ \t\r\n]+", logos::skip)]
    #[regex(r"//[^\r\n]*", logos::skip)]
    Whitespace,
}

pub struct TokenStream<'source> {
    pub token: Token,
    pub span: Span,
    pub slice: &'source str,
}

pub fn tokenize(source: &str) -> Result<Vec<TokenStream>, CrabbyError> {
    let mut tokens = Vec::new();
    let mut lex = Token::lexer(source);
    let mut line = 1;
    let mut column = 1;

    // Track the last valid character position
    let mut last_valid_pos = 0;

    while let Some(token_result) = lex.next() {
        let span_start = lex.span().start;

        // Update line and column for any skipped whitespace
        for (_pos, ch) in source[last_valid_pos..span_start].chars().enumerate() {
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        match token_result {
            Ok(token) => {
                // Skip the Whitespace token as it's handled above
                if matches!(token, Token::Whitespace) {
                    continue;
                }

                let span = Span::new(
                    span_start,
                    lex.span().end,
                    line,
                    column,
                );

                tokens.push(TokenStream {
                    token,
                    span,
                    slice: lex.slice(),
                });

                // Update column for the token
                column += lex.slice().len();
                last_valid_pos = lex.span().end;
            }
            Err(_) => {
                if last_valid_pos < source.len() {
                    let problem_char = source[span_start..].chars().next()
                        .map(|c| format!("'{}'", c))
                        .unwrap_or_else(|| "unknown".to_string());

                    return Err(CrabbyError::LexerError {
                        line,
                        column,
                        message: format!("Invalid character {} at position {}", problem_char, span_start),
                    });
                }
            }
        }
    }

    if tokens.is_empty() {
        return Err(CrabbyError::LexerError {
            line: 1,
            column: 1,
            message: "Empty source file".to_string(),
        });
    }

    Ok(tokens)
}
