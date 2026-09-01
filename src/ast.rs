use crate::token::Span;
use crate::types::SherType;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value, Span),
    Variable(String, Span),
    ArrayLiteral(Vec<Expr>, Span),
    TupleLiteral(Vec<Expr>, Span),
    MapLiteral {
        entries: Vec<(Expr, Expr)>,
        span: Span,
    },
    StructLiteral {
        struct_name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    FieldAccess {
        target: Box<Expr>,
        field: String,
        span: Span,
    },
    FieldAssign {
        target: Box<Expr>,
        field: String,
        value: Box<Expr>,
        span: Span,
    },
    Increment {
        target: Box<Expr>,
        is_prefix: bool,
        span: Span,
    },
    Decrement {
        target: Box<Expr>,
        is_prefix: bool,
        span: Span,
    },
    CompoundAssign {
        name: String,
        op: BinaryOp,
        value: Box<Expr>,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        span: Span,
    },
    Assign {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
}

#[allow(dead_code)]
impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span)
            | Expr::Variable(_, span)
            | Expr::ArrayLiteral(_, span)
            | Expr::TupleLiteral(_, span)
            | Expr::MapLiteral { span, .. }
            | Expr::StructLiteral { span, .. }
            | Expr::Index { span, .. }
            | Expr::FieldAccess { span, .. }
            | Expr::FieldAssign { span, .. }
            | Expr::Increment { span, .. }
            | Expr::Decrement { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Call { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::Range { span, .. }
            | Expr::Assign { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        is_const: bool,
        var_type: SherType,
        name: String,
        init: Expr,
        span: Span,
    },
    StructDef {
        name: String,
        fields: Vec<(String, SherType)>,
        span: Span,
    },
    EnumDef {
        name: String,
        variants: Vec<(String, Option<Expr>)>,
        span: Span,
    },
    FuncDecl {
        name: Option<String>,
        params: Vec<(String, SherType)>,
        return_type: SherType,
        body: Vec<Stmt>,
        span: Span,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    ForIn {
        item_name: String,
        item_type: SherType,
        iterable: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Break(Span),
    Continue(Span),
    Import {
        module: String,
        span: Span,
    },
    Print {
        args: Vec<Expr>,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    #[allow(dead_code)]
    IndexAssign {
        target: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    Expr(Expr),
    Block(Vec<Stmt>, Span),
}

#[allow(dead_code)]
impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl { span, .. }
            | Stmt::StructDef { span, .. }
            | Stmt::EnumDef { span, .. }
            | Stmt::FuncDecl { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForIn { span, .. }
            | Stmt::Break(span)
            | Stmt::Continue(span)
            | Stmt::Import { span, .. }
            | Stmt::Print { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::IndexAssign { span, .. }
            | Stmt::Block(_, span) => *span,
            Stmt::Expr(expr) => expr.span(),
        }
    }
}
