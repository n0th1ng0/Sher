use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use crate::value::Value;
use crate::types::SherType;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub value: Value,
    pub var_type: SherType,
    pub is_const: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Variable>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            parent: None,
        }))
    }

    pub fn new_with_parent(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn define(&mut self, name: String, value: Value, var_type: SherType, is_const: bool) {
        self.values.insert(name, Variable { value, var_type, is_const });
    }

    pub fn get(&self, name: &str) -> Option<Variable> {
        if let Some(var) = self.values.get(name) {
            Some(var.clone())
        } else if let Some(ref parent) = self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, new_value: Value) -> Result<(), (String, Option<String>)> {
        if let Some(var) = self.values.get_mut(name) {
            if var.is_const {
                return Err((
                    format!("Cannot reassign constant variable '{}' declared with 'let'", name),
                    Some("Declare the variable with 'var' instead of 'let' if it needs to be mutable".to_string()),
                ));
            }

            let mut final_val = new_value;
            if let Value::String(ref s) = final_val {
                match var.var_type {
                    SherType::Int8
                    | SherType::Int16
                    | SherType::Int26
                    | SherType::Int32
                    | SherType::Int64 => {
                        if let Ok(n) = s.trim().parse::<i64>() {
                            final_val = Value::Int(n);
                        }
                    }
                    SherType::Float8
                    | SherType::Float16
                    | SherType::Float32
                    | SherType::Float64 => {
                        if let Ok(f) = s.trim().parse::<f64>() {
                            final_val = Value::Float(f);
                        }
                    }
                    SherType::Bool => {
                        let trimmed = s.trim().to_lowercase();
                        if trimmed == "true" || trimmed == "1" || trimmed == "tak" || trimmed == "yes" {
                            final_val = Value::Bool(true);
                        } else if trimmed == "false" || trimmed == "0" || trimmed == "nie" || trimmed == "no" {
                            final_val = Value::Bool(false);
                        }
                    }
                    SherType::Char => {
                        let trimmed = s.trim();
                        if trimmed.chars().count() == 1 {
                            final_val = Value::Char(trimmed.chars().next().unwrap());
                        }
                    }
                    _ => {}
                }
            }

            if let Value::Int(n) = final_val {
                match var.var_type {
                    SherType::Float8
                    | SherType::Float16
                    | SherType::Float32
                    | SherType::Float64 => {
                        final_val = Value::Float(n as f64);
                    }
                    _ => {}
                }
            }

            if !final_val.matches_type(&var.var_type) {
                return Err((
                    format!(
                        "Type mismatch: cannot assign value of type '{}' to variable '{}' of type '{}'",
                        final_val.get_type(),
                        name,
                        var.var_type
                    ),
                    Some(format!("Expected a value compatible with '{}'", var.var_type)),
                ));
            }
            var.value = final_val;
            Ok(())
        } else if let Some(ref parent) = self.parent {
            parent.borrow_mut().assign(name, new_value)
        } else {
            Err((
                format!("Undefined variable '{}'", name),
                Some(format!("Declare the variable before using it: var int32: {} = ...;", name)),
            ))
        }
    }
}
