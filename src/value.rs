// Value file that defines what value **exist** in Crabby.

use crate::utils::CrabbyError;
use crate::ast::Statement;

#[derive(Clone, PartialEq)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Box<Statement>,
}

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Lambda(Function),
    Boolean(bool),
    Array(Vec<Value>),
    Void,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Lambda(_), Value::Lambda(_)) => false,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Void, Value::Void) => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Lambda(_) => "<lambda>".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(elements) => {
                let elements_str: Vec<String> = elements.iter()
                    .map(|e| e.to_string())
                    .collect();
                format!("[{}]", elements_str.join(", "))
            },
            Value::Void => "void".to_string()
        }
    }

    pub fn matches(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        }
    }

    pub fn get_index(&self, index: i64) -> Result<Value, CrabbyError> {
        match self {
            Value::Array(elements) => {
                if index < 0 || index >= elements.len() as i64 {
                    Err(CrabbyError::InterpreterError(format!(
                        "Array index out of bounds: {}", index
                    )))
                } else {
                    Ok(elements[index as usize].clone())
                }
            }
            _ => Err(CrabbyError::InterpreterError(
                "Cannot index non-array value".to_string()
            )),
        }
    }
}
