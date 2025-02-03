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

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
        return_type: String,
        docstring: String,
    },
    Let {
        name: String,
        value: Box<Expression>,
    },
    // Const {
    //    name: String,
    //    value: Box<Expression>,
    // },
    Var {
        name: String,
        value: Box<Expression>,
    },
    // Trait {

    // },
    // Impl {

    // },
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
    Macro {
        name: String,
        params: String,
        body: Box<Expression>,
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
    // Static {
    //    name: String,
    //    value: Option<Box<Expression>>,
    // },
    // Class {
    //    name: String,
    //    fields: Vec<StructField>,
    //    methods: Vec<Statement>,
    //    parent: Option<String>,
    //    implements: Vec<String>,
    // },
    ArrayAssign {
        array: Expression,
        index: Box<Expression>,
        value: Box<Expression>,
    },
    // Network {
    //    kind: NetworkOperation,
    //    address: Box<Expression>,
    //    port: Box<Expression>,
    //    body: Option<Box<Statement>>,
    // },
    Block(Vec<Statement>),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkOperation {
    Listen {
        addr: String,
        port: u16
    },
    Connect {
        addr: String,
        port: u16
    },
    Send {
        data: Box<Expression>,
        conn_index: usize
    },
    Receive,
    Bind {
        addr: String,
        port: u16
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Expression,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_expr: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    Variable(String),
    Range(Box<Expression>),
    Boolean(bool),
    Array(Vec<Expression>),
    Pattern(Box<PatternKind>),
    Where {
        expr: Box<Expression>,
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    Network {
        operation: NetworkOperation,
        handler: Option<Box<Expression>>,
    },
    FString {
        template: String,
        expressions: Vec<Expression>,
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
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternKind {
    Literal(Box<Expression>),
    Variable(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Dot,
    MatchOp,
}
