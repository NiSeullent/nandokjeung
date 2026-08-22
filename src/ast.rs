use crate::token::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Decl {
        name: Identifier,
        line: usize,
        column: usize,
    },
    Assign {
        name: Identifier,
        value: Expr,
        line: usize,
        column: usize,
    },
    Expr {
        expr: Expr,
        line: usize,
        column: usize,
    },
    Output {
        value: Expr,
        line: usize,
        column: usize,
    },
    Input {
        name: Identifier,
        line: usize,
        column: usize,
    },
    Return {
        value: Expr,
        line: usize,
        column: usize,
    },
    Throw {
        value: Expr,
        line: usize,
        column: usize,
    },
    Comment {
        tokens: Vec<String>,
        line: usize,
        column: usize,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    For {
        var: Identifier,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    FuncDef {
        name: Identifier,
        params: Vec<Identifier>,
        body: Vec<Stmt>,
        line: usize,
        column: usize,
    },
    Try {
        try_body: Vec<Stmt>,
        catch_body: Vec<Stmt>,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int {
        value: i64,
        line: usize,
        column: usize,
    },
    String {
        value: String,
        line: usize,
        column: usize,
    },
    Bool {
        value: bool,
        line: usize,
        column: usize,
    },
    Var {
        name: Identifier,
        line: usize,
        column: usize,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        line: usize,
        column: usize,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        line: usize,
        column: usize,
    },
    Group {
        inner: Box<Expr>,
        line: usize,
        column: usize,
    },
    Call {
        name: Identifier,
        args: Vec<Expr>,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Lt,
    Gt,
    Eq,
    Ne,
    And,
    Or,
}

impl Expr {
    pub fn span(&self) -> (usize, usize) {
        match self {
            Expr::Int { line, column, .. }
            | Expr::String { line, column, .. }
            | Expr::Bool { line, column, .. }
            | Expr::Var { line, column, .. }
            | Expr::Unary { line, column, .. }
            | Expr::Binary { line, column, .. }
            | Expr::Group { line, column, .. }
            | Expr::Call { line, column, .. } => (*line, *column),
        }
    }
}

impl Stmt {
    pub fn span(&self) -> (usize, usize) {
        match self {
            Stmt::Decl { line, column, .. }
            | Stmt::Assign { line, column, .. }
            | Stmt::Expr { line, column, .. }
            | Stmt::Output { line, column, .. }
            | Stmt::Input { line, column, .. }
            | Stmt::Return { line, column, .. }
            | Stmt::Throw { line, column, .. }
            | Stmt::Comment { line, column, .. }
            | Stmt::If { line, column, .. }
            | Stmt::While { line, column, .. }
            | Stmt::For { line, column, .. }
            | Stmt::FuncDef { line, column, .. }
            | Stmt::Try { line, column, .. } => (*line, *column),
        }
    }
}
