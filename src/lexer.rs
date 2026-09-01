use crate::token::{Span, Token, TokenType};

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

pub struct Lexer<'a> {
    _source: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            _source: source,
            chars,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            let start_span = Span {
                line: self.line,
                column: self.col,
            };

            match ch {
                // Whitespace
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.col = 1;
                }

                // Comments & Slash operators
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        self.advance(); // consume /
                        self.advance(); // consume *
                        let mut closed = false;
                        while let Some(c) = self.peek() {
                            if c == '*' && self.peek_next() == Some('/') {
                                self.advance(); // consume *
                                self.advance(); // consume /
                                closed = true;
                                break;
                            }
                            if c == '\n' {
                                self.line += 1;
                                self.col = 0;
                            }
                            self.advance();
                        }
                        if !closed {
                            return Err(LexerError {
                                message: "Unterminated block comment".to_string(),
                                span: start_span,
                                hint: Some("Close the block comment with '*/'".to_string()),
                            });
                        }
                    } else {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            tokens.push(Token::new(TokenType::SlashAssign, start_span, "/="));
                        } else {
                            tokens.push(Token::new(TokenType::Slash, start_span, "/"));
                        }
                    }
                }

                // Delimiters
                '(' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::LParen, start_span, "("));
                }
                ')' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::RParen, start_span, ")"));
                }
                '{' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::LBrace, start_span, "{"));
                }
                '}' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::RBrace, start_span, "}"));
                }
                '[' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::LBracket, start_span, "["));
                }
                ']' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::RBracket, start_span, "]"));
                }
                ',' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::Comma, start_span, ","));
                }
                ':' => {
                    self.advance();
                    if self.peek() == Some(':') {
                        self.advance();
                        tokens.push(Token::new(TokenType::ColonColon, start_span, "::"));
                    } else {
                        tokens.push(Token::new(TokenType::Colon, start_span, ":"));
                    }
                }
                ';' => {
                    self.advance();
                    tokens.push(Token::new(TokenType::Semicolon, start_span, ";"));
                }
                '.' => {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        tokens.push(Token::new(TokenType::DotDot, start_span, ".."));
                    } else {
                        tokens.push(Token::new(TokenType::Dot, start_span, "."));
                    }
                }

                // Operators
                '+' => {
                    self.advance();
                    if self.peek() == Some('+') {
                        self.advance();
                        tokens.push(Token::new(TokenType::PlusPlus, start_span, "++"));
                    } else if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::PlusAssign, start_span, "+="));
                    } else {
                        tokens.push(Token::new(TokenType::Plus, start_span, "+"));
                    }
                }
                '-' => {
                    self.advance();
                    if self.peek() == Some('-') {
                        self.advance();
                        tokens.push(Token::new(TokenType::MinusMinus, start_span, "--"));
                    } else if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::MinusAssign, start_span, "-="));
                    } else {
                        tokens.push(Token::new(TokenType::Minus, start_span, "-"));
                    }
                }
                '*' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::StarAssign, start_span, "*="));
                    } else {
                        tokens.push(Token::new(TokenType::Star, start_span, "*"));
                    }
                }
                '%' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::PercentAssign, start_span, "%="));
                    } else {
                        tokens.push(Token::new(TokenType::Percent, start_span, "%"));
                    }
                }

                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::Equal, start_span, "=="));
                    } else {
                        tokens.push(Token::new(TokenType::Assign, start_span, "="));
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::NotEqual, start_span, "!="));
                    } else {
                        tokens.push(Token::new(TokenType::Not, start_span, "!"));
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::GreaterEq, start_span, ">="));
                    } else {
                        tokens.push(Token::new(TokenType::Greater, start_span, ">"));
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenType::LessEq, start_span, "<="));
                    } else {
                        tokens.push(Token::new(TokenType::Less, start_span, "<"));
                    }
                }
                '&' => {
                    self.advance();
                    if self.peek() == Some('&') {
                        self.advance();
                        tokens.push(Token::new(TokenType::And, start_span, "&&"));
                    } else {
                        return Err(LexerError {
                            message: "Unexpected character '&'".to_string(),
                            span: start_span,
                            hint: Some("For logical AND use '&&'".to_string()),
                        });
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') {
                        self.advance();
                        tokens.push(Token::new(TokenType::Or, start_span, "||"));
                    } else {
                        return Err(LexerError {
                            message: "Unexpected character '|'".to_string(),
                            span: start_span,
                            hint: Some("For logical OR use '||'".to_string()),
                        });
                    }
                }

                // Strings
                '"' => {
                    let s = self.read_string(start_span)?;
                    tokens.push(Token::new(TokenType::StringLiteral(s.clone()), start_span, format!("\"{}\"", s)));
                }

                // Chars
                '\'' => {
                    let c = self.read_char(start_span)?;
                    tokens.push(Token::new(TokenType::CharLiteral(c), start_span, format!("'{}'", c)));
                }

                // Numbers
                '0'..='9' => {
                    let (tok_type, lexeme) = self.read_number(start_span)?;
                    tokens.push(Token::new(tok_type, start_span, lexeme));
                }

                // Identifiers & Keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = self.read_identifier();
                    let tok_type = match ident.as_str() {
                        "func" => TokenType::Func,
                        "let" => TokenType::Let,
                        "var" => TokenType::Var,
                        "if" => TokenType::If,
                        "else" => TokenType::Else,
                        "while" => TokenType::While,
                        "for" => TokenType::For,
                        "in" => TokenType::In,
                        "break" => TokenType::Break,
                        "continue" => TokenType::Continue,
                        "import" => TokenType::Import,
                        "struct" => TokenType::Struct,
                        "enum" => TokenType::Enum,
                        "print" => TokenType::Print,
                        "true" => TokenType::True,
                        "false" => TokenType::False,
                        "null" => TokenType::Null,
                        "return" => TokenType::Return,

                        // Integer types
                        "int8" | "i8" => TokenType::TypeInt8,
                        "int16" | "i16" => TokenType::TypeInt16,
                        "int26" | "i26" => TokenType::TypeInt26,
                        "int32" | "i32" => TokenType::TypeInt32,
                        "int64" | "i64" | "int" => TokenType::TypeInt64,

                        // Float types
                        "float8" | "f8" => TokenType::TypeFloat8,
                        "float16" | "f16" => TokenType::TypeFloat16,
                        "float32" | "f32" | "float" => TokenType::TypeFloat32,
                        "float64" | "f64" => TokenType::TypeFloat64,

                        // Other types
                        "char" => TokenType::TypeChar,
                        "string" | "str" => TokenType::TypeString,
                        "bool" | "boolean" => TokenType::TypeBool,
                        "void" => TokenType::TypeVoid,
                        "any" => TokenType::TypeAny,

                        _ => TokenType::Identifier(ident.clone()),
                    };
                    tokens.push(Token::new(tok_type, start_span, ident));
                }

                _ => {
                    let err_span = start_span;
                    self.advance();
                    return Err(LexerError {
                        message: format!("Unexpected character '{}'", ch),
                        span: err_span,
                        hint: Some("Remove the invalid character or check syntax".to_string()),
                    });
                }
            }
        }

        tokens.push(Token::new(
            TokenType::Eof,
            Span {
                line: self.line,
                column: self.col,
            },
            "",
        ));

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        if self.cursor < self.chars.len() {
            Some(self.chars[self.cursor].1)
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.cursor + 1 < self.chars.len() {
            Some(self.chars[self.cursor + 1].1)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor].1;
            self.cursor += 1;
            self.col += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn read_char(&mut self, start_span: Span) -> Result<char, LexerError> {
        self.advance(); // consume opening quote '\''

        let c = match self.peek() {
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('n') => { self.advance(); '\n' }
                    Some('t') => { self.advance(); '\t' }
                    Some('r') => { self.advance(); '\r' }
                    Some('\\') => { self.advance(); '\\' }
                    Some('\'') => { self.advance(); '\'' }
                    Some('\"') => { self.advance(); '\"' }
                    Some('0') => { self.advance(); '\0' }
                    Some(other) => { self.advance(); other }
                    None => return Err(LexerError {
                        message: "Unterminated escape sequence in char literal".to_string(),
                        span: start_span,
                        hint: Some("Fix the escape sequence after '\\'".to_string()),
                    }),
                }
            }
            Some('\'') => {
                return Err(LexerError {
                    message: "Empty char literal".to_string(),
                    span: start_span,
                    hint: Some("Character literals must contain exactly one character (e.g. 'a')".to_string()),
                });
            }
            Some(ch) => {
                self.advance();
                ch
            }
            None => {
                return Err(LexerError {
                    message: "Unterminated char literal".to_string(),
                    span: start_span,
                    hint: Some("Add a closing quote '\''".to_string()),
                });
            }
        };

        if self.peek() == Some('\'') {
            self.advance(); // consume closing quote '\''
            Ok(c)
        } else {
            Err(LexerError {
                message: "Unclosed character literal (expected single character in quotes)".to_string(),
                span: start_span,
                hint: Some("Character literals must contain only one character (e.g. 'x')".to_string()),
            })
        }
    }

    fn read_string(&mut self, start_span: Span) -> Result<String, LexerError> {
        self.advance(); // consume opening quote '"'
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(result);
            } else if ch == '\\' {
                self.advance(); // consume '\'
                match self.peek() {
                    Some('n') => {
                        self.advance();
                        result.push('\n');
                    }
                    Some('t') => {
                        self.advance();
                        result.push('\t');
                    }
                    Some('r') => {
                        self.advance();
                        result.push('\r');
                    }
                    Some('\\') => {
                        self.advance();
                        result.push('\\');
                    }
                    Some('"') => {
                        self.advance();
                        result.push('"');
                    }
                    Some('\'') => {
                        self.advance();
                        result.push('\'');
                    }
                    Some(c) => {
                        self.advance();
                        result.push(c);
                    }
                    None => {
                        return Err(LexerError {
                            message: "Unterminated escape sequence in string".to_string(),
                            span: start_span,
                            hint: Some("Fix the escape sequence after '\\'".to_string()),
                        });
                    }
                }
            } else if ch == '\n' {
                return Err(LexerError {
                    message: "Unterminated string literal (newline inside quotes)".to_string(),
                    span: start_span,
                    hint: Some("Add a closing quote '\"' before the newline".to_string()),
                });
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(LexerError {
            message: "Unterminated string literal".to_string(),
            span: start_span,
            hint: Some("Add a closing quote '\"' at the end of the string".to_string()),
        })
    }

    fn read_number(&mut self, _start_span: Span) -> Result<(TokenType, String), LexerError> {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            match num_str.parse::<f64>() {
                Ok(val) => Ok((TokenType::FloatLiteral(val), num_str)),
                Err(_) => Err(LexerError {
                    message: format!("Invalid float literal '{}'", num_str),
                    span: _start_span,
                    hint: Some("Expected valid float format (e.g. 3.14)".to_string()),
                }),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(val) => Ok((TokenType::IntLiteral(val), num_str)),
                Err(_) => Err(LexerError {
                    message: format!("Invalid integer literal '{}'", num_str),
                    span: _start_span,
                    hint: Some("Expected valid integer format (e.g. 42)".to_string()),
                }),
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }
}
