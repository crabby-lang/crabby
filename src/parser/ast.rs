#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
    },
    Let {
        name: String,
        value: Box<Expression>,
    },
    Return(Box<Expression>),
    If {
        condition: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Async {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    Await {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    And {
        left: String,
        right: String,
    },
    While {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
        where_clause: Option<Box<Expression>>,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
        where_clause: Option<Box<Expression>>,
    },
    Loop {
        count: Box<Expression>,
        body: Box<Statement>,
    },
    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    ForIn {
        variable: String,
        iterator: Box<Expression>,
        body: Box<Statement>,
    },
    Import {
        name: String,
        source: Option<String>,
    },
    Block(Vec<Statement>),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Expression,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<Expression>>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_expr: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    Variable(String),
    Range(Box<Expression>),
    Boolean(bool),
    Pattern(Box<PatternKind>),
    Where {
        expr: Box<Expression>,
        condition: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    Literal(Box<Expression>),
    Variable(String),
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Dot,
    MatchOp,
}
