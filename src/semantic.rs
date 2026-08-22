use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::{CompileError, Result};
use crate::token::Identifier;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Jeong,
    Ppyung,
    Mental,
    Void,
    Ipnwon,
}

impl Type {
    pub fn name_ko(self) -> &'static str {
        match self {
            Type::Jeong => "정수",
            Type::Ppyung => "뼝문자",
            Type::Mental => "멘탈",
            Type::Void => "공허",
            Type::Ipnwon => "입원함수",
        }
    }
}

#[derive(Clone)]
struct VarInfo {
    ty: Type,
    mutable: bool,
}

#[derive(Clone)]
struct FuncInfo {
    params: Vec<Identifier>,
    return_type: Type,
    defined_at: (usize, usize),
}

struct Scope {
    vars: HashMap<String, VarInfo>,
}

pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    functions: HashMap<String, FuncInfo>,
    in_function: bool,
    current_func_return: Type,
    errors: Vec<CompileError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                vars: HashMap::new(),
            }],
            functions: HashMap::new(),
            in_function: false,
            current_func_return: Type::Void,
            errors: Vec::new(),
        }
    }

    pub fn analyze(program: &Program) -> Result<()> {
        let mut analyzer = Self::new();
        analyzer.register_builtins();
        for stmt in &program.statements {
            analyzer.check_stmt(stmt);
        }
        if let Some(err) = analyzer.errors.into_iter().next() {
            return Err(err);
        }
        Ok(())
    }

    fn register_builtins(&mut self) {
        self.functions.insert(
            "행정 정신".to_string(),
            FuncInfo {
                params: vec![],
                return_type: Type::Jeong,
                defined_at: (0, 0),
            },
        );
        self.functions.insert(
            "행정 뼝".to_string(),
            FuncInfo {
                params: vec![],
                return_type: Type::Jeong,
                defined_at: (0, 0),
            },
        );
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            vars: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_var(&mut self, name: &Identifier, ty: Type, line: usize, column: usize) {
        let key = name.name();
        if self.scopes.last().unwrap().vars.contains_key(&key) {
            self.errors.push(CompileError::semantic(
                "E002",
                format!("duplicate declaration of '{key}'"),
                line,
                column,
            ));
            return;
        }
        self.scopes.last_mut().unwrap().vars.insert(
            key,
            VarInfo {
                ty,
                mutable: true,
            },
        );
    }

    fn resolve_var(&self, name: &Identifier) -> Option<Type> {
        let key = name.name();
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.vars.get(&key) {
                return Some(info.ty);
            }
        }
        None
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl { name, line, column } => {
                self.declare_var(name, Type::Jeong, *line, *column);
            }
            Stmt::Assign { name, value, line, column } => {
                let value_ty = self.check_expr(value);
                match self.resolve_var(name) {
                    Some(var_ty) => {
                        if !types_compatible(var_ty, value_ty) {
                            self.errors.push(type_error(*line, *column, var_ty, value_ty));
                        }
                    }
                    None => {
                        self.declare_var(name, value_ty, *line, *column);
                    }
                }
            }
            Stmt::Expr { expr, .. } => {
                self.check_expr(expr);
            }
            Stmt::Output { value, .. } | Stmt::Return { value, .. } | Stmt::Throw { value, .. } => {
                let ty = self.check_expr(value);
                if matches!(stmt, Stmt::Return { .. })
                    && !types_compatible(self.current_func_return, ty)
                    && self.current_func_return != Type::Void
                {
                    let (line, column) = stmt.span();
                    self.errors.push(CompileError::semantic(
                        "E020",
                        format!(
                            "return type mismatch: expected {}, got {}",
                            self.current_func_return.name_ko(),
                            ty.name_ko()
                        ),
                        line,
                        column,
                    ));
                }
            }
            Stmt::Input { name, line, column } => {
                if self.resolve_var(name).is_none() {
                    self.errors.push(CompileError::semantic(
                        "E001",
                        format!("undefined variable '{}'", name.name()),
                        *line,
                        *column,
                    ));
                }
            }
            Stmt::Comment { .. } => {}
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.check_expr(condition);
                self.push_scope();
                for s in then_body {
                    self.check_stmt(s);
                }
                self.pop_scope();
                self.push_scope();
                for s in else_body {
                    self.check_stmt(s);
                }
                self.pop_scope();
            }
            Stmt::While { condition, body, .. } => {
                self.check_expr(condition);
                self.push_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.pop_scope();
            }
            Stmt::For {
                var,
                start,
                end,
                body,
                line,
                column,
            } => {
                let s_ty = self.check_expr(start);
                let e_ty = self.check_expr(end);
                if s_ty != Type::Jeong || e_ty != Type::Jeong {
                    self.errors.push(CompileError::semantic(
                        "E003",
                        "for range requires 정수 bounds",
                        *line,
                        *column,
                    ));
                }
                self.push_scope();
                self.declare_var(var, Type::Jeong, *line, *column);
                for s in body {
                    self.check_stmt(s);
                }
                self.pop_scope();
            }
            Stmt::FuncDef {
                name,
                params,
                body,
                line,
                column,
            } => {
                let fname = name.name();
                if self.functions.contains_key(&fname) && fname != "행정 정신" && fname != "행정 뼝"
                {
                    self.errors.push(CompileError::semantic(
                        "E011",
                        format!("duplicate function '{fname}'"),
                        *line,
                        *column,
                    ));
                }
                self.functions.insert(
                    fname.clone(),
                    FuncInfo {
                        params: params.clone(),
                        return_type: Type::Void,
                        defined_at: (*line, *column),
                    },
                );

                let prev_in = self.in_function;
                let prev_ret = self.current_func_return;
                self.in_function = true;
                self.current_func_return = Type::Void;

                self.push_scope();
                for p in params {
                    self.declare_var(p, Type::Jeong, *line, *column);
                }
                for s in body {
                    self.check_stmt(s);
                    if let Stmt::Return { value, .. } = s {
                        let rty = self.check_expr(value);
                        if let Some(func) = self.functions.get_mut(&fname) {
                            func.return_type = rty;
                            self.current_func_return = rty;
                        }
                    }
                }
                self.pop_scope();
                self.in_function = prev_in;
                self.current_func_return = prev_ret;
            }
            Stmt::Try {
                try_body,
                catch_body,
                ..
            } => {
                self.push_scope();
                for s in try_body {
                    self.check_stmt(s);
                }
                self.pop_scope();
                self.push_scope();
                for s in catch_body {
                    self.check_stmt(s);
                }
                self.pop_scope();
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Int { .. } => Type::Jeong,
            Expr::String { .. } => Type::Ppyung,
            Expr::Bool { .. } => Type::Mental,
            Expr::Var { name, line, column } => match self.resolve_var(name) {
                Some(ty) => ty,
                None => {
                    self.errors.push(CompileError::semantic(
                        "E001",
                        format!("undefined variable '{}'", name.name()),
                        *line,
                        *column,
                    ));
                    Type::Jeong
                }
            },
            Expr::Unary { op, operand, line, column } => {
                let ty = self.check_expr(operand);
                match op {
                    UnaryOp::Not => {
                        if ty != Type::Mental {
                            self.errors.push(CompileError::semantic(
                                "E003",
                                format!("! requires 멘탈, got {}", ty.name_ko()),
                                *line,
                                *column,
                            ));
                        }
                        Type::Mental
                    }
                }
            }
            Expr::Binary { op, left, right, line, column } => {
                let lt = self.check_expr(left);
                let rt = self.check_expr(right);
                match op {
                    BinaryOp::Add => {
                        if lt == Type::Jeong && rt == Type::Jeong {
                            Type::Jeong
                        } else if lt == Type::Ppyung && rt == Type::Ppyung {
                            Type::Ppyung
                        } else {
                            self.errors.push(type_error(*line, *column, lt, rt));
                            Type::Jeong
                        }
                    }
                    BinaryOp::Sub | BinaryOp::Mul => {
                        if lt != Type::Jeong || rt != Type::Jeong {
                            self.errors.push(type_error(*line, *column, lt, rt));
                        }
                        Type::Jeong
                    }
                    BinaryOp::Lt | BinaryOp::Gt => {
                        if lt != Type::Jeong || rt != Type::Jeong {
                            self.errors.push(type_error(*line, *column, lt, rt));
                        }
                        Type::Mental
                    }
                    BinaryOp::Eq | BinaryOp::Ne => Type::Mental,
                    BinaryOp::And | BinaryOp::Or => {
                        if lt != Type::Mental || rt != Type::Mental {
                            self.errors.push(CompileError::semantic(
                                "E003",
                                "? logic requires 멘탈 operands",
                                *line,
                                *column,
                            ));
                        }
                        Type::Mental
                    }
                }
            }
            Expr::Group { inner, .. } => self.check_expr(inner),
            Expr::Call {
                name,
                args,
                line,
                column,
            } => {
                let fname = name.name();
                let func = self.functions.get(&fname).cloned();
                match func {
                    Some(info) => {
                        if info.params.len() != args.len()
                            && fname != "행정 정신"
                            && fname != "행정 뼝"
                        {
                            self.errors.push(CompileError::semantic(
                                "E010",
                                format!(
                                    "function '{fname}' expects {} arguments, got {}",
                                    info.params.len(),
                                    args.len()
                                ),
                                *line,
                                *column,
                            ));
                        }
                        for arg in args {
                            self.check_expr(arg);
                        }
                        info.return_type
                    }
                    None => {
                        self.errors.push(CompileError::semantic(
                            "E012",
                            format!("undefined function '{fname}'"),
                            *line,
                            *column,
                        ));
                        Type::Void
                    }
                }
            }
        }
    }
}

fn types_compatible(expected: Type, actual: Type) -> bool {
    if expected == actual {
        return true;
    }
    if expected == Type::Void || actual == Type::Void {
        return true;
    }
    false
}

fn type_error(line: usize, column: usize, a: Type, b: Type) -> CompileError {
    CompileError::semantic(
        "E003",
        format!("type mismatch: {} vs {}", a.name_ko(), b.name_ko()),
        line,
        column,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn analyze_source(src: &str) -> Result<()> {
        let tokens = Lexer::new(src).tokenize()?;
        let program = Parser::new(tokens).parse()?;
        SemanticAnalyzer::analyze(&program)
    }

    #[test]
    fn implicit_assign_ok() {
        assert!(analyze_source("뼝 정신 뼝 원 !").is_ok());
    }

    #[test]
    fn valid_program() {
        assert!(analyze_source("행정 정신 뼝 ! 뼝 정신 뼝 원 !").is_ok());
    }
}
