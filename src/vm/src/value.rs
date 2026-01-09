#[derive(Clone, Debug)]
pub enum ValueVM {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl ValueVM {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ValueVM::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            ValueVM::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            ValueVM::Boolean(b) => *b,
            ValueVM::Nil => false,
            _ => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ValueVM::Number(n) => n.to_string(),
            ValueVM::String(s) => s.to_string(),
            ValueVM::Boolean(b) => b.to_string(),
            ValueVM::Nil => "nil".to_string(),
        }
    }
}
