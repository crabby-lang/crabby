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
        visibiity: Visibility,
    },
    FunctionFun {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
        return_type: String,
        docstring: String,
        visibiity: Visibility,
    },
    Let {
        name: String,
        value: Box<Expression>,
    },
    Const {
        name: String,
        value: Box<Expression>,
    },
    Var {
        name: String,
        value: Box<Expression>,
    },
    Return(Box<Expression>),
    If {
        condition: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    AsyncFunction {
        name: String,
        params: Vec<String>,
        body: Box<Statement>,
        return_type: Option<String>,
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
    // Macro {
    //    name: String,
    //    params: String,
    //    body: Box<Expression>,
    // },
    // Mutable {
    //    name: String,
    // }, // the `mut` keyword
    // Unless {
    //    name: String,
    //    body: Box<Statement>,
    // },
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
    Class {
        name: String,
        parent: Option<String>,
        methods: Vec<Statement>,
        fields: Vec<String>,
    },
    Extend {
        class: String,
        parent: String,
        methods: Vec<Statement>,
    },
    Trait {
        name: String,
        methods: Vec<MethodDefinition>,
    },
    // Maybe {
    //    name: String,
    // },
    // Probably {
    //    name: String,
    // },
    Impl {
        target: String,
        trait_name: Option<String>,
        methods: Vec<MethodDefinition>,
    },
    ArrayAssign {
        array: Expression,
        index: Box<Expression>,
        value: Box<Expression>,
    },
    Block(Vec<Statement>),
    Expression(Expression),
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
    FString {
        template: String,
        expressions: Vec<Expression>,
    },
    Await {
        expr: Box<Expression>,
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
pub enum Visibility {
    Public,
    Private,
    Protect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Statement>,
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
