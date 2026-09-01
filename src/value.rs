use std::collections::HashMap;
use std::fmt;
use crate::types::SherType;
use crate::ast::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Map(std::rc::Rc<std::cell::RefCell<Vec<(Value, Value)>>>),
    Struct {
        struct_name: String,
        fields: HashMap<String, Value>,
    },
    Enum {
        enum_name: String,
        variant: String,
        value: Option<Box<Value>>,
    },
    Function {
        name: Option<String>,
        params: Vec<(String, SherType)>,
        return_type: SherType,
        body: Vec<Stmt>,
    },
    Null,
}

impl Value {
    pub fn get_type(&self) -> SherType {
        match self {
            Value::Int(n) => {
                if *n >= i8::MIN as i64 && *n <= i8::MAX as i64 {
                    SherType::Int8
                } else if *n >= i16::MIN as i64 && *n <= i16::MAX as i64 {
                    SherType::Int16
                } else if *n >= -33554432 && *n <= 33554431 {
                    SherType::Int26
                } else if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                    SherType::Int32
                } else {
                    SherType::Int64
                }
            }
            Value::Float(_) => SherType::Float64,
            Value::Char(_) => SherType::Char,
            Value::String(_) => SherType::String,
            Value::Bool(_) => SherType::Bool,
            Value::Array(items) => {
                if let Some(first) = items.first() {
                    SherType::Array(Box::new(first.get_type()))
                } else {
                    SherType::Array(Box::new(SherType::Any))
                }
            }
            Value::Tuple(items) => {
                let types = items.iter().map(|it| it.get_type()).collect();
                SherType::Tuple(types)
            }
            Value::Map(entries_cell) => {
                let entries = entries_cell.borrow();
                if let Some((k, v)) = entries.first() {
                    SherType::Map(Box::new(k.get_type()), Box::new(v.get_type()))
                } else {
                    SherType::Map(Box::new(SherType::Any), Box::new(SherType::Any))
                }
            }
            Value::Struct { struct_name, .. } => SherType::Custom(struct_name.clone()),
            Value::Enum { enum_name, .. } => SherType::Custom(enum_name.clone()),
            Value::Function { .. } => SherType::Any,
            Value::Null => SherType::Void,
        }
    }

    pub fn matches_type(&self, expected: &SherType) -> bool {
        match expected {
            SherType::Any => true,
            SherType::Void => matches!(self, Value::Null),
            SherType::Int8 => match self {
                Value::Int(n) => *n >= i8::MIN as i64 && *n <= i8::MAX as i64,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Int16 => match self {
                Value::Int(n) => *n >= i16::MIN as i64 && *n <= i16::MAX as i64,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Int26 => match self {
                Value::Int(n) => *n >= -33554432 && *n <= 33554431,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Int32 => match self {
                Value::Int(n) => *n >= i32::MIN as i64 && *n <= i32::MAX as i64,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Int64 => match self {
                Value::Int(_) => true,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Float8 | SherType::Float16 | SherType::Float32 | SherType::Float64 => {
                match self {
                    Value::Float(_) | Value::Int(_) => true,
                    Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                    _ => false,
                }
            }
            SherType::Char => match self {
                Value::Char(_) => true,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::String => match self {
                Value::String(_) => true,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Bool => match self {
                Value::Bool(_) => true,
                Value::Array(items) => items.iter().all(|it| it.matches_type(expected)),
                _ => false,
            },
            SherType::Array(inner) => match self {
                Value::Array(items) => items.iter().all(|it| it.matches_type(inner)),
                _ => false,
            },
            SherType::Tuple(types) => match self {
                Value::Tuple(items) => {
                    if items.len() != types.len() {
                        return false;
                    }
                    items.iter().zip(types.iter()).all(|(item, expected_t)| item.matches_type(expected_t))
                }
                _ => false,
            },
            SherType::Map(expected_k, expected_v) => match self {
                Value::Map(entries_cell) => {
                    entries_cell.borrow().iter().all(|(k, v)| k.matches_type(expected_k) && v.matches_type(expected_v))
                }
                _ => false,
            },
            SherType::Custom(expected_name) => match self {
                Value::Struct { struct_name, .. } => struct_name == expected_name,
                Value::Enum { enum_name, .. } => enum_name == expected_name,
                _ => false,
            },
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Tuple(tup) => !tup.is_empty(),
            Value::Map(map_cell) => !map_cell.borrow().is_empty(),
            Value::Struct { .. } => true,
            Value::Enum { .. } => true,
            Value::Null => false,
            Value::Char(c) => *c != '\0',
            Value::Function { .. } => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Char(c) => write!(f, "{}", c),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::Map(entries_cell) => {
                let entries = entries_cell.borrow();
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Struct { struct_name, fields } => {
                write!(f, "{} {{ ", struct_name)?;
                let mut first = true;
                for (name, val) in fields {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}: {}", name, val)?;
                }
                write!(f, " }}")
            }
            Value::Enum { enum_name, variant, .. } => {
                write!(f, "{}::{}", enum_name, variant)
            }
            Value::Function { name, .. } => {
                if let Some(n) = name {
                    write!(f, "<func {}>", n)
                } else {
                    write!(f, "<anonymous func>")
                }
            }
            Value::Null => write!(f, "null"),
        }
    }
}
