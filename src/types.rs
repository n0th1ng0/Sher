use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SherType {
    Int8,
    Int16,
    Int26,
    Int32,
    Int64,
    Float8,
    Float16,
    Float32,
    Float64,
    Char,
    String,
    Bool,
    Void,
    Any,
    Array(Box<SherType>),
    Tuple(Vec<SherType>),
    Map(Box<SherType>, Box<SherType>),
    Custom(String),
}

impl SherType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "int8" | "i8" => Some(SherType::Int8),
            "int16" | "i16" => Some(SherType::Int16),
            "int26" | "i26" => Some(SherType::Int26),
            "int32" | "i32" | "int" => Some(SherType::Int32),
            "int64" | "i64" => Some(SherType::Int64),

            "float8" | "f8" => Some(SherType::Float8),
            "float16" | "f16" => Some(SherType::Float16),
            "float32" | "f32" | "float" => Some(SherType::Float32),
            "float64" | "f64" => Some(SherType::Float64),

            "char" => Some(SherType::Char),
            "string" | "str" => Some(SherType::String),
            "bool" | "boolean" => Some(SherType::Bool),
            "void" => Some(SherType::Void),
            "any" => Some(SherType::Any),
            "array" => Some(SherType::Array(Box::new(SherType::Any))),
            "map" => Some(SherType::Map(Box::new(SherType::Any), Box::new(SherType::Any))),
            other => {
                if !other.is_empty() && (other.chars().next().unwrap().is_alphabetic() || other.starts_with('_')) {
                    Some(SherType::Custom(other.to_string()))
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for SherType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SherType::Int8 => write!(f, "int8"),
            SherType::Int16 => write!(f, "int16"),
            SherType::Int26 => write!(f, "int26"),
            SherType::Int32 => write!(f, "int32"),
            SherType::Int64 => write!(f, "int64"),
            SherType::Float8 => write!(f, "float8"),
            SherType::Float16 => write!(f, "float16"),
            SherType::Float32 => write!(f, "float32"),
            SherType::Float64 => write!(f, "float64"),
            SherType::Char => write!(f, "char"),
            SherType::String => write!(f, "string"),
            SherType::Bool => write!(f, "bool"),
            SherType::Void => write!(f, "void"),
            SherType::Any => write!(f, "any"),
            SherType::Array(inner) => write!(f, "[{}]", inner),
            SherType::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            SherType::Map(k, v) => write!(f, "map[{}, {}]", k, v),
            SherType::Custom(name) => write!(f, "{}", name),
        }
    }
}
