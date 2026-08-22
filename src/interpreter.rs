use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::{CompileError, Result};
use crate::token::Identifier;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Jeong(i64),
    Ppyung(String),
    Mental(bool),
    Void,
    Ipnwon(Rc<RefCell<FunctionValue>>),
    Jeongbyeong(String),
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub name: Identifier,
    pub params: Vec<Identifier>,
    pub body: Vec<Stmt>,
}

#[derive(Clone)]
struct Environment {
    vars: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            vars: HashMap::new(),
            parent: None,
        }))
    }

    fn child(parent: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        None
    }

    fn set(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            return true;
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        false
    }
}

pub struct Interpreter {
    env: Rc<RefCell<Environment>>,
    functions: HashMap<String, Rc<RefCell<FunctionValue>>>,
    output: Vec<String>,
    input_lines: Vec<String>,
    input_index: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            functions: HashMap::new(),
            output: Vec::new(),
            input_lines: Vec::new(),
            input_index: 0,
        }
    }

    pub fn with_input(mut self, lines: Vec<String>) -> Self {
        self.input_lines = lines;
        self
    }

    pub fn run(program: &Program) -> Result<String> {
        let mut interp = Self::new();
        interp.execute(program)?;
        Ok(interp.output.join(""))
    }

    pub fn run_with_input(program: &Program, input: Vec<String>) -> Result<String> {
        let mut interp = Self::new().with_input(input);
        interp.execute(program)?;
        Ok(interp.output.join(""))
    }

    /// HTTP 요청 경로를 입력 한 줄로 전달해 실행 (웹 핸들러용).
    pub fn run_with_request(program: &Program, path: &str) -> Result<String> {
        Self::run_with_input(program, vec![path.to_string()])
    }

    fn execute(&mut self, program: &Program) -> Result<()> {
        for stmt in &program.statements {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ExecResult> {
        match stmt {
            Stmt::Decl { name, .. } => {
                self.env
                    .borrow_mut()
                    .set(&name.name(), Value::Jeong(0));
                Ok(ExecResult::Normal)
            }
            Stmt::Assign { name, value, line, column } => {
                let val = self.eval_expr(value)?;
                if !self.env.borrow_mut().assign(&name.name(), val.clone()) {
                    self.env.borrow_mut().set(&name.name(), val);
                }
                Ok(ExecResult::Normal)
            }
            Stmt::Expr { expr, .. } => {
                self.eval_expr(expr)?;
                Ok(ExecResult::Normal)
            }
            Stmt::Output { value, .. } => {
                let val = self.eval_expr(value)?;
                self.output.push(value_to_string(&val));
                Ok(ExecResult::Normal)
            }
            Stmt::Input { name, line, column } => {
                let line_str = if self.input_index < self.input_lines.len() {
                    let s = self.input_lines[self.input_index].clone();
                    self.input_index += 1;
                    s
                } else {
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    let mut buf = String::new();
                    handle.read_line(&mut buf).map_err(|e| {
                        CompileError::semantic("E030", e.to_string(), *line, *column)
                    })?;
                    buf.trim_end_matches(['\r', '\n']).to_string()
                };
                if !self.env.borrow_mut().assign(&name.name(), Value::Ppyung(line_str)) {
                    return Err(CompileError::semantic(
                        "E001",
                        format!("undefined variable '{}'", name.name()),
                        *line,
                        *column,
                    ));
                }
                Ok(ExecResult::Normal)
            }
            Stmt::Return { value, .. } => {
                let val = self.eval_expr(value)?;
                Ok(ExecResult::Return(val))
            }
            Stmt::Throw { value, .. } => {
                let val = self.eval_expr(value)?;
                Err(runtime_throw(val))
            }
            Stmt::Comment { .. } => Ok(ExecResult::Normal),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond = self.eval_expr(condition)?;
                if self.is_truthy(&cond) {
                    self.exec_block(then_body)
                } else {
                    self.exec_block(else_body)
                }
            }
            Stmt::While { condition, body, .. } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    if !self.is_truthy(&cond) {
                        break;
                    }
                    match self.exec_block(body)? {
                        ExecResult::Return(v) => return Ok(ExecResult::Return(v)),
                        ExecResult::Break => break,
                        ExecResult::Normal => {}
                    }
                }
                Ok(ExecResult::Normal)
            }
            Stmt::For {
                var,
                start,
                end,
                body,
                ..
            } => {
                let s = match self.eval_expr(start)? {
                    Value::Jeong(n) => n,
                    other => return Err(type_runtime_error(other)),
                };
                let e = match self.eval_expr(end)? {
                    Value::Jeong(n) => n,
                    other => return Err(type_runtime_error(other)),
                };
                let child = Environment::child(self.env.clone());
                child
                    .borrow_mut()
                    .set(&var.name(), Value::Jeong(s));
                let prev = self.env.clone();
                self.env = child;
                let mut i = s;
                while i < e {
                    self.env
                        .borrow_mut()
                        .set(&var.name(), Value::Jeong(i));
                    match self.exec_block(body)? {
                        ExecResult::Return(v) => {
                            self.env = prev;
                            return Ok(ExecResult::Return(v));
                        }
                        ExecResult::Break => break,
                        ExecResult::Normal => {}
                    }
                    i += 1;
                }
                self.env = prev;
                Ok(ExecResult::Normal)
            }
            Stmt::FuncDef {
                name,
                params,
                body,
                ..
            } => {
                let func = Rc::new(RefCell::new(FunctionValue {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                }));
                self.functions.insert(name.name(), func);
                Ok(ExecResult::Normal)
            }
            Stmt::Try {
                try_body,
                catch_body,
                ..
            } => match self.exec_block(try_body) {
                Ok(r) => Ok(r),
                Err(_) => self.exec_block(catch_body),
            },
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<ExecResult> {
        let child = Environment::child(self.env.clone());
        let prev = self.env.clone();
        self.env = child;
        for stmt in stmts {
            match self.exec_stmt(stmt)? {
                ExecResult::Return(v) => {
                    self.env = prev;
                    return Ok(ExecResult::Return(v));
                }
                ExecResult::Break => {
                    self.env = prev;
                    return Ok(ExecResult::Break);
                }
                ExecResult::Normal => {}
            }
        }
        self.env = prev;
        Ok(ExecResult::Normal)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Int { value, .. } => Ok(Value::Jeong(*value)),
            Expr::String { value, .. } => Ok(Value::Ppyung(value.clone())),
            Expr::Bool { value, .. } => Ok(Value::Mental(*value)),
            Expr::Var { name, line, column } => self
                .env
                .borrow()
                .get(&name.name())
                .ok_or_else(|| {
                    CompileError::semantic(
                        "E001",
                        format!("undefined variable '{}'", name.name()),
                        *line,
                        *column,
                    )
                }),
            Expr::Unary { op, operand, .. } => {
                let val = self.eval_expr(operand)?;
                match op {
                    UnaryOp::Not => Ok(Value::Mental(!self.is_truthy(&val))),
                }
            }
            Expr::Binary { op, left, right, .. } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                match op {
                    BinaryOp::Add => match (l, r) {
                        (Value::Jeong(a), Value::Jeong(b)) => Ok(Value::Jeong(a + b)),
                        (Value::Ppyung(a), Value::Ppyung(b)) => Ok(Value::Ppyung(format!("{a}{b}"))),
                        (a, b) => Err(type_runtime_error_pair(a, b)),
                    },
                    BinaryOp::Sub => int_binop(l, r, |a, b| a - b),
                    BinaryOp::Mul => int_binop(l, r, |a, b| a * b),
                    BinaryOp::Lt => int_cmp(l, r, |a, b| a < b),
                    BinaryOp::Gt => int_cmp(l, r, |a, b| a > b),
                    BinaryOp::Eq => Ok(Value::Mental(values_equal(&l, &r))),
                    BinaryOp::Ne => Ok(Value::Mental(!values_equal(&l, &r))),
                    BinaryOp::And => {
                        let truth = self.is_truthy(&l);
                        if !truth {
                            Ok(Value::Mental(false))
                        } else {
                            Ok(Value::Mental(self.is_truthy(&r)))
                        }
                    }
                    BinaryOp::Or => {
                        let truth = self.is_truthy(&l);
                        if truth {
                            Ok(Value::Mental(true))
                        } else {
                            Ok(Value::Mental(self.is_truthy(&r)))
                        }
                    }
                }
            }
            Expr::Group { inner, .. } => self.eval_expr(inner),
            Expr::Call {
                name,
                args,
                line,
                column,
            } => self.call_function(name, args, *line, *column),
        }
    }

    fn call_function(
        &mut self,
        name: &Identifier,
        args: &[Expr],
        line: usize,
        column: usize,
    ) -> Result<Value> {
        let fname = name.name();

        if fname == "행정 정신" {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            return Ok(Value::Jeong(now % 100000));
        }
        if fname == "행정 뼝" {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64;
            return Ok(Value::Jeong(seed.abs() % 101));
        }

        let func = self
            .functions
            .get(&fname)
            .cloned()
            .ok_or_else(|| {
                CompileError::semantic(
                    "E012",
                    format!("undefined function '{fname}'"),
                    line,
                    column,
                )
            })?;

        let func_ref = func.borrow();
        if func_ref.params.len() != args.len() {
            return Err(CompileError::semantic(
                "E010",
                format!(
                    "function '{fname}' expects {} arguments, got {}",
                    func_ref.params.len(),
                    args.len()
                ),
                line,
                column,
            ));
        }

        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }

        let child = Environment::child(self.env.clone());
        for (param, val) in func_ref.params.iter().zip(arg_values.iter()) {
            child.borrow_mut().set(&param.name(), val.clone());
        }
        let body = func_ref.body.clone();
        drop(func_ref);

        let prev = self.env.clone();
        self.env = child;
        let mut result = Value::Void;
        for stmt in &body {
            match self.exec_stmt(stmt)? {
                ExecResult::Return(v) => {
                    result = v;
                    break;
                }
                ExecResult::Break | ExecResult::Normal => {}
            }
        }
        self.env = prev;
        Ok(result)
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Jeong(n) => *n != 0,
            Value::Ppyung(s) => !s.is_empty(),
            Value::Mental(b) => *b,
            Value::Void => false,
            Value::Ipnwon(_) => true,
            Value::Jeongbyeong(_) => true,
        }
    }
}

