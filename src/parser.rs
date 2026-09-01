use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::token::{Span, Token, TokenType};
use crate::types::SherType;
use crate::value::Value;

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    loop_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Let]) {
            self.var_declaration(true)
        } else if self.match_token(&[TokenType::Var]) {
            self.var_declaration(false)
        } else if self.match_token(&[TokenType::Func]) {
            self.func_declaration()
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_token(&[TokenType::Break]) {
            self.break_statement()
        } else if self.match_token(&[TokenType::Continue]) {
            self.continue_statement()
        } else if self.match_token(&[TokenType::Import]) {
            self.import_statement()
        } else if self.match_token(&[TokenType::Struct]) {
            self.struct_definition()
        } else if self.match_token(&[TokenType::Enum]) {
            self.enum_definition()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenType::LBrace]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    fn import_statement(&mut self) -> Result<Stmt, ParserError> {
        let import_span = self.previous().span;
        let module: String;

        if self.match_token(&[TokenType::Less]) {
            let name_tok = self.advance();
            module = name_tok.lexeme.clone();
            self.consume(
                TokenType::Greater,
                "Expected '>' after imported module name",
                Some("Use syntax: import <io>"),
            )?;
        } else if let TokenType::StringLiteral(ref s) = self.peek().token_type.clone() {
            self.advance();
            module = s.clone();
        } else if let TokenType::Identifier(ref id) = self.peek().token_type.clone() {
            self.advance();
            module = id.clone();
        } else {
            return Err(ParserError {
                message: "Expected module name or file path after 'import'".to_string(),
                span: self.peek().span,
                hint: Some("Use syntax: import <io> or import \"file.sr\"".to_string()),
            });
        }

        if self.check(&TokenType::Semicolon) {
            self.advance();
        }

        Ok(Stmt::Import {
            module,
            span: import_span,
        })
    }

    fn struct_definition(&mut self) -> Result<Stmt, ParserError> {
        let span = self.previous().span;
        let name = if let TokenType::Identifier(n) = self.peek().token_type.clone() {
            self.advance();
            n
        } else {
            return Err(ParserError {
                message: "Expected struct name after 'struct'".to_string(),
                span,
                hint: Some("Provide a struct identifier (e.g. struct Osoba { ... })".to_string()),
            });
        };

        self.consume(
            TokenType::LBrace,
            "Expected '{' after struct name",
            Some("Open the struct definition with '{'"),
        )?;

        let mut fields = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            let field_type = self.parse_type()?;
            self.consume(
                TokenType::Colon,
                "Expected ':' after field type",
                Some("Use syntax: <type>: <field_name>; (e.g. string: imie;)"),
            )?;

            let field_name = if let TokenType::Identifier(n) = self.peek().token_type.clone() {
                self.advance();
                n
            } else {
                return Err(ParserError {
                    message: "Expected field name after ':' in struct definition".to_string(),
                    span: self.peek().span,
                    hint: Some("Provide a field name (e.g. string: imie;)".to_string()),
                });
            };

            self.consume(
                TokenType::Semicolon,
                "Expected ';' after struct field definition",
                Some("End each struct field with a semicolon ';' (e.g. string: imie;)"),
            )?;

            fields.push((field_name, field_type));
        }

        self.consume(
            TokenType::RBrace,
            "Expected '}' at the end of struct definition",
            Some("Close the struct definition with '}'"),
        )?;

        Ok(Stmt::StructDef { name, fields, span })
    }

    fn enum_definition(&mut self) -> Result<Stmt, ParserError> {
        let span = self.previous().span;
        let name = if let TokenType::Identifier(n) = self.peek().token_type.clone() {
            self.advance();
            n
        } else {
            return Err(ParserError {
                message: "Expected enum name after 'enum'".to_string(),
                span,
                hint: Some("Provide an enum identifier (e.g. enum Kierunek { Polnoc, Poludnie })".to_string()),
            });
        };

        self.consume(
            TokenType::LBrace,
            "Expected '{' after enum name",
            Some("Open the enum definition with '{'"),
        )?;

        let mut variants = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            let variant_name = if let TokenType::Identifier(n) = self.peek().token_type.clone() {
                self.advance();
                n
            } else {
                return Err(ParserError {
                    message: "Expected variant name in enum definition".to_string(),
                    span: self.peek().span,
                    hint: Some("Provide a variant identifier (e.g. Polnoc, Success = 200)".to_string()),
                });
            };

            let variant_value = if self.match_token(&[TokenType::Assign]) {
                Some(self.expression()?)
            } else {
                None
            };

            variants.push((variant_name, variant_value));

            if !self.match_token(&[TokenType::Comma, TokenType::Semicolon]) {
                break;
            }
        }

        self.consume(
            TokenType::RBrace,
            "Expected '}' at the end of enum definition",
            Some("Close the enum definition with '}'"),
        )?;

        if self.check(&TokenType::Semicolon) {
            self.advance();
        }

        Ok(Stmt::EnumDef { name, variants, span })
    }

    fn parse_type(&mut self) -> Result<SherType, ParserError> {
        if self.match_token(&[TokenType::LParen]) {
            // Tuple type: (int32, string, char)
            let mut types = Vec::new();
            if !self.check(&TokenType::RParen) {
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
            self.consume(
                TokenType::RParen,
                "Expected ')' after tuple type definition",
                Some("Close the tuple type with ')' (e.g. (int32, string, char))"),
            )?;
            return Ok(SherType::Tuple(types));
        }

        if self.match_token(&[TokenType::LBracket]) {
            // Array type: [int32]
            let inner = self.parse_type()?;
            self.consume(
                TokenType::RBracket,
                "Expected ']' after array type definition",
                Some("Close the array type with ']' (e.g. [int32])"),
            )?;
            return Ok(SherType::Array(Box::new(inner)));
        }

        let tok = self.advance();
        let base_type = match tok.token_type {
            TokenType::TypeInt8 => SherType::Int8,
            TokenType::TypeInt16 => SherType::Int16,
            TokenType::TypeInt26 => SherType::Int26,
            TokenType::TypeInt32 => SherType::Int32,
            TokenType::TypeInt64 => SherType::Int64,
            TokenType::TypeFloat8 => SherType::Float8,
            TokenType::TypeFloat16 => SherType::Float16,
            TokenType::TypeFloat32 => SherType::Float32,
            TokenType::TypeFloat64 => SherType::Float64,
            TokenType::TypeChar => SherType::Char,
            TokenType::TypeString => SherType::String,
            TokenType::TypeBool => SherType::Bool,
            TokenType::TypeVoid => SherType::Void,
            TokenType::TypeAny => SherType::Any,
            TokenType::Identifier(ref id) => {
                if id == "map" {
                    if self.match_token(&[TokenType::LBracket]) {
                        let key_type = self.parse_type()?;
                        self.consume(
                            TokenType::Comma,
                            "Expected ',' between map key and value types",
                            Some("Usage: map[key_type, value_type] (e.g. map[string, int32])"),
                        )?;
                        let val_type = self.parse_type()?;
                        self.consume(
                            TokenType::RBracket,
                            "Expected ']' after map type definition",
                            Some("Close the map type with ']' (e.g. map[string, int32])"),
                        )?;
                        return Ok(SherType::Map(Box::new(key_type), Box::new(val_type)));
                    } else {
                        return Ok(SherType::Map(Box::new(SherType::Any), Box::new(SherType::Any)));
                    }
                } else if let Some(t) = SherType::from_str(id) {
                    t
                } else {
                    return Err(ParserError {
                        message: format!("Unknown type '{}'", id),
                        span: tok.span,
                        hint: Some("Available types: int8, int16, int26, int32, int64, float32, float64, char, string, bool, map[k, v], (type1, type2)".to_string()),
                    });
                }
            }
            _ => {
                return Err(ParserError {
                    message: format!("Expected type, found '{}'", tok.lexeme),
                    span: tok.span,
                    hint: Some("Provide a valid type (e.g. int32, string, (int32, string))".to_string()),
                });
            }
        };

        if self.match_token(&[TokenType::LBracket]) {
            self.consume(
                TokenType::RBracket,
                "Expected ']' after '[' in array type",
                Some("Use syntax: int32[]"),
            )?;
            return Ok(SherType::Array(Box::new(base_type)));
        }

        Ok(base_type)
    }

    fn peek_is_type(&self) -> bool {
        match self.peek().token_type {
            TokenType::LParen
            | TokenType::LBracket
            | TokenType::TypeInt8
            | TokenType::TypeInt16
            | TokenType::TypeInt26
            | TokenType::TypeInt32
            | TokenType::TypeInt64
            | TokenType::TypeFloat8
            | TokenType::TypeFloat16
            | TokenType::TypeFloat32
            | TokenType::TypeFloat64
            | TokenType::TypeChar
            | TokenType::TypeString
            | TokenType::TypeBool
            | TokenType::TypeVoid
            | TokenType::TypeAny => true,
            TokenType::Identifier(ref id) => SherType::from_str(id).is_some(),
            _ => false,
        }
    }

    // Parses: let/var <type>: <name> = <expr>; OR let/var <name> = <expr>;
    fn var_declaration(&mut self, is_const: bool) -> Result<Stmt, ParserError> {
        let keyword_span = self.previous().span;
        let decl_name = if is_const { "let" } else { "var" };

        let mut var_type = SherType::Any;
        let var_name: String;

        if self.peek_is_type() && (self.peek().token_type != TokenType::LParen || self.peek_next_is_tuple_type()) {
            var_type = self.parse_type()?;
            self.consume(
                TokenType::Colon,
                "Expected ':' after variable type",
                Some(&format!("Use syntax: {} <type>: <name> = <value>; (e.g. {} int32: age = 15;)", decl_name, decl_name)),
            )?;

            if let TokenType::Identifier(name) = self.peek().token_type.clone() {
                self.advance();
                var_name = name;
            } else {
                return Err(ParserError {
                    message: "Expected variable name after ':'".to_string(),
                    span: self.peek().span,
                    hint: Some(format!("Provide a variable identifier (e.g. {} int32: age = 15;)", decl_name)),
                });
            }
        } else if let TokenType::Identifier(name) = self.peek().token_type.clone() {
            self.advance();
            var_name = name;

            if self.match_token(&[TokenType::Colon]) {
                var_type = self.parse_type()?;
            }
        } else {
            return Err(ParserError {
                message: format!("Expected type or variable name after '{}'", decl_name),
                span: self.peek().span,
                hint: Some(format!("Valid syntax: {} int32: age = 15;", decl_name)),
            });
        }

        self.consume(
            TokenType::Assign,
            "Expected '=' in variable declaration",
            Some("Assign an initial value using '='"),
        )?;
        let init = self.expression()?;
        let is_struct_init = matches!(init, Expr::StructLiteral { .. } | Expr::MapLiteral { .. });
        if is_struct_init {
            if self.check(&TokenType::Semicolon) {
                self.advance();
            }
        } else {
            self.consume(
                TokenType::Semicolon,
                "Expected ';' at the end of variable declaration",
                Some("All variable declarations in Sher must end with a semicolon ';' (e.g. var int32: i = 0;)"),
            )?;
        }

        Ok(Stmt::VarDecl {
            is_const,
            var_type,
            name: var_name,
            init,
            span: keyword_span,
        })
    }

    fn peek_next_is_tuple_type(&self) -> bool {
        if self.current + 1 < self.tokens.len() {
            let next = &self.tokens[self.current + 1];
            match next.token_type {
                TokenType::TypeInt8
                | TokenType::TypeInt16
                | TokenType::TypeInt26
                | TokenType::TypeInt32
                | TokenType::TypeInt64
                | TokenType::TypeFloat8
                | TokenType::TypeFloat16
                | TokenType::TypeFloat32
                | TokenType::TypeFloat64
                | TokenType::TypeChar
                | TokenType::TypeString
                | TokenType::TypeBool
                | TokenType::TypeVoid
                | TokenType::TypeAny => true,
                TokenType::Identifier(ref id) => SherType::from_str(id).is_some(),
                _ => false,
            }
        } else {
            false
        }
    }

    // Parses: func <name>(<params>) <return_type> { ... }
    fn func_declaration(&mut self) -> Result<Stmt, ParserError> {
        let func_span = self.previous().span;

        let name = if let TokenType::Identifier(n) = self.peek().token_type.clone() {
            self.advance();
            Some(n)
        } else {
            return Err(ParserError {
                message: "Expected function name after 'func'".to_string(),
                span: self.peek().span,
                hint: Some("Provide a name for the function (e.g. func main() { ... })".to_string()),
            });
        };

        self.consume(
            TokenType::LParen,
            "Expected '(' after function name",
            Some("Add parameter parentheses (e.g. func add(int32: A, int32: B) int32 { ... })"),
        )?;
        let mut params = Vec::new();

        if !self.check(&TokenType::RParen) {
            loop {
                let param_type = self.parse_type()?;
                self.consume(
                    TokenType::Colon,
                    "Expected ':' after parameter type",
                    Some("Use syntax: <type>: <name> (e.g. int32: A)"),
                )?;

                let param_name = if let TokenType::Identifier(pname) = self.peek().token_type.clone() {
                    self.advance();
                    pname
                } else {
                    return Err(ParserError {
                        message: "Expected parameter name after ':'".to_string(),
                        span: self.peek().span,
                        hint: Some("Provide a parameter identifier (e.g. int32: A)".to_string()),
                    });
                };

                params.push((param_name, param_type));

                if !self.match_token(&[TokenType::Comma, TokenType::Semicolon]) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RParen,
            "Expected ')' after function parameter list",
            Some("Close the parameter list with ')'"),
        )?;

        let mut return_type = SherType::Void;

        if self.check(&TokenType::Colon) || self.check(&TokenType::Semicolon) {
            self.advance();
        }

        if self.peek_is_type() {
            return_type = self.parse_type()?;
        }

        self.consume(
            TokenType::LBrace,
            "Expected '{' before function body",
            Some("Open the function body with '{'"),
        )?;
        let body = self.block_body()?;

        Ok(Stmt::FuncDecl {
            name,
            params,
            return_type,
            body,
            span: func_span,
        })
    }

    // Parses: if (cond) { ... } else { ... }
    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        let if_span = self.previous().span;

        let has_paren = self.match_token(&[TokenType::LParen]);
        let condition = self.expression()?;
        if has_paren {
            self.consume(
                TokenType::RParen,
                "Expected ')' after if condition",
                Some("Close the condition with ')'"),
            )?;
        }

        self.consume(
            TokenType::LBrace,
            "Expected '{' after if condition",
            Some("Start the 'if' block with '{'"),
        )?;
        let then_branch = self.block_body()?;

        let mut else_branch = None;
        if self.match_token(&[TokenType::Else]) {
            if self.match_token(&[TokenType::If]) {
                let else_if = self.if_statement()?;
                else_branch = Some(vec![else_if]);
            } else {
                self.consume(
                    TokenType::LBrace,
                    "Expected '{' after 'else'",
                    Some("Start the 'else' block with '{'"),
                )?;
                else_branch = Some(self.block_body()?);
            }
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span: if_span,
        })
    }

    // Parses: while (cond) { ... }
    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        let while_span = self.previous().span;

        let has_paren = self.match_token(&[TokenType::LParen]);
        let condition = self.expression()?;
        if has_paren {
            self.consume(
                TokenType::RParen,
                "Expected ')' after while condition",
                Some("Close the while condition with ')'"),
            )?;
        }

        self.consume(
            TokenType::LBrace,
            "Expected '{' after while condition",
            Some("Start the while body with '{'"),
        )?;

        self.loop_depth += 1;
        let body = self.block_body()?;
        self.loop_depth -= 1;

        Ok(Stmt::While {
            condition,
            body,
            span: while_span,
        })
    }

    // Parses: for (let/var <type>: <item> in <iterable>) { ... } OR for (<item> in <iterable>) { ... }
    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        let for_span = self.previous().span;

        let has_paren = self.match_token(&[TokenType::LParen]);

        let _is_decl = self.match_token(&[TokenType::Let, TokenType::Var]);

        let mut item_type = SherType::Any;
        let item_name: String;

        if self.peek_is_type() && (self.peek().token_type != TokenType::LParen || self.peek_next_is_tuple_type()) {
            item_type = self.parse_type()?;
            self.consume(TokenType::Colon, "Expected ':' after loop variable type", None)?;
            if let TokenType::Identifier(n) = self.advance().token_type {
                item_name = n;
            } else {
                return Err(ParserError {
                    message: "Expected loop variable name".to_string(),
                    span: for_span,
                    hint: Some("Provide a variable name (e.g. for (let int32: x in tablica))".to_string()),
                });
            }
        } else if let TokenType::Identifier(n) = self.advance().token_type {
            item_name = n;
        } else {
            return Err(ParserError {
                message: "Expected loop variable in for-in loop".to_string(),
                span: for_span,
                hint: Some("Use syntax: for (let int32: x in tablica) { ... }".to_string()),
            });
        }

        self.consume(
            TokenType::In,
            "Expected 'in' in for loop",
            Some("Use syntax: for (x in tablica)"),
        )?;

        let iterable = self.expression()?;

        if has_paren {
            self.consume(
                TokenType::RParen,
                "Expected ')' after for-in expression",
                Some("Close the for header with ')'"),
            )?;
        }

        self.consume(
            TokenType::LBrace,
            "Expected '{' after for header",
            Some("Start the for loop body with '{'"),
        )?;

        self.loop_depth += 1;
        let body = self.block_body()?;
        self.loop_depth -= 1;

        Ok(Stmt::ForIn {
            item_name,
            item_type,
            iterable,
            body,
            span: for_span,
        })
    }

    fn break_statement(&mut self) -> Result<Stmt, ParserError> {
        let span = self.previous().span;
        if self.loop_depth == 0 {
            return Err(ParserError {
                message: "'break' cannot be used outside of a loop".to_string(),
                span,
                hint: Some("'break' can only be used inside 'while' or 'for' loops. If you want to exit a function, use 'return;'".to_string()),
            });
        }
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after 'break'",
            Some("Add a semicolon ';' after 'break' (e.g. break;)"),
        )?;
        Ok(Stmt::Break(span))
    }

    fn continue_statement(&mut self) -> Result<Stmt, ParserError> {
        let span = self.previous().span;
        if self.loop_depth == 0 {
            return Err(ParserError {
                message: "'continue' cannot be used outside of a loop".to_string(),
                span,
                hint: Some("'continue' can only be used inside 'while' or 'for' loops".to_string()),
            });
        }
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after 'continue'",
            Some("Add a semicolon ';' after 'continue' (e.g. continue;)"),
        )?;
        Ok(Stmt::Continue(span))
    }

    // Parses: print(arg1, arg2, ...);
    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let print_span = self.previous().span;
        self.consume(
            TokenType::LParen,
            "Expected '(' after 'print'",
            Some("Use syntax: print(\"message\", variable);"),
        )?;

        let mut args = Vec::new();
        if !self.check(&TokenType::RParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RParen,
            "Expected ')' after print arguments",
            Some("Close the print argument list with ')'"),
        )?;
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after print statement",
            Some("All print statements in Sher must end with a semicolon ';' (e.g. print(i);)"),
        )?;

        Ok(Stmt::Print {
            args,
            span: print_span,
        })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParserError> {
        let ret_span = self.previous().span;
        let value = if !self.check(&TokenType::Semicolon) && !self.check(&TokenType::RBrace) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after return statement",
            Some("All return statements in Sher must end with a semicolon ';' (e.g. return a + b;)"),
        )?;

        Ok(Stmt::Return {
            value,
            span: ret_span,
        })
    }

    fn block_statement(&mut self) -> Result<Stmt, ParserError> {
        let brace_span = self.previous().span;
        let body = self.block_body()?;
        Ok(Stmt::Block(body, brace_span))
    }

    fn block_body(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut stmts = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            stmts.push(self.statement()?);
        }
        self.consume(
            TokenType::RBrace,
            "Expected '}' at the end of block",
            Some("Ensure every opening brace '{' has a matching closing brace '}'"),
        )?;
        Ok(stmts)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        let is_struct = matches!(expr, Expr::StructLiteral { .. });
        if is_struct {
            if self.check(&TokenType::Semicolon) {
                self.advance();
            }
        } else {
            self.consume(
                TokenType::Semicolon,
                "Expected ';' after expression statement",
                Some("All statements in Sher must end with a semicolon ';' (e.g. i++;)"),
            )?;
        }
        Ok(Stmt::Expr(expr))
    }

    pub fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.range()?;

        if self.match_token(&[TokenType::Assign]) {
            let equals = self.previous().span;
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name, _) => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                        span: equals,
                    });
                }
                Expr::FieldAccess { target, field, span } => {
                    return Ok(Expr::FieldAssign {
                        target,
                        field,
                        value: Box::new(value),
                        span,
                    });
                }
                Expr::Index { target, index, span } => {
                    return Ok(Expr::Call {
                        callee: "__index_assign__".to_string(),
                        args: vec![*target, *index, value],
                        span,
                    });
                }
                _ => {
                    return Err(ParserError {
                        message: "Invalid assignment target".to_string(),
                        span: equals,
                        hint: Some("Values can only be assigned to variables, struct fields, or array indices".to_string()),
                    });
                }
            }
        }

        if self.match_token(&[
            TokenType::PlusAssign,
            TokenType::MinusAssign,
            TokenType::StarAssign,
            TokenType::SlashAssign,
            TokenType::PercentAssign,
        ]) {
            let op_tok = self.previous();
            let op = match op_tok.token_type {
                TokenType::PlusAssign => BinaryOp::Add,
                TokenType::MinusAssign => BinaryOp::Subtract,
                TokenType::StarAssign => BinaryOp::Multiply,
                TokenType::SlashAssign => BinaryOp::Divide,
                TokenType::PercentAssign => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let value = self.assignment()?;

            if let Expr::Variable(name, _) = expr {
                return Ok(Expr::CompoundAssign {
                    name,
                    op,
                    value: Box::new(value),
                    span: op_tok.span,
                });
            }

            if let Expr::FieldAccess { target, field, span } = expr {
                let bin_expr = Expr::Binary {
                    left: Box::new(Expr::FieldAccess {
                        target: target.clone(),
                        field: field.clone(),
                        span,
                    }),
                    op,
                    right: Box::new(value),
                    span: op_tok.span,
                };
                return Ok(Expr::FieldAssign {
                    target,
                    field,
                    value: Box::new(bin_expr),
                    span: op_tok.span,
                });
            }

            return Err(ParserError {
                message: "Invalid compound assignment target".to_string(),
                span: op_tok.span,
                hint: Some("Compound assignment can only be used on variables or struct fields (e.g. g.hp += 1;)".to_string()),
            });
        }

        Ok(expr)
    }

    fn range(&mut self) -> Result<Expr, ParserError> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::DotDot]) {
            let span = self.previous().span;
            let end = self.or()?;
            return Ok(Expr::Range {
                start: Box::new(expr),
                end: Box::new(end),
                span,
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let span = self.previous().span;
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let span = self.previous().span;
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::Equal, TokenType::NotEqual]) {
            let op_tok = self.previous();
            let op = match op_tok.token_type {
                TokenType::Equal => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: op_tok.span,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEq,
            TokenType::Less,
            TokenType::LessEq,
        ]) {
            let op_tok = self.previous();
            let op = match op_tok.token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEq => BinaryOp::GreaterEq,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEq => BinaryOp::LessEq,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: op_tok.span,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op_tok = self.previous();
            let op = match op_tok.token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: op_tok.span,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let op_tok = self.previous();
            let op = match op_tok.token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: op_tok.span,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[TokenType::Not, TokenType::Minus]) {
            let op_tok = self.previous();
            let op = match op_tok.token_type {
                TokenType::Not => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Negate,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
                span: op_tok.span,
            });
        }

        if self.match_token(&[TokenType::PlusPlus]) {
            let span = self.previous().span;
            let target = self.unary()?;
            return Ok(Expr::Increment {
                target: Box::new(target),
                is_prefix: true,
                span,
            });
        }

        if self.match_token(&[TokenType::MinusMinus]) {
            let span = self.previous().span;
            let target = self.unary()?;
            return Ok(Expr::Decrement {
                target: Box::new(target),
                is_prefix: true,
                span,
            });
        }

        self.postfix()
    }

    // Handles postfix ++, --, function calls, and array/tuple indexing: tablica.[2]
    fn postfix(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::PlusPlus]) {
                let span = self.previous().span;
                expr = Expr::Increment {
                    target: Box::new(expr),
                    is_prefix: false,
                    span,
                };
            } else if self.match_token(&[TokenType::MinusMinus]) {
                let span = self.previous().span;
                expr = Expr::Decrement {
                    target: Box::new(expr),
                    is_prefix: false,
                    span,
                };
            } else if self.match_token(&[TokenType::LParen]) {
                // Function call: expr(args)
                let mut args = Vec::new();
                if !self.check(&TokenType::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                let rparen = self.consume(
                    TokenType::RParen,
                    "Expected ')' after function arguments",
                    Some("Close the function call with ')'"),
                )?;

                if let Expr::Variable(name, span) = expr {
                    expr = Expr::Call {
                        callee: name,
                        args,
                        span,
                    };
                } else {
                    return Err(ParserError {
                        message: "Only named functions can be called directly".to_string(),
                        span: rparen.span,
                        hint: None,
                    });
                }
            } else if self.match_token(&[TokenType::Dot]) {
                let dot_span = self.previous().span;
                if self.match_token(&[TokenType::LBracket]) {
                    let index = self.expression()?;
                    self.consume(
                        TokenType::RBracket,
                        "Expected ']' after index",
                        Some("Use syntax: array.[index] or tuple.[index]"),
                    )?;
                    expr = Expr::Index {
                        target: Box::new(expr),
                        index: Box::new(index),
                        span: dot_span,
                    };
                } else if let TokenType::IntLiteral(n) = self.peek().token_type {
                    let tok = self.advance();
                    expr = Expr::Index {
                        target: Box::new(expr),
                        index: Box::new(Expr::Literal(Value::Int(n), tok.span)),
                        span: dot_span,
                    };
                } else if let TokenType::Identifier(member_name) = self.peek().token_type.clone() {
                    self.advance();
                    if self.match_token(&[TokenType::LParen]) {
                        let mut args = Vec::new();
                        if !self.check(&TokenType::RParen) {
                            loop {
                                args.push(self.expression()?);
                                if !self.match_token(&[TokenType::Comma]) {
                                    break;
                                }
                            }
                        }
                        self.consume(
                            TokenType::RParen,
                            "Expected ')' after method arguments",
                            Some("Close the method call with ')'"),
                        )?;
                        expr = Expr::MethodCall {
                            target: Box::new(expr),
                            method: member_name,
                            args,
                            span: dot_span,
                        };
                    } else {
                        expr = Expr::FieldAccess {
                            target: Box::new(expr),
                            field: member_name,
                            span: dot_span,
                        };
                    }
                } else {
                    return Err(ParserError {
                        message: "Expected field name, method name, '[' or integer index after '.'".to_string(),
                        span: self.peek().span,
                        hint: Some("Use syntax: object.field, tablica.add(val), tablica[0]".to_string()),
                    });
                }
            } else if self.match_token(&[TokenType::LBracket]) {
                // Standard indexing: tablica[2]
                let bracket_span = self.previous().span;
                let index = self.expression()?;
                self.consume(
                    TokenType::RBracket,
                    "Expected ']' after index",
                    Some("Use syntax: array[index]"),
                )?;
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                    span: bracket_span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn peek_is_struct_literal(&self) -> bool {
        if self.current < self.tokens.len() && self.tokens[self.current].token_type == TokenType::LBrace {
            if self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].token_type == TokenType::RBrace {
                return true;
            }
            if self.current + 2 < self.tokens.len() {
                if let TokenType::Identifier(_) = self.tokens[self.current + 1].token_type {
                    if self.tokens[self.current + 2].token_type == TokenType::Colon {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        let tok = self.advance();
        match tok.token_type {
            TokenType::IntLiteral(n) => Ok(Expr::Literal(Value::Int(n), tok.span)),
            TokenType::FloatLiteral(f) => Ok(Expr::Literal(Value::Float(f), tok.span)),
            TokenType::CharLiteral(c) => Ok(Expr::Literal(Value::Char(c), tok.span)),
            TokenType::StringLiteral(s) => Ok(Expr::Literal(Value::String(s), tok.span)),
            TokenType::True => Ok(Expr::Literal(Value::Bool(true), tok.span)),
            TokenType::False => Ok(Expr::Literal(Value::Bool(false), tok.span)),
            TokenType::Null => Ok(Expr::Literal(Value::Null, tok.span)),
            TokenType::Identifier(mut id) => {
                let span = tok.span;
                while self.match_token(&[TokenType::ColonColon]) {
                    let next_tok = self.advance();
                    id.push_str("::");
                    id.push_str(&next_tok.lexeme);
                }

                if self.check(&TokenType::LBrace) && self.peek_is_struct_literal() {
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();
                    if !self.check(&TokenType::RBrace) {
                        loop {
                            let fname = if let TokenType::Identifier(n) = self.peek().token_type.clone() {
                                self.advance();
                                n
                            } else {
                                return Err(ParserError {
                                    message: "Expected field name in struct literal".to_string(),
                                    span: self.peek().span,
                                    hint: Some("Use syntax: StructName { field: value }".to_string()),
                                });
                            };
                            self.consume(
                                TokenType::Colon,
                                "Expected ':' after field name in struct literal",
                                Some("Use syntax: field_name: value"),
                            )?;
                            let val_expr = self.expression()?;
                            fields.push((fname, val_expr));
                            if !self.match_token(&[TokenType::Comma, TokenType::Semicolon]) {
                                break;
                            }
                        }
                    }
                    self.consume(
                        TokenType::RBrace,
                        "Expected '}' at the end of struct literal",
                        Some("Close the struct literal with '}'"),
                    )?;
                    return Ok(Expr::StructLiteral {
                        struct_name: id,
                        fields,
                        span,
                    });
                }

                Ok(Expr::Variable(id, span))
            }

            // Array literal: [5, 6, 5, 3, 4]
            TokenType::LBracket => {
                let start_span = tok.span;
                let mut items = Vec::new();
                if !self.check(&TokenType::RBracket) {
                    loop {
                        items.push(self.expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(
                    TokenType::RBracket,
                    "Expected ']' at the end of array literal",
                    Some("Close the array with ']' (e.g. [1, 2, 3])"),
                )?;
                Ok(Expr::ArrayLiteral(items, start_span))
            }

            // Tuple literal or parenthesized expression: (5, "sok", 'a') OR (1 + 2)
            TokenType::LParen => {
                let start_span = tok.span;
                if self.check(&TokenType::RParen) {
                    self.advance();
                    return Ok(Expr::TupleLiteral(Vec::new(), start_span));
                }

                let first = self.expression()?;

                if self.match_token(&[TokenType::Comma]) {
                    let mut items = vec![first];
                    if !self.check(&TokenType::RParen) {
                        loop {
                            items.push(self.expression()?);
                            if !self.match_token(&[TokenType::Comma]) {
                                break;
                            }
                        }
                    }
                    self.consume(
                        TokenType::RParen,
                        "Expected ')' at the end of tuple literal",
                        Some("Close the tuple with ')' (e.g. (1, \"text\", 'c'))"),
                    )?;
                    Ok(Expr::TupleLiteral(items, start_span))
                } else {
                    self.consume(
                        TokenType::RParen,
                        "Expected ')' after expression",
                        Some("Add a matching closing parenthesis ')'"),
                    )?;
                    Ok(first)
                }
            }

            // Map literal: { "a": 1, "b": 2 } or {}
            TokenType::LBrace => {
                let start_span = tok.span;
                let mut entries = Vec::new();
                if !self.check(&TokenType::RBrace) {
                    loop {
                        let key_expr = self.expression()?;
                        self.consume(
                            TokenType::Colon,
                            "Expected ':' after map key",
                            Some("Use syntax: { key: value, ... }"),
                        )?;
                        let val_expr = self.expression()?;
                        entries.push((key_expr, val_expr));
                        if !self.match_token(&[TokenType::Comma, TokenType::Semicolon]) {
                            break;
                        }
                    }
                }
                self.consume(
                    TokenType::RBrace,
                    "Expected '}' at the end of map literal",
                    Some("Close the map with '}' (e.g. { \"key\": value })"),
                )?;
                Ok(Expr::MapLiteral {
                    entries,
                    span: start_span,
                })
            }

            _ => Err(ParserError {
                message: format!("Unexpected token '{}'", tok.lexeme),
                span: tok.span,
                hint: Some("Check syntax at this position".to_string()),
            }),
        }
    }

    // Helper methods
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(
        &mut self,
        token_type: TokenType,
        message: &str,
        hint: Option<&str>,
    ) -> Result<Token, ParserError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            let prev = self.previous();
            let peek = self.peek();

            // When expecting a semicolon, point right at the end of the previous token if line changed or at block end
            let span = if token_type == TokenType::Semicolon
                && (peek.span.line > prev.span.line
                    || peek.token_type == TokenType::RBrace
                    || peek.token_type == TokenType::Eof)
            {
                Span {
                    line: prev.span.line,
                    column: prev.span.column + prev.lexeme.len(),
                }
            } else {
                peek.span
            };

            Err(ParserError {
                message: message.to_string(),
                span,
                hint: hint.map(|s| s.to_string()),
            })
        }
    }
}
