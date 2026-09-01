use std::cell::RefCell;
use std::rc::Rc;
use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::environment::Environment;
use crate::token::Span;
use crate::types::SherType;
use crate::value::Value;

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

enum ControlFlow {
    None,
    Return(Value),
    Break,
    Continue,
}

fn next_rand_u64() -> u64 {
    use std::cell::Cell;
    thread_local! {
        static RNG_STATE: Cell<u64> = Cell::new({
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(123456789) as u64;
            nanos.wrapping_mul(0x9E3779B97F4A7C15) ^ 0xBF58476D1CE4E5B9
        });
    }
    RNG_STATE.with(|cell| {
        let mut x = cell.get();
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        let result = z ^ (z >> 31);
        cell.set(x);
        result
    })
}

fn val_to_f64(val: &Value) -> Option<f64> {
    match val {
        Value::Float(f) => Some(*f),
        Value::Int(n) => Some(*n as f64),
        _ => None,
    }
}

pub struct Interpreter {
    global_env: Rc<RefCell<Environment>>,
    current_file: Option<std::path::PathBuf>,
    struct_defs: std::collections::HashMap<String, Vec<(String, SherType)>>,
    enum_defs: std::collections::HashMap<String, Vec<(String, Option<Value>)>>,
}

impl Interpreter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            global_env: Environment::new(),
            current_file: None,
            struct_defs: std::collections::HashMap::new(),
            enum_defs: std::collections::HashMap::new(),
        }
    }

    pub fn with_file(file_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            global_env: Environment::new(),
            current_file: Some(file_path.into()),
            struct_defs: std::collections::HashMap::new(),
            enum_defs: std::collections::HashMap::new(),
        }
    }

    fn resolve_path(&self, raw_path: &str) -> std::path::PathBuf {
        let path = std::path::Path::new(raw_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref current) = self.current_file {
            if let Some(parent) = current.parent() {
                parent.join(path)
            } else {
                path.to_path_buf()
            }
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in statements {
            let res = self.execute_stmt(stmt, Rc::clone(&self.global_env))?;
            if let ControlFlow::Return(_) = res {
                return Ok(());
            }
        }

        // If a function named `main` was defined, auto-execute it if it takes 0 params
        let main_fn = {
            let env = self.global_env.borrow();
            env.get("main")
        };

        if let Some(var) = main_fn {
            if let Value::Function { params, body, .. } = var.value {
                if params.is_empty() {
                    let call_env = Environment::new_with_parent(Rc::clone(&self.global_env));
                    self.execute_block(&body, call_env)?;
                }
            }
        }

        Ok(())
    }

    fn execute_stmt(
        &mut self,
        stmt: &Stmt,
        env: Rc<RefCell<Environment>>,
    ) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::VarDecl {
                is_const,
                var_type,
                name,
                init,
                span,
            } => {
                let mut val = self.eval_expr(init, Rc::clone(&env))?;

                if let Value::String(ref s) = val {
                    match var_type {
                        SherType::Int8
                        | SherType::Int16
                        | SherType::Int26
                        | SherType::Int32
                        | SherType::Int64 => {
                            if let Ok(n) = s.trim().parse::<i64>() {
                                val = Value::Int(n);
                            }
                        }
                        SherType::Float8
                        | SherType::Float16
                        | SherType::Float32
                        | SherType::Float64 => {
                            if let Ok(f) = s.trim().parse::<f64>() {
                                val = Value::Float(f);
                            }
                        }
                        SherType::Bool => {
                            let trimmed = s.trim().to_lowercase();
                            if trimmed == "true" || trimmed == "1" || trimmed == "tak" || trimmed == "yes" {
                                val = Value::Bool(true);
                            } else if trimmed == "false" || trimmed == "0" || trimmed == "nie" || trimmed == "no" {
                                val = Value::Bool(false);
                            }
                        }
                        SherType::Char => {
                            let trimmed = s.trim();
                            if trimmed.chars().count() == 1 {
                                val = Value::Char(trimmed.chars().next().unwrap());
                            }
                        }
                        _ => {}
                    }
                }

                if let Value::Int(n) = val {
                    match var_type {
                        SherType::Float8
                        | SherType::Float16
                        | SherType::Float32
                        | SherType::Float64 => {
                            val = Value::Float(n as f64);
                        }
                        _ => {}
                    }
                }

                if !val.matches_type(var_type) {
                    return Err(RuntimeError {
                        message: format!(
                            "Type mismatch: variable '{}' expects type '{}', but was assigned value '{}' of type '{}'",
                            name,
                            var_type,
                            val,
                            val.get_type()
                        ),
                        span: *span,
                        hint: Some(format!(
                            "Change the variable type to '{}' or assign a value of matching type",
                            val.get_type()
                        )),
                    });
                }

                env.borrow_mut()
                    .define(name.clone(), val, var_type.clone(), *is_const);
                Ok(ControlFlow::None)
            }

            Stmt::StructDef { name, fields, .. } => {
                self.struct_defs.insert(name.clone(), fields.clone());
                Ok(ControlFlow::None)
            }

            Stmt::EnumDef { name, variants, span: _ } => {
                let mut evaluated_variants = Vec::new();
                for (vname, vexpr) in variants {
                    let val = match vexpr {
                        Some(e) => Some(self.eval_expr(e, Rc::clone(&env))?),
                        None => None,
                    };
                    evaluated_variants.push((vname.clone(), val));
                }
                self.enum_defs.insert(name.clone(), evaluated_variants);
                Ok(ControlFlow::None)
            }

            Stmt::FuncDecl {
                name,
                params,
                return_type,
                body,
                span: _,
            } => {
                let func_val = Value::Function {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: body.clone(),
                };

                if let Some(ref fname) = name {
                    env.borrow_mut()
                        .define(fname.clone(), func_val, SherType::Any, false);
                    Ok(ControlFlow::None)
                } else {
                    let local_env = Environment::new_with_parent(Rc::clone(&env));
                    let res = self.execute_block(body, local_env)?;
                    if let ControlFlow::Return(_) = res {
                        Ok(ControlFlow::None)
                    } else {
                        Ok(ControlFlow::None)
                    }
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_val = self.eval_expr(condition, Rc::clone(&env))?;
                if cond_val.is_truthy() {
                    let local_env = Environment::new_with_parent(Rc::clone(&env));
                    self.execute_block(then_branch, local_env)
                } else if let Some(else_stmts) = else_branch {
                    let local_env = Environment::new_with_parent(Rc::clone(&env));
                    self.execute_block(else_stmts, local_env)
                } else {
                    Ok(ControlFlow::None)
                }
            }

            Stmt::While { condition, body, .. } => {
                while self.eval_expr(condition, Rc::clone(&env))?.is_truthy() {
                    let local_env = Environment::new_with_parent(Rc::clone(&env));
                    let res = self.execute_block(body, local_env)?;
                    match res {
                        ControlFlow::Break => break,
                        ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                        ControlFlow::Continue | ControlFlow::None => {}
                    }
                }
                Ok(ControlFlow::None)
            }

            Stmt::ForIn {
                item_name,
                item_type,
                iterable,
                body,
                span,
            } => {
                let items: Vec<Value> = if let Expr::Range { start, end, span: rspan } = iterable {
                    let s_val = self.eval_expr(start, Rc::clone(&env))?;
                    let e_val = self.eval_expr(end, Rc::clone(&env))?;
                    match (s_val, e_val) {
                        (Value::Int(s), Value::Int(e)) => {
                            if s <= e {
                                (s..e).map(Value::Int).collect()
                            } else {
                                (e + 1..=s).rev().map(Value::Int).collect()
                            }
                        }
                        (s, e) => {
                            return Err(RuntimeError {
                                message: format!("Range bounds must be integers, got '{}' and '{}'", s.get_type(), e.get_type()),
                                span: *rspan,
                                hint: Some("Use integer range syntax: for (let int32: i in 0..5)".to_string()),
                            });
                        }
                    }
                } else {
                    let iter_val = self.eval_expr(iterable, Rc::clone(&env))?;
                    match iter_val {
                        Value::Array(arr) => arr,
                        Value::Tuple(tup) => tup,
                        Value::String(s) => s.chars().map(Value::Char).collect(),
                        Value::Map(map_cell) => map_cell.borrow().iter().map(|(k, _)| k.clone()).collect(),
                        _ => {
                            return Err(RuntimeError {
                                message: format!("Cannot iterate over value of type '{}'", iter_val.get_type()),
                                span: *span,
                                hint: Some("For-in loops can iterate over ranges (0..5), arrays ([1, 2]), maps, strings, or tuples".to_string()),
                            });
                        }
                    }
                };

                for item in items {
                    let local_env = Environment::new_with_parent(Rc::clone(&env));
                    local_env.borrow_mut().define(
                        item_name.clone(),
                        item,
                        item_type.clone(),
                        false,
                    );
                    let res = self.execute_block(body, local_env)?;
                    match res {
                        ControlFlow::Break => break,
                        ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                        ControlFlow::Continue | ControlFlow::None => {}
                    }
                }
                Ok(ControlFlow::None)
            }

            Stmt::Break(_) => Ok(ControlFlow::Break),
            Stmt::Continue(_) => Ok(ControlFlow::Continue),

            Stmt::Import { module: _, span: _ } => {
                // Import module (e.g. import <io>)
                Ok(ControlFlow::None)
            }

            Stmt::Print { args, .. } => {
                let mut printed_parts = Vec::new();
                for arg in args {
                    let val = self.eval_expr(arg, Rc::clone(&env))?;
                    printed_parts.push(format!("{}", val));
                }
                println!("{}", printed_parts.join(" "));
                Ok(ControlFlow::None)
            }

            Stmt::Return { value, .. } => {
                let ret_val = if let Some(v_expr) = value {
                    self.eval_expr(v_expr, Rc::clone(&env))?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(ret_val))
            }

            Stmt::IndexAssign {
                target,
                index,
                value,
                span,
            } => {
                if let Expr::Variable(name, var_span) = &**target {
                    let var = env.borrow().get(name);
                    if let Some(v) = var {
                        if v.is_const {
                            return Err(RuntimeError {
                                message: format!("Cannot modify elements of constant array '{}' declared with 'let'", name),
                                span: *var_span,
                                hint: Some("Declare the array with 'var' instead of 'let' if it needs to be mutable".to_string()),
                            });
                        }

                        if let Value::Array(mut items) = v.value {
                            let idx_val = self.eval_expr(index, Rc::clone(&env))?;
                            let new_val = self.eval_expr(value, Rc::clone(&env))?;

                            if let Value::Int(idx) = idx_val {
                                if idx < 0 || (idx as usize) >= items.len() {
                                    return Err(RuntimeError {
                                        message: format!("Index {} is out of bounds for array of length {}", idx, items.len()),
                                        span: *span,
                                        hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                    });
                                }
                                items[idx as usize] = new_val;
                                if let Err((err, hint)) = env.borrow_mut().assign(name, Value::Array(items)) {
                                    return Err(RuntimeError {
                                        message: err,
                                        span: *span,
                                        hint,
                                    });
                                }
                                return Ok(ControlFlow::None);
                            } else {
                                return Err(RuntimeError {
                                    message: format!("Array index must be an integer, got '{}'", idx_val.get_type()),
                                    span: *span,
                                    hint: Some("Use an integer index (e.g. tablica.[0])".to_string()),
                                });
                            }
                        } else if let Value::Map(ref map_cell) = v.value {
                            let key_val = self.eval_expr(index, Rc::clone(&env))?;
                            let new_val = self.eval_expr(value, Rc::clone(&env))?;
                            let mut entries = map_cell.borrow_mut();
                            if let Some((_, val)) = entries.iter_mut().find(|(k, _)| k == &key_val) {
                                *val = new_val;
                            } else {
                                entries.push((key_val, new_val));
                            }
                            return Ok(ControlFlow::None);
                        } else {
                            return Err(RuntimeError {
                                message: format!("Cannot index-assign to variable '{}' of type '{}'", name, v.value.get_type()),
                                span: *span,
                                hint: Some("Index assignment can only be performed on arrays or maps".to_string()),
                            });
                        }
                    } else {
                        return Err(RuntimeError {
                            message: format!("Undefined variable '{}'", name),
                            span: *var_span,
                            hint: None,
                        });
                    }
                }
                Ok(ControlFlow::None)
            }

            Stmt::Block(stmts, _) => {
                let local_env = Environment::new_with_parent(Rc::clone(&env));
                self.execute_block(stmts, local_env)
            }

            Stmt::Expr(expr) => {
                self.eval_expr(expr, env)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> Result<ControlFlow, RuntimeError> {
        for stmt in stmts {
            let res = self.execute_stmt(stmt, Rc::clone(&env))?;
            match res {
                ControlFlow::Return(_) | ControlFlow::Break | ControlFlow::Continue => return Ok(res),
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
    }

    fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Environment>>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val, _) => Ok(val.clone()),

            Expr::Variable(name, span) => {
                if let Some(var) = env.borrow().get(name) {
                    Ok(var.value)
                } else if name == "math::pi" || name == "math::PI" || name == "math::Pi" || name == "pi" || name == "PI" {
                    Ok(Value::Float(std::f64::consts::PI))
                } else if name == "math::e" || name == "math::E" || name == "e" || name == "E" {
                    Ok(Value::Float(std::f64::consts::E))
                } else if name == "math::tau" || name == "math::TAU" || name == "math::Tau" {
                    Ok(Value::Float(std::f64::consts::TAU))
                } else if let Some((enum_name, variant_name)) = name.split_once("::") {
                    if let Some(variants) = self.enum_defs.get(enum_name) {
                        if let Some((vname, val)) = variants.iter().find(|(v, _)| v == variant_name) {
                            Ok(Value::Enum {
                                enum_name: enum_name.to_string(),
                                variant: vname.clone(),
                                value: val.clone().map(Box::new),
                            })
                        } else {
                            let available = variants.iter().map(|(v, _)| v.as_str()).collect::<Vec<_>>().join(", ");
                            Err(RuntimeError {
                                message: format!("Enum '{}' has no variant named '{}'", enum_name, variant_name),
                                span: *span,
                                hint: Some(format!("Available variants for '{}': {}", enum_name, available)),
                            })
                        }
                    } else {
                        Err(RuntimeError {
                            message: format!("Undefined enum or module '{}'", enum_name),
                            span: *span,
                            hint: Some(format!("Define the enum before using it: enum {} {{ ... }}", enum_name)),
                        })
                    }
                } else {
                    Err(RuntimeError {
                        message: format!("Undefined variable '{}'", name),
                        span: *span,
                        hint: Some(format!(
                            "Declare the variable before using it: var int32: {} = ...;",
                            name
                        )),
                    })
                }
            }

            Expr::ArrayLiteral(elements, _) => {
                let mut values = Vec::new();
                for el in elements {
                    values.push(self.eval_expr(el, Rc::clone(&env))?);
                }
                Ok(Value::Array(values))
            }

            Expr::TupleLiteral(items, _) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item, Rc::clone(&env))?);
                }
                Ok(Value::Tuple(vals))
            }

            Expr::MapLiteral { entries, .. } => {
                let mut map_entries = Vec::new();
                for (k_expr, v_expr) in entries {
                    let k_val = self.eval_expr(k_expr, Rc::clone(&env))?;
                    let v_val = self.eval_expr(v_expr, Rc::clone(&env))?;
                    if let Some((_, existing_v)) = map_entries.iter_mut().find(|(k, _)| k == &k_val) {
                        *existing_v = v_val;
                    } else {
                        map_entries.push((k_val, v_val));
                    }
                }
                Ok(Value::Map(Rc::new(RefCell::new(map_entries))))
            }

            Expr::StructLiteral {
                struct_name,
                fields,
                span,
            } => {
                let def_fields = match self.struct_defs.get(struct_name) {
                    Some(f) => f.clone(),
                    None => {
                        return Err(RuntimeError {
                            message: format!("Undefined struct '{}'", struct_name),
                            span: *span,
                            hint: Some(format!("Define the struct before instantiating it: struct {} {{ ... }}", struct_name)),
                        });
                    }
                };

                let mut field_map = std::collections::HashMap::new();
                for (fname, fexpr) in fields {
                    let mut fval = self.eval_expr(fexpr, Rc::clone(&env))?;
                    let expected_type = def_fields.iter().find(|(name, _)| name == fname).map(|(_, t)| t);
                    if let Some(exp_t) = expected_type {
                        if let Value::Int(n) = fval {
                            if matches!(exp_t, SherType::Float8 | SherType::Float16 | SherType::Float32 | SherType::Float64) {
                                fval = Value::Float(n as f64);
                            }
                        }
                        if !fval.matches_type(exp_t) {
                            return Err(RuntimeError {
                                message: format!("Type mismatch for field '{}.{}': expected '{}', got value '{}' of type '{}'", struct_name, fname, exp_t, fval, fval.get_type()),
                                span: *span,
                                hint: Some(format!("Provide a value of type '{}' for field '{}'", exp_t, fname)),
                            });
                        }
                    } else {
                        return Err(RuntimeError {
                            message: format!("Struct '{}' has no field named '{}'", struct_name, fname),
                            span: *span,
                            hint: None,
                        });
                    }
                    field_map.insert(fname.clone(), fval);
                }

                for (dfname, _) in &def_fields {
                    if !field_map.contains_key(dfname) {
                        return Err(RuntimeError {
                            message: format!("Missing field '{}' in instantiation of struct '{}'", dfname, struct_name),
                            span: *span,
                            hint: Some(format!("Provide all required fields when creating struct '{}'", struct_name)),
                        });
                    }
                }

                Ok(Value::Struct {
                    struct_name: struct_name.clone(),
                    fields: field_map,
                })
            }

            Expr::Index { target, index, span } => {
                if let Expr::Range { start, end, span: rspan } = &**index {
                    let target_val = self.eval_expr(target, Rc::clone(&env))?;
                    let s_val = self.eval_expr(start, Rc::clone(&env))?;
                    let e_val = self.eval_expr(end, Rc::clone(&env))?;
                    if let (Value::Int(s_idx), Value::Int(e_idx)) = (s_val, e_val) {
                        match target_val {
                            Value::Array(items) => {
                                if s_idx < 0 || (s_idx as usize) >= items.len() || e_idx < s_idx || (e_idx as usize) >= items.len() {
                                    return Err(RuntimeError {
                                        message: format!("Slice index {}:{} is out of bounds for array of length {}", s_idx, e_idx, items.len()),
                                        span: *rspan,
                                        hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                    });
                                }
                                let count = (e_idx - s_idx + 1) as usize;
                                let slice = items[(s_idx as usize)..(s_idx as usize + count)].to_vec();
                                return Ok(Value::Array(slice));
                            }
                            Value::String(s) => {
                                let chars: Vec<char> = s.chars().collect();
                                if s_idx < 0 || (s_idx as usize) >= chars.len() || e_idx < s_idx || (e_idx as usize) >= chars.len() {
                                    return Err(RuntimeError {
                                        message: format!("Slice index {}:{} is out of bounds for string of length {}", s_idx, e_idx, chars.len()),
                                        span: *rspan,
                                        hint: Some(format!("Valid index range is 0 to {}", chars.len().saturating_sub(1))),
                                    });
                                }
                                let count = (e_idx - s_idx + 1) as usize;
                                let sub: String = chars[(s_idx as usize)..(s_idx as usize + count)].iter().collect();
                                return Ok(Value::String(sub));
                            }
                            _ => {}
                        }
                    }
                }

                let target_val = self.eval_expr(target, Rc::clone(&env))?;
                let index_val = self.eval_expr(index, Rc::clone(&env))?;

                if let Value::Map(ref map_cell) = target_val {
                    let entries = map_cell.borrow();
                    if let Some((_, val)) = entries.iter().find(|(k, _)| k == &index_val) {
                        return Ok(val.clone());
                    } else {
                        return Err(RuntimeError {
                            message: format!("Key '{}' not found in map", index_val),
                            span: *span,
                            hint: Some("Check if the key exists using .has(key) before accessing it".to_string()),
                        });
                    }
                }

                if let Value::Int(idx) = index_val {
                    match target_val {
                        Value::Array(items) => {
                            if idx < 0 || (idx as usize) >= items.len() {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Index {} is out of bounds for array of length {}",
                                        idx,
                                        items.len()
                                    ),
                                    span: *span,
                                    hint: Some(format!(
                                        "Valid index range is 0 to {}",
                                        items.len().saturating_sub(1)
                                    )),
                                });
                            }
                            Ok(items[idx as usize].clone())
                        }
                        Value::Tuple(items) => {
                            if idx < 0 || (idx as usize) >= items.len() {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Index {} is out of bounds for tuple of size {}",
                                        idx,
                                        items.len()
                                    ),
                                    span: *span,
                                    hint: Some(format!(
                                        "Valid index range is 0 to {}",
                                        items.len().saturating_sub(1)
                                    )),
                                });
                            }
                            Ok(items[idx as usize].clone())
                        }
                        Value::String(s) => {
                            let chars: Vec<char> = s.chars().collect();
                            if idx < 0 || (idx as usize) >= chars.len() {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Index {} is out of bounds for string of length {}",
                                        idx,
                                        chars.len()
                                    ),
                                    span: *span,
                                    hint: Some(format!(
                                        "Valid index range is 0 to {}",
                                        chars.len().saturating_sub(1)
                                    )),
                                });
                            }
                            Ok(Value::Char(chars[idx as usize]))
                        }
                        _ => Err(RuntimeError {
                            message: format!(
                                "Cannot index value of type '{}'",
                                target_val.get_type()
                            ),
                            span: *span,
                            hint: Some("Indexing can only be performed on arrays, tuples, maps, or strings".to_string()),
                        }),
                    }
                } else {
                    Err(RuntimeError {
                        message: format!(
                            "Index must be an integer, got '{}'",
                            index_val.get_type()
                        ),
                        span: *span,
                        hint: Some("Use an integer index for arrays, strings, or tuples".to_string()),
                    })
                }
            }

            Expr::FieldAccess {
                target,
                field,
                span,
            } => {
                if let Expr::Variable(ref base_name, _) = **target {
                    if base_name == "math" {
                        if field.eq_ignore_ascii_case("pi") {
                            return Ok(Value::Float(std::f64::consts::PI));
                        } else if field.eq_ignore_ascii_case("e") {
                            return Ok(Value::Float(std::f64::consts::E));
                        } else if field.eq_ignore_ascii_case("tau") {
                            return Ok(Value::Float(std::f64::consts::TAU));
                        }
                    }
                    if let Some(variants) = self.enum_defs.get(base_name) {
                        if let Some((vname, val)) = variants.iter().find(|(v, _)| v == field) {
                            return Ok(Value::Enum {
                                enum_name: base_name.clone(),
                                variant: vname.clone(),
                                value: val.clone().map(Box::new),
                            });
                        } else {
                            let available = variants.iter().map(|(v, _)| v.as_str()).collect::<Vec<_>>().join(", ");
                            return Err(RuntimeError {
                                message: format!("Enum '{}' has no variant named '{}'", base_name, field),
                                span: *span,
                                hint: Some(format!("Available variants for '{}': {}", base_name, available)),
                            });
                        }
                    }
                }

                let target_val = self.eval_expr(target, Rc::clone(&env))?;
                if let Value::Struct { ref struct_name, ref fields } = target_val {
                    if let Some(val) = fields.get(field) {
                        Ok(val.clone())
                    } else {
                        Err(RuntimeError {
                            message: format!("Struct '{}' has no field named '{}'", struct_name, field),
                            span: *span,
                            hint: None,
                        })
                    }
                } else {
                    Err(RuntimeError {
                        message: format!("Cannot access field '{}' on non-struct type '{}'", field, target_val.get_type()),
                        span: *span,
                        hint: Some("Fields can only be accessed on struct instances (e.g. user.imie)".to_string()),
                    })
                }
            }

            Expr::FieldAssign {
                target,
                field,
                value,
                span,
            } => {
                if let Expr::Variable(ref var_name, ref var_span) = **target {
                    let var = env.borrow().get(var_name);
                    if let Some(v) = var {
                        if v.is_const {
                            return Err(RuntimeError {
                                message: format!("Cannot modify field '{}' of constant struct '{}' declared with 'let'", field, var_name),
                                span: *var_span,
                                hint: Some("Declare the struct variable with 'var' instead of 'let' if it needs to be mutable".to_string()),
                            });
                        }
                        if let Value::Struct { struct_name, mut fields } = v.value {
                            if !fields.contains_key(field) {
                                return Err(RuntimeError {
                                    message: format!("Struct '{}' has no field named '{}'", struct_name, field),
                                    span: *span,
                                    hint: None,
                                });
                            }
                            let mut val = self.eval_expr(value, Rc::clone(&env))?;
                            if let Some(def_fields) = self.struct_defs.get(&struct_name) {
                                if let Some((_, exp_t)) = def_fields.iter().find(|(name, _)| name == field) {
                                    if let Value::Int(n) = val {
                                        if matches!(exp_t, SherType::Float8 | SherType::Float16 | SherType::Float32 | SherType::Float64) {
                                            val = Value::Float(n as f64);
                                        }
                                    }
                                    if !val.matches_type(exp_t) {
                                        return Err(RuntimeError {
                                            message: format!("Type mismatch: cannot assign value of type '{}' to field '{}.{}' of type '{}'", val.get_type(), struct_name, field, exp_t),
                                            span: *span,
                                            hint: Some(format!("Provide a value of type '{}'", exp_t)),
                                        });
                                    }
                                }
                            }
                            fields.insert(field.clone(), val.clone());
                            if let Err((err, hint)) = env.borrow_mut().assign(var_name, Value::Struct { struct_name, fields }) {
                                return Err(RuntimeError {
                                    message: err,
                                    span: *span,
                                    hint,
                                });
                            }
                            return Ok(val);
                        } else {
                            return Err(RuntimeError {
                                message: format!("Variable '{}' is not a struct", var_name),
                                span: *var_span,
                                hint: None,
                            });
                        }
                    } else {
                        return Err(RuntimeError {
                            message: format!("Undefined variable '{}'", var_name),
                            span: *var_span,
                            hint: None,
                        });
                    }
                }
                Err(RuntimeError {
                    message: "Invalid field assignment target".to_string(),
                    span: *span,
                    hint: Some("Assign to a struct variable field (e.g. user.imie = \"Anna\";)".to_string()),
                })
            }

            Expr::Increment { target, is_prefix, span } => {
                if let Expr::Variable(name, var_span) = &**target {
                    let var_opt = env.borrow().get(name);
                    if let Some(var) = var_opt {
                        if var.is_const {
                            return Err(RuntimeError {
                                message: format!("Cannot increment constant variable '{}' declared with 'let'", name),
                                span: *var_span,
                                hint: Some("Declare the variable with 'var' instead of 'let' if it needs to be mutable".to_string()),
                            });
                        }
                        let (old_val, new_val) = match var.value {
                            Value::Int(n) => (Value::Int(n), Value::Int(n + 1)),
                            Value::Float(f) => (Value::Float(f), Value::Float(f + 1.0)),
                            _ => return Err(RuntimeError {
                                message: format!("Cannot increment variable of type '{}'", var.value.get_type()),
                                span: *span,
                                hint: Some("Increment '++' can only be applied to numeric variables".to_string()),
                            }),
                        };
                        if let Err((err, hint)) = env.borrow_mut().assign(name, new_val.clone()) {
                            return Err(RuntimeError {
                                message: err,
                                span: *span,
                                hint,
                            });
                        }
                        if *is_prefix { Ok(new_val) } else { Ok(old_val) }
                    } else {
                        Err(RuntimeError {
                            message: format!("Undefined variable '{}'", name),
                            span: *var_span,
                            hint: None,
                        })
                    }
                } else {
                    Err(RuntimeError {
                        message: "Invalid target for increment '++'".to_string(),
                        span: *span,
                        hint: Some("Increment operator can only be applied to variables (e.g. i++)".to_string()),
                    })
                }
            }

            Expr::Decrement { target, is_prefix, span } => {
                if let Expr::Variable(name, var_span) = &**target {
                    let var_opt = env.borrow().get(name);
                    if let Some(var) = var_opt {
                        if var.is_const {
                            return Err(RuntimeError {
                                message: format!("Cannot decrement constant variable '{}' declared with 'let'", name),
                                span: *var_span,
                                hint: Some("Declare the variable with 'var' instead of 'let' if it needs to be mutable".to_string()),
                            });
                        }
                        let (old_val, new_val) = match var.value {
                            Value::Int(n) => (Value::Int(n), Value::Int(n - 1)),
                            Value::Float(f) => (Value::Float(f), Value::Float(f - 1.0)),
                            _ => return Err(RuntimeError {
                                message: format!("Cannot decrement variable of type '{}'", var.value.get_type()),
                                span: *span,
                                hint: Some("Decrement '--' can only be applied to numeric variables".to_string()),
                            }),
                        };
                        if let Err((err, hint)) = env.borrow_mut().assign(name, new_val.clone()) {
                            return Err(RuntimeError {
                                message: err,
                                span: *span,
                                hint,
                            });
                        }
                        if *is_prefix { Ok(new_val) } else { Ok(old_val) }
                    } else {
                        Err(RuntimeError {
                            message: format!("Undefined variable '{}'", name),
                            span: *var_span,
                            hint: None,
                        })
                    }
                } else {
                    Err(RuntimeError {
                        message: "Invalid target for decrement '--'".to_string(),
                        span: *span,
                        hint: Some("Decrement operator can only be applied to variables (e.g. i--)".to_string()),
                    })
                }
            }

            Expr::CompoundAssign { name, op, value, span } => {
                let right_val = self.eval_expr(value, Rc::clone(&env))?;
                let var_opt = env.borrow().get(name);
                if let Some(var) = var_opt {
                    if var.is_const {
                        return Err(RuntimeError {
                            message: format!("Cannot modify constant variable '{}' declared with 'let'", name),
                            span: *span,
                            hint: Some("Declare the variable with 'var' instead of 'let' if it needs to be mutable".to_string()),
                        });
                    }

                    let new_val = match (op, var.value, right_val) {
                        (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                        (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                        (BinaryOp::Add, Value::String(a), b) => Value::String(format!("{}{}", a, b)),
                        (BinaryOp::Subtract, Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        (BinaryOp::Subtract, Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        (BinaryOp::Multiply, Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                        (BinaryOp::Multiply, Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        (BinaryOp::Divide, Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(RuntimeError {
                                    message: "Division by zero".to_string(),
                                    span: *span,
                                    hint: None,
                                });
                            }
                            Value::Int(a / b)
                        }
                        (BinaryOp::Divide, Value::Float(a), Value::Float(b)) => {
                            if b == 0.0 {
                                return Err(RuntimeError {
                                    message: "Division by zero".to_string(),
                                    span: *span,
                                    hint: None,
                                });
                            }
                            Value::Float(a / b)
                        }
                        (BinaryOp::Modulo, Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(RuntimeError {
                                    message: "Modulo by zero".to_string(),
                                    span: *span,
                                    hint: None,
                                });
                            }
                            Value::Int(a % b)
                        }
                        _ => {
                            return Err(RuntimeError {
                                message: "Unsupported types for compound assignment".to_string(),
                                span: *span,
                                hint: None,
                            });
                        }
                    };

                    if let Err((err, hint)) = env.borrow_mut().assign(name, new_val.clone()) {
                        return Err(RuntimeError {
                            message: err,
                            span: *span,
                            hint,
                        });
                    }
                    Ok(new_val)
                } else {
                    Err(RuntimeError {
                        message: format!("Undefined variable '{}'", name),
                        span: *span,
                        hint: None,
                    })
                }
            }

            Expr::Assign { name, value, span } => {
                let val = self.eval_expr(value, Rc::clone(&env))?;
                if let Err((err, hint)) = env.borrow_mut().assign(name, val.clone()) {
                    return Err(RuntimeError {
                        message: err,
                        span: *span,
                        hint,
                    });
                }
                Ok(val)
            }

            Expr::Unary { op, expr, span } => {
                let val = self.eval_expr(expr, env)?;
                match op {
                    UnaryOp::Negate => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(RuntimeError {
                            message: format!("Cannot negate value of type '{}'", val.get_type()),
                            span: *span,
                            hint: Some("Unary '-' operator can only be applied to numbers".to_string()),
                        }),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                }
            }

            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if *op == BinaryOp::And {
                    let left_val = self.eval_expr(left, Rc::clone(&env))?;
                    if !left_val.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                    let right_val = self.eval_expr(right, env)?;
                    return Ok(Value::Bool(right_val.is_truthy()));
                }

                if *op == BinaryOp::Or {
                    let left_val = self.eval_expr(left, Rc::clone(&env))?;
                    if left_val.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                    let right_val = self.eval_expr(right, env)?;
                    return Ok(Value::Bool(right_val.is_truthy()));
                }

                let left_val = self.eval_expr(left, Rc::clone(&env))?;
                let right_val = self.eval_expr(right, env)?;

                match op {
                    BinaryOp::Add => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
                        (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b))),
                        (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::String(format!("{}{}", a, b))),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Unsupported operand types for '+': '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("The '+' operator supports numbers and string concatenation".to_string()),
                        }),
                    },
                    BinaryOp::Subtract => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Unsupported operand types for '-': '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("The '-' operator requires numeric operands".to_string()),
                        }),
                    },
                    BinaryOp::Multiply => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Unsupported operand types for '*': '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("The '*' operator requires numeric operands".to_string()),
                        }),
                    },
                    BinaryOp::Divide => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(RuntimeError {
                                    message: "Division by zero".to_string(),
                                    span: *span,
                                    hint: Some("Ensure the divisor is non-zero".to_string()),
                                });
                            }
                            Ok(Value::Int(a / b))
                        }
                        (Value::Float(a), Value::Float(b)) => {
                            if b == 0.0 {
                                return Err(RuntimeError {
                                    message: "Division by zero".to_string(),
                                    span: *span,
                                    hint: Some("Ensure the divisor is non-zero".to_string()),
                                });
                            }
                            Ok(Value::Float(a / b))
                        }
                        (Value::Int(a), Value::Float(b)) => {
                            if b == 0.0 {
                                return Err(RuntimeError {
                                    message: "Division by zero".to_string(),
                                    span: *span,
                                    hint: Some("Ensure the divisor is non-zero".to_string()),
                                });
                            }
                            Ok(Value::Float(a as f64 / b))
                        }
                        (Value::Float(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(RuntimeError {
                                    message: "Division by zero".to_string(),
                                    span: *span,
                                    hint: Some("Ensure the divisor is non-zero".to_string()),
                                });
                            }
                            Ok(Value::Float(a / b as f64))
                        }
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Unsupported operand types for '/': '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("The '/' operator requires numeric operands".to_string()),
                        }),
                    },
                    BinaryOp::Modulo => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(RuntimeError {
                                    message: "Modulo by zero".to_string(),
                                    span: *span,
                                    hint: Some("Ensure the modulo divisor is non-zero".to_string()),
                                });
                            }
                            Ok(Value::Int(a % b))
                        }
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Unsupported operand types for '%': '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("The '%' operator requires integer operands".to_string()),
                        }),
                    },
                    BinaryOp::Equal => {
                        if left_val == right_val {
                            Ok(Value::Bool(true))
                        } else {
                            match (&left_val, &right_val) {
                                (Value::Enum { value: Some(v), .. }, other) | (other, Value::Enum { value: Some(v), .. }) => {
                                    Ok(Value::Bool(**v == *other))
                                }
                                _ => Ok(Value::Bool(false)),
                            }
                        }
                    }
                    BinaryOp::NotEqual => {
                        if left_val == right_val {
                            Ok(Value::Bool(false))
                        } else {
                            match (&left_val, &right_val) {
                                (Value::Enum { value: Some(v), .. }, other) | (other, Value::Enum { value: Some(v), .. }) => {
                                    Ok(Value::Bool(**v != *other))
                                }
                                _ => Ok(Value::Bool(true)),
                            }
                        }
                    }
                    BinaryOp::Greater => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) > b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a > (b as f64))),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a > b)),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Cannot compare '>' between '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("Comparison operators can only be used on numbers or chars".to_string()),
                        }),
                    },
                    BinaryOp::GreaterEq => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) >= b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a >= (b as f64))),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a >= b)),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Cannot compare '>=' between '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("Comparison operators can only be used on numbers or chars".to_string()),
                        }),
                    },
                    BinaryOp::Less => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) < b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a < (b as f64))),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a < b)),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Cannot compare '<' between '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("Comparison operators can only be used on numbers or chars".to_string()),
                        }),
                    },
                    BinaryOp::LessEq => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) <= b)),
                        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a <= (b as f64))),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a <= b)),
                        (a, b) => Err(RuntimeError {
                            message: format!(
                                "Cannot compare '<=' between '{}' and '{}'",
                                a.get_type(),
                                b.get_type()
                            ),
                            span: *span,
                            hint: Some("Comparison operators can only be used on numbers or chars".to_string()),
                        }),
                    },
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                }
            }

            Expr::MethodCall {
                target,
                method,
                args,
                span,
            } => {
                if let Expr::Variable(name, var_span) = &**target {
                    let var_opt = env.borrow().get(name);
                    if let Some(var) = var_opt {
                        match method.as_str() {
                            "add" | "push" => {
                                if var.is_const {
                                    return Err(RuntimeError {
                                        message: format!("Cannot modify constant array '{}' declared with 'let'", name),
                                        span: *var_span,
                                        hint: Some("Declare the array with 'var' instead of 'let' if it needs to be mutable".to_string()),
                                    });
                                }
                                if args.len() != 1 {
                                    return Err(RuntimeError {
                                        message: format!("Method '{}.add' expects 1 argument, got {}", name, args.len()),
                                        span: *span,
                                        hint: Some(format!("Usage: {}.add(element);", name)),
                                    });
                                }
                                if let Value::Array(mut items) = var.value {
                                    let mut new_elem = self.eval_expr(&args[0], Rc::clone(&env))?;
                                    // If array has a specific inner type and new_elem is int/float promotion
                                    if let SherType::Array(ref inner_t) = var.var_type {
                                        if let Value::Int(n) = new_elem {
                                            if matches!(**inner_t, SherType::Float8 | SherType::Float16 | SherType::Float32 | SherType::Float64) {
                                                new_elem = Value::Float(n as f64);
                                            }
                                        }
                                        if !new_elem.matches_type(inner_t) {
                                            return Err(RuntimeError {
                                                message: format!("Type mismatch: cannot add element of type '{}' to array of type '{}'", new_elem.get_type(), var.var_type),
                                                span: *span,
                                                hint: Some(format!("Provide an element of type '{}'", inner_t)),
                                            });
                                        }
                                    }
                                    items.push(new_elem);
                                    if let Err((err, hint)) = env.borrow_mut().assign(name, Value::Array(items)) {
                                        return Err(RuntimeError {
                                            message: err,
                                            span: *span,
                                            hint,
                                        });
                                    }
                                    return Ok(Value::Null);
                                } else if let Value::Map(ref map_cell) = var.value {
                                    if args.len() != 2 {
                                        return Err(RuntimeError {
                                            message: format!("Method '{}.add' on map expects 2 arguments (key, value), got {}", name, args.len()),
                                            span: *span,
                                            hint: Some(format!("Usage: {}.add(key, value);", name)),
                                        });
                                    }
                                    let k_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                                    let v_val = self.eval_expr(&args[1], Rc::clone(&env))?;
                                    let mut entries = map_cell.borrow_mut();
                                    if let Some((_, val)) = entries.iter_mut().find(|(k, _)| k == &k_val) {
                                        *val = v_val;
                                    } else {
                                        entries.push((k_val, v_val));
                                    }
                                    return Ok(Value::Null);
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Method 'add' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: Some("Method 'add' can only be called on arrays or maps".to_string()),
                                    });
                                }
                            }
                            "remove" | "pop" => {
                                if var.is_const {
                                    return Err(RuntimeError {
                                        message: format!("Cannot modify constant array '{}' declared with 'let'", name),
                                        span: *var_span,
                                        hint: Some("Declare the array with 'var' instead of 'let' if it needs to be mutable".to_string()),
                                    });
                                }
                                if let Value::Array(mut items) = var.value {
                                    if items.is_empty() {
                                        return Err(RuntimeError {
                                            message: format!("Cannot remove element from empty array '{}'", name),
                                            span: *span,
                                            hint: None,
                                        });
                                    }

                                    let removed = if args.is_empty() {
                                        items.pop().unwrap()
                                    } else if args.len() == 1 {
                                        if let Expr::Range { start, end, span: rspan } = &args[0] {
                                            let s_val = self.eval_expr(start, Rc::clone(&env))?;
                                            let e_val = self.eval_expr(end, Rc::clone(&env))?;
                                            if let (Value::Int(s_idx), Value::Int(e_idx)) = (s_val, e_val) {
                                                if s_idx < 0 || (s_idx as usize) >= items.len() || e_idx < s_idx || (e_idx as usize) >= items.len() {
                                                    return Err(RuntimeError {
                                                        message: format!("Range {}:{} is out of bounds for array of length {}", s_idx, e_idx, items.len()),
                                                        span: *rspan,
                                                        hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                                    });
                                                }
                                                let count = (e_idx - s_idx + 1) as usize;
                                                let removed_slice: Vec<Value> = items.drain((s_idx as usize)..(s_idx as usize + count)).collect();
                                                Value::Array(removed_slice)
                                            } else {
                                                return Err(RuntimeError {
                                                    message: "Range bounds must be integers (e.g. 0:2)".to_string(),
                                                    span: *rspan,
                                                    hint: None,
                                                });
                                            }
                                        } else {
                                            let idx_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                                            if let Value::Int(idx) = idx_val {
                                                if idx < 0 || (idx as usize) >= items.len() {
                                                    return Err(RuntimeError {
                                                        message: format!("Index {} is out of bounds for array of length {}", idx, items.len()),
                                                        span: *span,
                                                        hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                                    });
                                                }
                                                items.remove(idx as usize)
                                            } else {
                                                return Err(RuntimeError {
                                                    message: format!("Remove index must be an integer, got '{}'", idx_val.get_type()),
                                                    span: *span,
                                                    hint: Some(format!("Usage: {}.remove() or {}.remove(index) or {}.remove(0:2)", name, name, name)),
                                                });
                                            }
                                        }
                                    } else {
                                        // Multiple arguments: array.remove(0, 1, 2)
                                        let mut indices_to_remove = Vec::new();
                                        for arg in args {
                                            let idx_val = self.eval_expr(arg, Rc::clone(&env))?;
                                            if let Value::Int(idx) = idx_val {
                                                if idx < 0 || (idx as usize) >= items.len() {
                                                    return Err(RuntimeError {
                                                        message: format!("Index {} is out of bounds for array of length {}", idx, items.len()),
                                                        span: *span,
                                                        hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                                    });
                                                }
                                                indices_to_remove.push(idx as usize);
                                            } else {
                                                return Err(RuntimeError {
                                                    message: format!("All remove arguments must be integers, got '{}'", idx_val.get_type()),
                                                    span: *span,
                                                    hint: Some(format!("Usage: {}.remove(0, 1, 2);", name)),
                                                });
                                            }
                                        }

                                        indices_to_remove.sort_unstable();
                                        indices_to_remove.dedup();
                                        indices_to_remove.reverse();

                                        let mut removed_list = Vec::new();
                                        for idx in indices_to_remove {
                                            removed_list.push(items.remove(idx));
                                        }
                                        removed_list.reverse();
                                        Value::Array(removed_list)
                                    };

                                    if let Err((err, hint)) = env.borrow_mut().assign(name, Value::Array(items)) {
                                        return Err(RuntimeError {
                                            message: err,
                                            span: *span,
                                            hint,
                                        });
                                    }
                                    return Ok(removed);
                                } else if let Value::Map(ref map_cell) = var.value {
                                    if args.len() != 1 {
                                        return Err(RuntimeError {
                                            message: format!("Method '{}.remove' on map expects 1 argument (key), got {}", name, args.len()),
                                            span: *span,
                                            hint: Some(format!("Usage: {}.remove(key);", name)),
                                        });
                                    }
                                    let k_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                                    let mut entries = map_cell.borrow_mut();
                                    if let Some(pos) = entries.iter().position(|(k, _)| k == &k_val) {
                                        let (_, removed_v) = entries.remove(pos);
                                        return Ok(removed_v);
                                    } else {
                                        return Ok(Value::Null);
                                    }
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Method 'remove' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: Some("Method 'remove' can only be called on arrays or maps".to_string()),
                                    });
                                }
                            }
                            "len" | "size" | "count" => {
                                match var.value {
                                    Value::Array(items) => return Ok(Value::Int(items.len() as i64)),
                                    Value::Tuple(items) => return Ok(Value::Int(items.len() as i64)),
                                    Value::String(s) => return Ok(Value::Int(s.chars().count() as i64)),
                                    Value::Map(map_cell) => return Ok(Value::Int(map_cell.borrow().len() as i64)),
                                    _ => return Err(RuntimeError {
                                        message: format!("Method 'len' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: None,
                                    }),
                                }
                            }
                            "clear" => {
                                if var.is_const {
                                    return Err(RuntimeError {
                                        message: format!("Cannot modify constant array '{}' declared with 'let'", name),
                                        span: *var_span,
                                        hint: Some("Declare the array with 'var' instead of 'let' if it needs to be mutable".to_string()),
                                    });
                                }
                                if let Value::Array(mut items) = var.value {
                                    items.clear();
                                    if let Err((err, hint)) = env.borrow_mut().assign(name, Value::Array(items)) {
                                        return Err(RuntimeError {
                                            message: err,
                                            span: *span,
                                            hint,
                                        });
                                    }
                                    return Ok(Value::Null);
                                } else if let Value::Map(ref map_cell) = var.value {
                                    map_cell.borrow_mut().clear();
                                    return Ok(Value::Null);
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Method 'clear' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: Some("Method 'clear' can only be called on arrays or maps".to_string()),
                                    });
                                }
                            }
                            "has" | "contains" => {
                                if args.len() != 1 {
                                    return Err(RuntimeError {
                                        message: format!("Method '{}.has' expects 1 argument, got {}", name, args.len()),
                                        span: *span,
                                        hint: Some(format!("Usage: {}.has(key);", name)),
                                    });
                                }
                                let search_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                                match var.value {
                                    Value::Map(ref map_cell) => {
                                        return Ok(Value::Bool(map_cell.borrow().iter().any(|(k, _)| k == &search_val)));
                                    }
                                    Value::Array(ref items) => {
                                        return Ok(Value::Bool(items.contains(&search_val)));
                                    }
                                    Value::String(ref s) => {
                                        let sub_str = format!("{}", search_val);
                                        return Ok(Value::Bool(s.contains(&sub_str)));
                                    }
                                    _ => return Err(RuntimeError {
                                        message: format!("Method 'has' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: Some("Method 'has' can only be called on maps, arrays, or strings".to_string()),
                                    }),
                                }
                            }
                            "keys" => {
                                if let Value::Map(ref map_cell) = var.value {
                                    let keys = map_cell.borrow().iter().map(|(k, _)| k.clone()).collect();
                                    return Ok(Value::Array(keys));
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Method 'keys' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: Some("Method 'keys' can only be called on maps".to_string()),
                                    });
                                }
                            }
                            "values" => {
                                if let Value::Map(ref map_cell) = var.value {
                                    let values = map_cell.borrow().iter().map(|(_, v)| v.clone()).collect();
                                    return Ok(Value::Array(values));
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Method 'values' is not supported on type '{}'", var.value.get_type()),
                                        span: *span,
                                        hint: Some("Method 'values' can only be called on maps".to_string()),
                                    });
                                }
                            }
                            _ => {
                                return Err(RuntimeError {
                                    message: format!("Unknown method '{}' for variable '{}'", method, name),
                                    span: *span,
                                    hint: Some("Available methods for arrays/maps: .add(), .remove(), .has(), .keys(), .values(), .len(), .clear()".to_string()),
                                });
                            }
                        }
                    } else {
                        return Err(RuntimeError {
                            message: format!("Undefined variable '{}'", name),
                            span: *var_span,
                            hint: None,
                        });
                    }
                }

                // If target is not variable directly
                let target_val = self.eval_expr(target, Rc::clone(&env))?;
                match method.as_str() {
                    "len" | "size" | "count" => match target_val {
                        Value::Array(items) => Ok(Value::Int(items.len() as i64)),
                        Value::Tuple(items) => Ok(Value::Int(items.len() as i64)),
                        Value::String(s) => Ok(Value::Int(s.chars().count() as i64)),
                        Value::Map(map_cell) => Ok(Value::Int(map_cell.borrow().len() as i64)),
                        _ => Err(RuntimeError {
                            message: format!("Method 'len' is not supported on type '{}'", target_val.get_type()),
                            span: *span,
                            hint: None,
                        }),
                    },
                    "has" | "contains" => {
                        if args.len() != 1 {
                            return Err(RuntimeError {
                                message: format!("Method 'has' expects 1 argument, got {}", args.len()),
                                span: *span,
                                hint: None,
                            });
                        }
                        let search_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                        match target_val {
                            Value::Map(ref map_cell) => Ok(Value::Bool(map_cell.borrow().iter().any(|(k, _)| k == &search_val))),
                            Value::Array(ref items) => Ok(Value::Bool(items.contains(&search_val))),
                            Value::String(ref s) => Ok(Value::Bool(s.contains(&format!("{}", search_val)))),
                            _ => Err(RuntimeError {
                                message: format!("Method 'has' is not supported on type '{}'", target_val.get_type()),
                                span: *span,
                                hint: None,
                            }),
                        }
                    }
                    "keys" => match target_val {
                        Value::Map(ref map_cell) => Ok(Value::Array(map_cell.borrow().iter().map(|(k, _)| k.clone()).collect())),
                        _ => Err(RuntimeError {
                            message: format!("Method 'keys' is not supported on type '{}'", target_val.get_type()),
                            span: *span,
                            hint: None,
                        }),
                    },
                    "values" => match target_val {
                        Value::Map(ref map_cell) => Ok(Value::Array(map_cell.borrow().iter().map(|(_, v)| v.clone()).collect())),
                        _ => Err(RuntimeError {
                            message: format!("Method 'values' is not supported on type '{}'", target_val.get_type()),
                            span: *span,
                            hint: None,
                        }),
                    },
                    _ => Err(RuntimeError {
                        message: format!("Cannot call mutating method '{}' on non-variable target", method),
                        span: *span,
                        hint: Some("Call mutating methods on a mutable variable (e.g. mapa.add(key, val))".to_string()),
                    }),
                }
            }

            Expr::Call {
                callee,
                args,
                span,
            } => {
                if callee == "__index_assign__" && args.len() == 3 {
                    if let Expr::Variable(ref name, ref var_span) = args[0] {
                        let var = env.borrow().get(name);
                        if let Some(v) = var {
                            if v.is_const {
                                return Err(RuntimeError {
                                    message: format!("Cannot modify constant '{}' declared with 'let'", name),
                                    span: *var_span,
                                    hint: Some(format!("Declare '{}' with 'var' instead of 'let' if it needs to be mutable", name)),
                                });
                            }

                            if let Value::Array(mut items) = v.value {
                                let idx_val = self.eval_expr(&args[1], Rc::clone(&env))?;
                                let new_val = self.eval_expr(&args[2], Rc::clone(&env))?;

                                if let Value::Int(idx) = idx_val {
                                    if idx < 0 || (idx as usize) >= items.len() {
                                        return Err(RuntimeError {
                                            message: format!("Index {} is out of bounds for array of length {}", idx, items.len()),
                                            span: *span,
                                            hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                        });
                                    }
                                    items[idx as usize] = new_val.clone();
                                    if let Err((err, hint)) = env.borrow_mut().assign(name, Value::Array(items)) {
                                        return Err(RuntimeError {
                                            message: err,
                                            span: *span,
                                            hint,
                                        });
                                    }
                                    return Ok(new_val);
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Array index must be an integer, got '{}'", idx_val.get_type()),
                                        span: *span,
                                        hint: Some("Use an integer index (e.g. [0])".to_string()),
                                    });
                                }
                            } else if let Value::Map(ref map_cell) = v.value {
                                let key_val = self.eval_expr(&args[1], Rc::clone(&env))?;
                                let new_val = self.eval_expr(&args[2], Rc::clone(&env))?;
                                let mut entries = map_cell.borrow_mut();
                                if let Some((_, val)) = entries.iter_mut().find(|(k, _)| k == &key_val) {
                                    *val = new_val.clone();
                                } else {
                                    entries.push((key_val, new_val.clone()));
                                }
                                return Ok(new_val);
                            } else {
                                return Err(RuntimeError {
                                    message: format!("Cannot index-assign to variable '{}' of type '{}'", name, v.value.get_type()),
                                    span: *span,
                                    hint: Some("Index assignment can only be performed on arrays or maps".to_string()),
                                });
                            }
                        } else {
                            return Err(RuntimeError {
                                message: format!("Undefined variable '{}'", name),
                                span: *var_span,
                                hint: None,
                            });
                        }
                    }
                }

                // Built-in functions
                if callee == "io::readFile" || callee == "readFile" || callee == "read_file" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (file path), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: var string: content = io::readFile(\"file.txt\");".to_string()),
                        });
                    }

                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };

                    let full_path = self.resolve_path(&raw_path);

                    match std::fs::read_to_string(&full_path) {
                        Ok(content) => return Ok(Value::String(content)),
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot read file '{}': {}", full_path.display(), e),
                                span: *span,
                                hint: Some("Make sure the file exists at the specified path".to_string()),
                            });
                        }
                    }
                }

                if callee == "io::writeFile" || callee == "writeFile" || callee == "write_file" {
                    if args.len() < 2 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' requires at least 2 arguments (file path and content)", callee),
                            span: *span,
                            hint: Some("Usage: io::writeFile(\"data.txt\", imie, wiek, ...)".to_string()),
                        });
                    }

                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };

                    let mut parts = Vec::new();
                    for arg in &args[1..] {
                        let val = self.eval_expr(arg, Rc::clone(&env))?;
                        parts.push(format!("{}", val));
                    }
                    let content = parts.join(" ");

                    let full_path = self.resolve_path(&raw_path);

                    match std::fs::write(&full_path, content) {
                        Ok(_) => return Ok(Value::Bool(true)),
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot write to file '{}': {}", full_path.display(), e),
                                span: *span,
                                hint: None,
                            });
                        }
                    }
                }

                if callee == "io::appendFile" || callee == "appendFile" || callee == "append_file" {
                    if args.len() < 2 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' requires at least 2 arguments (file path and content)", callee),
                            span: *span,
                            hint: Some("Usage: io::appendFile(\"log.txt\", \"new line\\n\")".to_string()),
                        });
                    }

                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };

                    let mut parts = Vec::new();
                    for arg in &args[1..] {
                        let val = self.eval_expr(arg, Rc::clone(&env))?;
                        parts.push(format!("{}", val));
                    }
                    let content = parts.join(" ");

                    let full_path = self.resolve_path(&raw_path);
                    use std::io::Write;
                    match std::fs::OpenOptions::new().create(true).append(true).open(&full_path) {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(content.as_bytes()) {
                                return Err(RuntimeError {
                                    message: format!("Cannot append to file '{}': {}", full_path.display(), e),
                                    span: *span,
                                    hint: None,
                                });
                            }
                            return Ok(Value::Bool(true));
                        }
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot open file '{}' for appending: {}", full_path.display(), e),
                                span: *span,
                                hint: None,
                            });
                        }
                    }
                }

                if callee == "io::fileExists" || callee == "fileExists" || callee == "file_exists" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (path), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: if (io::fileExists(\"file.txt\")) { ... }".to_string()),
                        });
                    }
                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };
                    let full_path = self.resolve_path(&raw_path);
                    return Ok(Value::Bool(full_path.exists()));
                }

                if callee == "io::isDir" || callee == "isDir" || callee == "is_dir" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (path), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: if (io::isDir(\"folder\")) { ... }".to_string()),
                        });
                    }
                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };
                    let full_path = self.resolve_path(&raw_path);
                    return Ok(Value::Bool(full_path.is_dir()));
                }

                if callee == "io::deleteFile" || callee == "deleteFile" || callee == "delete_file" || callee == "io::removeFile" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (file path), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: io::deleteFile(\"temp.txt\");".to_string()),
                        });
                    }
                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };
                    let full_path = self.resolve_path(&raw_path);
                    match std::fs::remove_file(&full_path) {
                        Ok(_) => return Ok(Value::Bool(true)),
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot delete file '{}': {}", full_path.display(), e),
                                span: *span,
                                hint: None,
                            });
                        }
                    }
                }

                if callee == "io::createDir" || callee == "createDir" || callee == "create_dir" || callee == "io::makeDir" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (dir path), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: io::createDir(\"new_folder\");".to_string()),
                        });
                    }
                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };
                    let full_path = self.resolve_path(&raw_path);
                    match std::fs::create_dir_all(&full_path) {
                        Ok(_) => return Ok(Value::Bool(true)),
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot create directory '{}': {}", full_path.display(), e),
                                span: *span,
                                hint: None,
                            });
                        }
                    }
                }

                if callee == "io::deleteDir" || callee == "deleteDir" || callee == "delete_dir" || callee == "io::removeDir" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (dir path), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: io::deleteDir(\"old_folder\");".to_string()),
                        });
                    }
                    let path_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let raw_path = match path_val {
                        Value::String(s) => s,
                        other => format!("{}", other),
                    };
                    let full_path = self.resolve_path(&raw_path);
                    match std::fs::remove_dir_all(&full_path) {
                        Ok(_) => return Ok(Value::Bool(true)),
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot delete directory '{}': {}", full_path.display(), e),
                                span: *span,
                                hint: None,
                            });
                        }
                    }
                }

                if callee == "io::listDir" || callee == "listDir" || callee == "list_dir" {
                    let raw_path = if let Some(arg) = args.first() {
                        let val = self.eval_expr(arg, Rc::clone(&env))?;
                        match val {
                            Value::String(s) => s,
                            other => format!("{}", other),
                        }
                    } else {
                        ".".to_string()
                    };
                    let full_path = self.resolve_path(&raw_path);
                    match std::fs::read_dir(&full_path) {
                        Ok(entries) => {
                            let mut list = Vec::new();
                            for entry in entries.flatten() {
                                if let Some(name) = entry.file_name().to_str() {
                                    list.push(Value::String(name.to_string()));
                                }
                            }
                            list.sort_by(|a, b| match (a, b) {
                                (Value::String(sa), Value::String(sb)) => sa.cmp(sb),
                                _ => std::cmp::Ordering::Equal,
                            });
                            return Ok(Value::Array(list));
                        }
                        Err(e) => {
                            return Err(RuntimeError {
                                message: format!("Cannot list directory '{}': {}", full_path.display(), e),
                                span: *span,
                                hint: Some("Make sure the directory exists".to_string()),
                            });
                        }
                    }
                }

                if callee == "io::clear" || callee == "io::clearScreen" || callee == "io::clear_screen" || callee == "clear" || callee == "clearScreen" {
                    use std::io::Write;
                    print!("\x1B[2J\x1B[1;1H");
                    let _ = std::io::stdout().flush();
                    return Ok(Value::Null);
                }

                // --- Math module functions ---
                if callee == "math::sqrt" || callee == "sqrt" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::sqrt(number)".to_string()),
                        });
                    }
                    let val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&val) {
                        if f < 0.0 {
                            return Err(RuntimeError {
                                message: format!("Cannot compute square root of negative number {}", f),
                                span: *span,
                                hint: Some("Provide a non-negative number to math::sqrt".to_string()),
                            });
                        }
                        return Ok(Value::Float(f.sqrt()));
                    } else {
                        return Err(RuntimeError {
                            message: format!("math::sqrt expects a number, got '{}'", val.get_type()),
                            span: *span,
                            hint: None,
                        });
                    }
                }

                if callee == "math::cbrt" || callee == "cbrt" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::cbrt(number)".to_string()),
                        });
                    }
                    let val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&val) {
                        return Ok(Value::Float(f.cbrt()));
                    } else {
                        return Err(RuntimeError {
                            message: format!("math::cbrt expects a number, got '{}'", val.get_type()),
                            span: *span,
                            hint: None,
                        });
                    }
                }

                if callee == "math::pow" || callee == "pow" {
                    if args.len() != 2 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 2 arguments (base, exponent), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::pow(base, exponent)".to_string()),
                        });
                    }
                    let base_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let exp_val = self.eval_expr(&args[1], Rc::clone(&env))?;
                    match (base_val, exp_val) {
                        (Value::Int(b), Value::Int(e)) if e >= 0 && e <= u32::MAX as i64 => {
                            return Ok(Value::Int(b.pow(e as u32)));
                        }
                        (b, e) => {
                            if let (Some(bf), Some(ef)) = (val_to_f64(&b), val_to_f64(&e)) {
                                return Ok(Value::Float(bf.powf(ef)));
                            } else {
                                return Err(RuntimeError {
                                    message: format!("math::pow requires numeric arguments, got '{}' and '{}'", b.get_type(), e.get_type()),
                                    span: *span,
                                    hint: None,
                                });
                            }
                        }
                    }
                }

                if callee == "math::abs" || callee == "abs" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::abs(number)".to_string()),
                        });
                    }
                    let val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    match val {
                        Value::Int(n) => return Ok(Value::Int(n.abs())),
                        Value::Float(f) => return Ok(Value::Float(f.abs())),
                        _ => return Err(RuntimeError {
                            message: format!("math::abs expects a number, got '{}'", val.get_type()),
                            span: *span,
                            hint: None,
                        }),
                    }
                }

                if callee == "math::floor" || callee == "floor" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::floor(number)".to_string()),
                        });
                    }
                    let val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&val) {
                        return Ok(Value::Float(f.floor()));
                    } else {
                        return Err(RuntimeError {
                            message: format!("math::floor expects a number, got '{}'", val.get_type()),
                            span: *span,
                            hint: None,
                        });
                    }
                }

                if callee == "math::ceil" || callee == "ceil" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::ceil(number)".to_string()),
                        });
                    }
                    let val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&val) {
                        return Ok(Value::Float(f.ceil()));
                    } else {
                        return Err(RuntimeError {
                            message: format!("math::ceil expects a number, got '{}'", val.get_type()),
                            span: *span,
                            hint: None,
                        });
                    }
                }

                if callee == "math::round" || callee == "round" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::round(number)".to_string()),
                        });
                    }
                    let val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&val) {
                        return Ok(Value::Float(f.round()));
                    } else {
                        return Err(RuntimeError {
                            message: format!("math::round expects a number, got '{}'", val.get_type()),
                            span: *span,
                            hint: None,
                        });
                    }
                }

                if callee == "math::min" || callee == "min" {
                    if args.len() != 2 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 2 arguments, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::min(a, b)".to_string()),
                        });
                    }
                    let a = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let b = self.eval_expr(&args[1], Rc::clone(&env))?;
                    match (&a, &b) {
                        (Value::Int(ia), Value::Int(ib)) => return Ok(Value::Int((*ia).min(*ib))),
                        _ => {
                            if let (Some(fa), Some(fb)) = (val_to_f64(&a), val_to_f64(&b)) {
                                return Ok(Value::Float(fa.min(fb)));
                            } else {
                                return Err(RuntimeError {
                                    message: format!("math::min requires numeric arguments, got '{}' and '{}'", a.get_type(), b.get_type()),
                                    span: *span,
                                    hint: None,
                                });
                            }
                        }
                    }
                }

                if callee == "math::max" || callee == "max" {
                    if args.len() != 2 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 2 arguments, got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::max(a, b)".to_string()),
                        });
                    }
                    let a = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let b = self.eval_expr(&args[1], Rc::clone(&env))?;
                    match (&a, &b) {
                        (Value::Int(ia), Value::Int(ib)) => return Ok(Value::Int((*ia).max(*ib))),
                        _ => {
                            if let (Some(fa), Some(fb)) = (val_to_f64(&a), val_to_f64(&b)) {
                                return Ok(Value::Float(fa.max(fb)));
                            } else {
                                return Err(RuntimeError {
                                    message: format!("math::max requires numeric arguments, got '{}' and '{}'", a.get_type(), b.get_type()),
                                    span: *span,
                                    hint: None,
                                });
                            }
                        }
                    }
                }

                if callee == "math::clamp" || callee == "clamp" {
                    if args.len() != 3 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 3 arguments (val, min, max), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::clamp(val, min, max)".to_string()),
                        });
                    }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let min_v = self.eval_expr(&args[1], Rc::clone(&env))?;
                    let max_v = self.eval_expr(&args[2], Rc::clone(&env))?;
                    match (&v, &min_v, &max_v) {
                        (Value::Int(iv), Value::Int(imin), Value::Int(imax)) => {
                            return Ok(Value::Int((*iv).clamp(*imin, *imax)));
                        }
                        _ => {
                            if let (Some(fv), Some(fmin), Some(fmax)) = (val_to_f64(&v), val_to_f64(&min_v), val_to_f64(&max_v)) {
                                return Ok(Value::Float(fv.clamp(fmin, fmax)));
                            } else {
                                return Err(RuntimeError {
                                    message: "math::clamp requires numeric arguments".to_string(),
                                    span: *span,
                                    hint: None,
                                });
                            }
                        }
                    }
                }

                if callee == "math::sin" || callee == "sin" {
                    if args.len() != 1 { return Err(RuntimeError { message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()), span: *span, hint: None }); }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&v) { return Ok(Value::Float(f.sin())); }
                }

                if callee == "math::cos" || callee == "cos" {
                    if args.len() != 1 { return Err(RuntimeError { message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()), span: *span, hint: None }); }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&v) { return Ok(Value::Float(f.cos())); }
                }

                if callee == "math::tan" || callee == "tan" {
                    if args.len() != 1 { return Err(RuntimeError { message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()), span: *span, hint: None }); }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&v) { return Ok(Value::Float(f.tan())); }
                }

                if callee == "math::log" || callee == "math::ln" || callee == "ln" {
                    if args.len() != 1 { return Err(RuntimeError { message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()), span: *span, hint: None }); }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&v) { return Ok(Value::Float(f.ln())); }
                }

                if callee == "math::log10" || callee == "log10" {
                    if args.len() != 1 { return Err(RuntimeError { message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()), span: *span, hint: None }); }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&v) { return Ok(Value::Float(f.log10())); }
                }

                if callee == "math::log2" || callee == "log2" {
                    if args.len() != 1 { return Err(RuntimeError { message: format!("Function '{}' expects 1 argument, got {}", callee, args.len()), span: *span, hint: None }); }
                    let v = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Some(f) = val_to_f64(&v) { return Ok(Value::Float(f.log2())); }
                }

                if callee == "math::random" || callee == "math::rand" || callee == "random" || callee == "rand" {
                    if args.is_empty() {
                        return Ok(Value::Float((next_rand_u64() as f64) / (u64::MAX as f64)));
                    } else if args.len() == 2 {
                        let min_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                        let max_val = self.eval_expr(&args[1], Rc::clone(&env))?;
                        match (min_val, max_val) {
                            (Value::Int(min), Value::Int(max)) => {
                                if min > max {
                                    return Err(RuntimeError {
                                        message: format!("math::random: min ({}) cannot be greater than max ({})", min, max),
                                        span: *span,
                                        hint: Some("Usage: math::random(min, max)".to_string()),
                                    });
                                }
                                let count = (max - min + 1) as u64;
                                let rand_val = min + (next_rand_u64() % count) as i64;
                                return Ok(Value::Int(rand_val));
                            }
                            (min, max) => {
                                if let (Some(fmin), Some(fmax)) = (val_to_f64(&min), val_to_f64(&max)) {
                                    let ratio = (next_rand_u64() as f64) / (u64::MAX as f64);
                                    return Ok(Value::Float(fmin + ratio * (fmax - fmin)));
                                } else {
                                    return Err(RuntimeError {
                                        message: "math::random requires numeric bounds".to_string(),
                                        span: *span,
                                        hint: None,
                                    });
                                }
                            }
                        }
                    } else {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 0 arguments or 2 arguments (min, max), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: math::random(1, 10) or math::random()".to_string()),
                        });
                    }
                }

                if callee == "math::randomFloat" || callee == "math::random_float" || callee == "randomFloat" || callee == "random_float" {
                    return Ok(Value::Float((next_rand_u64() as f64) / (u64::MAX as f64)));
                }

                // --- Time module functions ---
                if callee == "time::sleep" || callee == "sleep" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (milliseconds), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: time::sleep(500); (500 ms = 0.5 s)".to_string()),
                        });
                    }
                    let ms_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    let ms = match ms_val {
                        Value::Int(n) => n.max(0) as u64,
                        Value::Float(f) => (f * 1000.0).max(0.0) as u64,
                        _ => return Err(RuntimeError {
                            message: format!("time::sleep expects integer milliseconds, got '{}'", ms_val.get_type()),
                            span: *span,
                            hint: None,
                        }),
                    };
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                    return Ok(Value::Null);
                }

                if callee == "time::now" || callee == "now" {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                    return Ok(Value::Int(secs));
                }

                if callee == "time::nowMillis" || callee == "time::now_millis" || callee == "nowMillis" || callee == "now_millis" {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
                    return Ok(Value::Int(millis));
                }

                if callee == "time::elapsed" || callee == "elapsed" {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: format!("Function '{}' expects 1 argument (start timestamp in ms), got {}", callee, args.len()),
                            span: *span,
                            hint: Some("Usage: var int64: dt = time::elapsed(startMillis);".to_string()),
                        });
                    }
                    let start_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    if let Value::Int(start_ms) = start_val {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let current_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
                        return Ok(Value::Int((current_ms - start_ms).max(0)));
                    } else {
                        return Err(RuntimeError {
                            message: format!("time::elapsed expects integer start time in milliseconds, got '{}'", start_val.get_type()),
                            span: *span,
                            hint: None,
                        });
                    }
                }

                if callee == "input" && (args.is_empty() || args.len() == 1) {
                    if let Some(prompt_expr) = args.first() {
                        let prompt_val = self.eval_expr(prompt_expr, Rc::clone(&env))?;
                        print!("{}", prompt_val);
                        use std::io::Write;
                        let _ = std::io::stdout().flush();
                    }
                    let mut input_str = String::new();
                    if std::io::stdin().read_line(&mut input_str).is_ok() {
                        if input_str.ends_with('\n') {
                            input_str.pop();
                            if input_str.ends_with('\r') {
                                input_str.pop();
                            }
                        }
                        return Ok(Value::String(input_str));
                    } else {
                        return Ok(Value::String(String::new()));
                    }
                }



                if callee == "len" && args.len() == 1 {
                    let arg_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                    match arg_val {
                        Value::Array(items) => return Ok(Value::Int(items.len() as i64)),
                        Value::Tuple(items) => return Ok(Value::Int(items.len() as i64)),
                        Value::String(s) => return Ok(Value::Int(s.chars().count() as i64)),
                        Value::Map(entries_cell) => return Ok(Value::Int(entries_cell.borrow().len() as i64)),
                        _ => {
                            return Err(RuntimeError {
                                message: format!("Cannot get length of type '{}'", arg_val.get_type()),
                                span: *span,
                                hint: Some("len() supports arrays, tuples, maps, and strings".to_string()),
                            });
                        }
                    }
                }

                let func_var = env.borrow().get(callee);
                if let Some(var) = func_var {
                    if let Value::Tuple(ref items) = var.value {
                        if args.len() != 1 {
                            return Err(RuntimeError {
                                message: format!("Tuple indexing expects 1 integer index, got {} argument(s)", args.len()),
                                span: *span,
                                hint: Some(format!("Usage: {}(index) (e.g. {}(0))", callee, callee)),
                            });
                        }
                        let idx_val = self.eval_expr(&args[0], Rc::clone(&env))?;
                        if let Value::Int(idx) = idx_val {
                            if idx < 0 || (idx as usize) >= items.len() {
                                return Err(RuntimeError {
                                    message: format!("Index {} is out of bounds for tuple of length {}", idx, items.len()),
                                    span: *span,
                                    hint: Some(format!("Valid index range is 0 to {}", items.len().saturating_sub(1))),
                                });
                            }
                            return Ok(items[idx as usize].clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Tuple index must be an integer, got '{}'", idx_val.get_type()),
                                span: *span,
                                hint: Some(format!("Use an integer index: {}(0)", callee)),
                            });
                        }
                    }

                    if let Value::Function { params, return_type, body, .. } = var.value {
                        if args.len() != params.len() {
                            return Err(RuntimeError {
                                message: format!(
                                    "Function '{}' expected {} argument(s), got {}",
                                    callee,
                                    params.len(),
                                    args.len()
                                ),
                                span: *span,
                                hint: Some("Provide the correct number of arguments".to_string()),
                            });
                        }

                        let call_env = Environment::new_with_parent(Rc::clone(&self.global_env));

                        for (arg_expr, (param_name, param_type)) in args.iter().zip(params.iter()) {
                            let arg_val = self.eval_expr(arg_expr, Rc::clone(&env))?;
                            if !arg_val.matches_type(param_type) {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Type mismatch for parameter '{}' in function '{}': expected '{}', got '{}'",
                                        param_name,
                                        callee,
                                        param_type,
                                        arg_val.get_type()
                                    ),
                                    span: *span,
                                    hint: Some(format!("Pass an argument of type '{}'", param_type)),
                                });
                            }
                            call_env.borrow_mut().define(
                                param_name.clone(),
                                arg_val,
                                param_type.clone(),
                                false,
                            );
                        }

                        let res = self.execute_block(&body, call_env)?;
                        let ret_val = match res {
                            ControlFlow::Return(val) => val,
                            ControlFlow::None | ControlFlow::Break | ControlFlow::Continue => Value::Null,
                        };

                        if return_type != SherType::Void && return_type != SherType::Any {
                            if !ret_val.matches_type(&return_type) {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Return type mismatch for function '{}': expected '{}', but returned '{}' of type '{}'",
                                        callee,
                                        return_type,
                                        ret_val,
                                        ret_val.get_type()
                                    ),
                                    span: *span,
                                    hint: Some(format!("Return a value of type '{}'", return_type)),
                                });
                            }
                        }

                        Ok(ret_val)
                    } else {
                        Err(RuntimeError {
                            message: format!("'{}' is not a callable function", callee),
                            span: *span,
                            hint: Some("Check if the identifier belongs to a regular variable".to_string()),
                        })
                    }
                } else {
                    Err(RuntimeError {
                        message: format!("Undefined function '{}'", callee),
                        span: *span,
                        hint: Some(format!(
                            "Define the function before calling it: func {}() {{ ... }}",
                            callee
                        )),
                    })
                }
            }

            Expr::Range { start, end, .. } => {
                let s_val = self.eval_expr(start, Rc::clone(&env))?;
                let e_val = self.eval_expr(end, env)?;
                Ok(Value::Tuple(vec![s_val, e_val]))
            }
        }
    }
}