enum ExecResult {
    Normal,
    Return(Value),
    Break,
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Jeong(n) => n.to_string(),
        Value::Ppyung(s) => s.clone(),
        Value::Mental(b) => {
            if *b {
                "멘헤라 입원 멘헤라".to_string()
            } else {
                "멘헤라 보호 멘헤라".to_string()
            }
        }
        Value::Void => String::new(),
        Value::Ipnwon(_) => "<입원함수>".to_string(),
        Value::Jeongbyeong(s) => s.clone(),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Jeong(x), Value::Jeong(y)) => x == y,
        (Value::Ppyung(x), Value::Ppyung(y)) => x == y,
        (Value::Mental(x), Value::Mental(y)) => x == y,
        _ => false,
    }
}

fn int_binop<F>(l: Value, r: Value, f: F) -> Result<Value>
where
    F: FnOnce(i64, i64) -> i64,
{
    match (l, r) {
        (Value::Jeong(a), Value::Jeong(b)) => Ok(Value::Jeong(f(a, b))),
        (a, b) => Err(type_runtime_error_pair(a, b)),
    }
}

fn int_cmp<F>(l: Value, r: Value, f: F) -> Result<Value>
where
    F: FnOnce(i64, i64) -> bool,
{
    match (l, r) {
        (Value::Jeong(a), Value::Jeong(b)) => Ok(Value::Mental(f(a, b))),
        (a, b) => Err(type_runtime_error_pair(a, b)),
    }
}

fn type_runtime_error(v: Value) -> CompileError {
    CompileError::semantic(
        "E003",
        format!("expected 정수, got {:?}", v),
        0,
        0,
    )
}

fn runtime_throw(val: Value) -> CompileError {
    let msg = value_to_string(&val);
    CompileError::semantic("E040", msg, 0, 0)
}

fn type_runtime_error_pair(a: Value, b: Value) -> CompileError {
    CompileError::semantic(
        "E003",
        format!("type mismatch at runtime: {:?} vs {:?}", a, b),
        0,
        0,
    )
}
