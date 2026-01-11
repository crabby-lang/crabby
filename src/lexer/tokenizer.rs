use crate::utils::{CrabbyError, ErrorLocation, Span};
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // KEYWORDS
    #[token("def")]
    Def,
    #[token("fun")]
    Function,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("elif")]
    ElIf,
    #[token("while")]
    While,
    #[token("let")]
    Let,
    #[token("assert")]
    Assert,
    #[token("mode")]
    Mode,
    #[token("lambda")]
    Lambda,
    #[token("chan")]
    Channel,
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
    #[token("not")]
    Not,
    #[token("where")]
    Where,
    #[token("range")]
    Range,
    /**
     * Metaprogramming using **macros** in Crabby
     *
     * Example:
     *
     * macro something {
     *   /* macro code goes here */
     * }
     *
     * Use this if you want to use a macro in your code (`!` symbol)
     * For example: `def { something!(/* logic of your macro */) }`
     */
    #[token("macro")]
    Macro,
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("pub")]
    Public,
    /**
     * You can make a function have private or not
     * if not it'll still treat it as private
     *
     * You can freely choose to whether you use the **private** keyword or not.
     */
    #[token("private")]
    Private,
    #[token("protect")]
    Protect,
    /**
     * Runs language functions inside Crabby
     *
     * Refer to `examples/foreign.crab` for more information.
     */
    #[token("foreign")]
    Foreign,
    /**
     *
     * Yield - lazy~ish generator inside functions! :3
     *
     * def yieldFunction() {
     *      yield 1
     *      yield 2
     *      yield 3
     *      ...
     * }
     *
     */
    #[token("yield")]
    Yield,
    /**
     * Generators in Crabby (!! EXPERIMENTAL !!)
     *
     * Creates `resumable` iterators, similar to how `async` blocks work for AWAITABLES
     *
     * Ex:
     *
     * gen {
     *   yield 1
     *   yield 2
     *   yield 3
     *   ...
     * }
     *
     */
    #[token("gen")]
    Generate,
    #[token("union")]
    Union,
    #[token("interface")]
    Interface,
    #[token("unless")]
    Unless,
    #[token("until")]
    Until,
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
    #[token("extend")]
    Extend,
    #[token("except")]
    Except,
    #[token("expect")]
    Expect,
    #[token("throw")]
    Throw,
    #[token("impl")]
    Implement,
    #[token("trait")]
    Trait,
    #[token("override")]
    Override,
    #[token("extern")]
    Extern,
    #[token("global")] // Acts like 'pub', you can just use 2 options depending on your needs
    Global,
    #[token("static")]
    Static,
    #[token("var")]
    Variable,
    #[token("do")] // Does looping (now there are 3 options)
    Do,
    #[token("or")]
    OrKeyword,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("ref")]
    Reference,
    #[token("move")] // Move keyboard
    Move,
    /**
     *  Unsafe code in Crabby (Don't panic if you're a beginner)
     *
     *  In Crabby, the use of `unsafe` is for programmers who want to touch the memory,
     *  the core hardware or just maybe doing silly stuffs in Assembly :3
     *
     *  unsafe {
     *      @asm(
     *          "syscall"
     *    ) // Syscall instruction
     *  }
     *
     *  Introduces for C, C++ & Assembly FFI feature
     */
    #[token("unsafe")]
    Unsafe,
    #[token("del")]
    Delete,
    #[token("finally")]
    Finally,
    #[token("is")]
    Is,
    #[token("typedef")]
    TypeDef,
    #[token("typeof")]
    TypeOf,
    #[token("continue")]
    Continue,
    #[token("break")]
    Break,
    #[token("pass")]
    Pass,
    #[token("maybe")]
    Maybe,
    #[token("probably")]
    Probably,
    /**
     *
     * Nonlocal in Crabby (!! EXPERIMENTAL !!)
     *
     */
    #[token("nonlocal")]
    NonLocal,
    #[token("raise")]
    Raise,
    #[token("virtual")]
    Virtual,
    #[token("go")]
    Routines,

    // Imports
    #[token("import")]
    Import,
    #[token("from")]
    From,

    // LITERALS
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

    #[token("null")]
    Null,
    #[token("nil")]
    Nil,

    // OPERATORS and DELIMITERS
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
    #[token("_", priority = 3)]
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
    #[token("!")] // Can be used for macros or a "!= / `not` (keyword)" operator
    ExclamationMark,
    #[token("&&")]
    DoubleAmpersand,
    /**
     * Ampersand - For borrowing!
     *
     * let a = "a"
     * leb b = &a <-- borrowed
     *
     * You can use the `move` keyword too as an alternative approach.
     *
     * Not to be confused with the 'and' keyword or '&&' operator.
     */
    #[token("&")]
    Ampersand,
    /**
     * Crabby's decorator system - Metaprogramming
     * In Crabby, the `@` symbol indicates a decorator call,
     * similar to Python's decorator feature:
     *
     * ```
     * def sprinkles() {
     *  print("Adding Sprinkles!❄️")
     * }
     *
     * @sprinkles
     * def ice_cream() {
     *  print("Here is your ice cream! 🍨")
     * }
     * ```
     *
     */
    #[token("@")]
    Decorator,
    #[token("%")]
    Percentage,
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

pub struct TokenStream {
    pub token: Token,
    pub span: Span,
    pub len: String,
    pub slice: String,
    pub source: String,
}

impl TokenStream {
    // y was this `async`?
    pub fn tokenize(source: String) -> Result<Vec<Self>, CrabbyError> {
        let mut tokens = Vec::new();
        let mut lex = Token::lexer(&source);
        let mut line = 1;
        let mut column = 1;

        // Track the last valid character position
        let mut last_valid_pos = 0;

        while let Some(token_result) = lex.next() {
            let span_start = lex.span().start;

            // Update line and column for any skipped whitespace
            for (_pos, ch) in source[last_valid_pos..span_start]
                .chars()
                .enumerate()
                .clone()
            {
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

                    let span = Span::new(span_start, lex.span().end, line, column);

                    tokens.push(Self {
                        token,
                        span,
                        len: String::default(), // FIX
                        slice: lex.slice().to_string(),
                        source: source.clone(),
                    });

                    // Update column for the token
                    column += lex.slice().len();
                    last_valid_pos = lex.span().end;
                }
                Err(_) => {
                    if last_valid_pos < source.len() {
                        let problem_char = source[span_start..]
                            .chars()
                            .next()
                            .map(|c| format!("'{}'", c))
                            .unwrap_or_else(|| "unknown".to_string());

                        return Err(CrabbyError::LexerError(ErrorLocation {
                            line,
                            column,
                            message: format!(
                                "Invalid character {} at position {}",
                                problem_char, span_start
                            ),
                        }));
                    }
                }
            }
        }

        if tokens.is_empty() {
            return Err(CrabbyError::LexerError(ErrorLocation {
                line: 1,
                column: 1,
                message: "Empty source file".to_string(),
            }));
        }

        Ok(tokens)
    }
}
