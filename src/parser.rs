use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::{CompileError, Result};
use crate::token::{Identifier, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Program> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek_kind(&self) -> TokenKind {
        self.current().kind
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<Token> {
        if self.peek_kind() == kind {
            Ok(self.advance())
        } else {
            Err(CompileError::parser(
                format!("{msg}, got '{}'", self.current().lexeme),
                self.current(),
            ))
        }
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        let line = self.current().line;
        let column = self.current().column;

        match self.peek_kind() {
            TokenKind::Haengjeong => self.parse_haengjeong_stmt(),
            TokenKind::Ppyung => self.parse_assign_stmt(),
            TokenKind::Question => self.parse_question_stmt(),
            TokenKind::Ubeom => self.parse_loop_stmt(),
            TokenKind::Ipwon => self.parse_ipwon_stmt(),
            TokenKind::Boho => self.parse_try_stmt(),
            TokenKind::Jeongbyeong => self.parse_comment_stmt(),
            _ => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::Bang, "expected '!' after expression")?;
                Ok(Stmt::Expr {
                    expr,
                    line,
                    column,
                })
            }
        }
    }

    fn parse_haengjeong_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();
        let line = start.line;
        let column = start.column;

        if self.match_kind(TokenKind::Jeongsin) {
            let name = self.parse_identifier()?;
            self.expect(TokenKind::Bang, "expected '!' after declaration")?;
            return Ok(Stmt::Decl { name, line, column });
        }

        if self.match_kind(TokenKind::Bang) {
            let value = self.parse_expression()?;
            self.expect(TokenKind::Bang, "expected closing '!' after output")?;
            return Ok(Stmt::Output { value, line, column });
        }

        if self.match_kind(TokenKind::Ppyung) {
            let value = self.parse_expression()?;
            self.expect(TokenKind::Bang, "expected '!' after return")?;
            return Ok(Stmt::Return { value, line, column });
        }

        if self.match_kind(TokenKind::Jeongbyeong) {
            let value = self.parse_expression()?;
            self.expect(TokenKind::Bang, "expected '!' after throw")?;
            return Ok(Stmt::Throw { value, line, column });
        }

        Err(CompileError::parser(
            "expected 정신, !, 뼝, or 정병 after 행정",
            self.current(),
        ))
    }

    fn parse_assign_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();
        self.expect(TokenKind::Jeongsin, "expected 정신 after 뼝")?;
        let name = self.parse_identifier()?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Bang, "expected '!' after assignment")?;
        Ok(Stmt::Assign {
            name,
            value,
            line: start.line,
            column: start.column,
        })
    }

    fn parse_question_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();

        if self.peek_kind() == TokenKind::Haengjeong {
            self.advance();
            self.expect(TokenKind::Jeongsin, "expected 정신 after ? 행정")?;
            let name = self.parse_identifier()?;
            self.expect(TokenKind::Bang, "expected '!' after input")?;
            return Ok(Stmt::Input {
                name,
                line: start.line,
                column: start.column,
            });
        }

        let condition = self.parse_expression()?;
        self.expect(TokenKind::Menhera, "expected 멘헤라 after if condition")?;
        self.expect(TokenKind::Jaui, "expected 자의 branch")?;
        let then_body = self.parse_block_statements_until(&[TokenKind::Dongui])?;
        self.expect(TokenKind::Dongui, "expected 동의 for else branch")?;
        let else_body = self.parse_block_statements_until(&[TokenKind::Ppyung])?;
        self.expect(TokenKind::Ppyung, "expected 뼝 to close if")?;

        Ok(Stmt::If {
            condition,
            then_body,
            else_body,
            line: start.line,
            column: start.column,
        })
    }

    fn parse_loop_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();

        if self.match_kind(TokenKind::Question) {
            let condition = self.parse_expression()?;
            self.expect(TokenKind::Menhera, "expected 멘헤라 after while condition")?;
            let body = self.parse_block_statements_until(&[TokenKind::Ppyung])?;
            self.expect(TokenKind::Ppyung, "expected 뼝 to close while")?;
            return Ok(Stmt::While {
                condition,
                body,
                line: start.line,
                column: start.column,
            });
        }

        self.expect(TokenKind::Sonyeon, "expected 소년 after 우범 for for-loop")?;
        let var = self.parse_identifier()?;
        let start_expr = self.parse_expression()?;
        let end_expr = self.parse_expression()?;
        self.expect(TokenKind::Menhera, "expected 멘헤라 after for range")?;
        let body = self.parse_block_statements_until(&[TokenKind::Ppyung])?;
        self.expect(TokenKind::Ppyung, "expected 뼝 to close for")?;

        Ok(Stmt::For {
            var,
            start: start_expr,
            end: end_expr,
            body,
            line: start.line,
            column: start.column,
        })
    }

    fn parse_ipwon_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();
        let name = self.parse_identifier()?;

        if self.peek_kind() == TokenKind::Sonyeon || self.is_function_body_start() {
            let mut params = Vec::new();
            while self.peek_kind() == TokenKind::Sonyeon {
                self.advance();
                params.push(self.parse_identifier()?);
            }
            self.expect(TokenKind::Menhera, "expected 멘헤라 to start function body")?;
            let body = self.parse_block_statements_until(&[TokenKind::Ppyung])?;
            self.expect(TokenKind::Ppyung, "expected 뼝 to close function")?;
            return Ok(Stmt::FuncDef {
                name,
                params,
                body,
                line: start.line,
                column: start.column,
            });
        }

        let mut args = Vec::new();
        if self.is_call_arg_start() {
            args.push(self.parse_additive()?);
        }
        self.expect(TokenKind::Bang, "expected '!' after call")?;
        Ok(Stmt::Expr {
            expr: Expr::Call {
                name,
                args,
                line: start.line,
                column: start.column,
            },
            line: start.line,
            column: start.column,
        })
    }

    fn parse_try_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();
        self.expect(TokenKind::Menhera, "expected 멘헤라 after 보호")?;
        let try_body = self.parse_block_statements_until(&[TokenKind::Jeongbyeong])?;
        self.expect(TokenKind::Jeongbyeong, "expected 정병 for catch")?;
        self.expect(TokenKind::Menhera, "expected 멘헤라 after catch 정병")?;
        let catch_body = self.parse_block_statements_until(&[TokenKind::Ppyung])?;
        self.expect(TokenKind::Ppyung, "expected 뼝 to close try")?;
        Ok(Stmt::Try {
            try_body,
            catch_body,
            line: start.line,
            column: start.column,
        })
    }

    fn parse_comment_stmt(&mut self) -> Result<Stmt> {
        let start = self.advance();
        let mut tokens = Vec::new();
        while self.peek_kind() != TokenKind::Bang && !self.is_at_end() {
            tokens.push(self.advance().lexeme);
        }
        self.expect(TokenKind::Bang, "expected '!' to end comment")?;
        Ok(Stmt::Comment {
            tokens,
            line: start.line,
            column: start.column,
        })
    }

    fn parse_block_statements_until(&mut self, terminators: &[TokenKind]) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.is_block_terminator(terminators) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn is_block_terminator(&self, terminators: &[TokenKind]) -> bool {
        if !terminators.contains(&TokenKind::Ppyung) {
            return terminators.contains(&self.peek_kind());
        }
        if self.peek_kind() != TokenKind::Ppyung {
            return terminators.contains(&self.peek_kind());
        }
        // `뼝 정신` begins assignment, not block termination.
        self.tokens
            .get(self.pos + 1)
            .map(|t| t.kind)
            .is_some_and(|k| k != TokenKind::Jeongsin)
    }

    fn parse_identifier(&mut self) -> Result<Identifier> {
        let line = self.current().line;
        let column = self.current().column;
        let mut parts = Vec::new();

        if !self.current().kind.is_identifier_part() {
            return Err(CompileError::parser("expected identifier", self.current()));
        }

        parts.push(self.advance().kind);

        while self.can_continue_identifier(self.current().kind) {
            parts.push(self.advance().kind);
        }

        Ok(Identifier {
            parts,
            span_line: line,
            span_column: column,
        })
    }

    fn can_continue_identifier(&self, kind: TokenKind) -> bool {
        if !kind.is_identifier_part() {
            return false;
        }
        !matches!(
            kind,
            TokenKind::Jeongsin
                | TokenKind::Won
                | TokenKind::Menhera
                | TokenKind::Jeoeongsin
                | TokenKind::Ipwon
                | TokenKind::Bang
                | TokenKind::Question
                | TokenKind::Haengjeong
                | TokenKind::Ppyung
                | TokenKind::Ubeom
                | TokenKind::Boho
                | TokenKind::Jeongbyeong
                | TokenKind::Sonyeon
                | TokenKind::Jaui
                | TokenKind::Dongui
        )
    }

    fn is_function_body_start(&self) -> bool {
        if self.peek_kind() != TokenKind::Menhera {
            return false;
        }
        match self.tokens.get(self.pos + 1).map(|t| t.kind) {
            Some(TokenKind::Jeongsin) | Some(TokenKind::Ppyung) => false,
            Some(TokenKind::Ipwon) | Some(TokenKind::Boho) => false,
            Some(
                TokenKind::Haengjeong
                | TokenKind::Question
                | TokenKind::Ubeom
                | TokenKind::Jeongbyeong
                | TokenKind::Jaui
                | TokenKind::Dongui
                | TokenKind::Sonyeon,
            ) => true,
            _ => true,
        }
    }

    fn is_int_literal_after_menhera(&self) -> bool {
        let mut idx = self.pos;
        while idx < self.tokens.len() {
            match self.tokens[idx].kind {
                TokenKind::Jeongsin | TokenKind::Ppyung => {
                    idx += 1;
                }
                TokenKind::Menhera => {
                    return idx > self.pos;
                }
                _ => return false,
            }
        }
        false
    }

    fn is_call_arg_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Jeongsin
                | TokenKind::Menhera
                | TokenKind::Won
                | TokenKind::Jeoeongsin
                | TokenKind::Ipwon
                | TokenKind::Bang
        )
    }

    fn should_end_call_args(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Ppyung | TokenKind::Won | TokenKind::Question | TokenKind::Bang
        ) || (self.peek_kind() == TokenKind::Jeongsin && !self.is_var_ref_start())
            || self.is_comparison_menhera()
    }

    fn is_comparison_menhera(&self) -> bool {
        if self.peek_kind() != TokenKind::Menhera {
            return false;
        }
        matches!(
            self.tokens.get(self.pos + 1).map(|t| t.kind),
            Some(TokenKind::Ubeom) | Some(TokenKind::Sonyeon) | Some(TokenKind::Jaui) | Some(TokenKind::Dongui)
        )
    }

    fn is_statement_start(&self, kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Haengjeong
                | TokenKind::Ppyung
                | TokenKind::Question
                | TokenKind::Ubeom
                | TokenKind::Ipwon
                | TokenKind::Boho
                | TokenKind::Jeongbyeong
                | TokenKind::Menhera
                | TokenKind::Jaui
                | TokenKind::Dongui
                | TokenKind::Bang
                | TokenKind::Eof
        )
    }

    fn is_expression_boundary(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Bang
                | TokenKind::Menhera
                | TokenKind::Jaui
                | TokenKind::Dongui
                | TokenKind::Ppyung
                | TokenKind::Jeongbyeong
                | TokenKind::Eof
        )
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.match_kind(TokenKind::Question) {
            let op_token = self.tokens[self.pos - 1].clone();
            let mid = self.parse_and()?;
            if self.match_kind(TokenKind::Dongui) {
                let right = self.parse_and()?;
                let or_expr = Expr::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(mid),
                    right: Box::new(right),
                    line: op_token.line,
                    column: op_token.column,
                };
                left = Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(or_expr),
                    line: op_token.line,
                    column: op_token.column,
                };
            } else {
                left = Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(mid),
                    line: op_token.line,
                    column: op_token.column,
                };
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.peek_kind() != TokenKind::Menhera {
                break;
            }
            let saved = self.pos;
            self.advance();
            if self.match_kind(TokenKind::Jaui) {
                if self.is_branch_statement_start() {
                    self.pos = saved;
                    break;
                }
                let op_token = self.tokens[saved].clone();
                let right = self.parse_comparison()?;
                left = Expr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                    line: op_token.line,
                    column: op_token.column,
                };
            } else if self.match_kind(TokenKind::Dongui) {
                if self.is_branch_statement_start() {
                    self.pos = saved;
                    break;
                }
                let op_token = self.tokens[saved].clone();
                let right = self.parse_comparison()?;
                left = Expr::Binary {
                    op: BinaryOp::Ne,
                    left: Box::new(left),
                    right: Box::new(right),
                    line: op_token.line,
                    column: op_token.column,
                };
            } else {
                self.pos = saved;
                break;
            }
        }
        Ok(left)
    }

    fn is_branch_statement_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Haengjeong
                | TokenKind::Ppyung
                | TokenKind::Question
                | TokenKind::Ipwon
                | TokenKind::Ubeom
                | TokenKind::Boho
                | TokenKind::Jeongbyeong
                | TokenKind::Dongui
        )
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;
        loop {
            if self.match_kind(TokenKind::Menhera) {
                let op_token = self.current().clone();
                if self.match_kind(TokenKind::Ubeom) {
                    let right = self.parse_additive()?;
                    left = Expr::Binary {
                        op: BinaryOp::Lt,
                        left: Box::new(left),
                        right: Box::new(right),
                        line: op_token.line,
                        column: op_token.column,
                    };
                } else if self.match_kind(TokenKind::Sonyeon) {
                    let right = self.parse_additive()?;
                    left = Expr::Binary {
                        op: BinaryOp::Gt,
                        left: Box::new(left),
                        right: Box::new(right),
                        line: op_token.line,
                        column: op_token.column,
                    };
                } else {
                    self.pos -= 1;
                    break;
                }
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            match self.peek_kind() {
                TokenKind::Ppyung => {
                    let op_token = self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                        line: op_token.line,
                        column: op_token.column,
                    };
                }
                TokenKind::Jeongsin if !self.is_var_ref_start() => {
                    let op_token = self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::Binary {
                        op: BinaryOp::Sub,
                        left: Box::new(left),
                        right: Box::new(right),
                        line: op_token.line,
                        column: op_token.column,
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn is_var_ref_start(&self) -> bool {
        if self.peek_kind() != TokenKind::Jeongsin {
            return false;
        }
        self.tokens
            .get(self.pos + 1)
            .map(|t| t.kind.is_identifier_part())
            .unwrap_or(false)
            && !matches!(
                self.tokens.get(self.pos + 1).map(|t| t.kind),
                Some(TokenKind::Menhera)
            )
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        while self.peek_kind() == TokenKind::Won {
            let op_token = self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op: BinaryOp::Mul,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_kind(TokenKind::Bang) {
            let op_token = self.tokens[self.pos - 1].clone();
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                line: op_token.line,
                column: op_token.column,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.current().clone();
        let line = token.line;
        let column = token.column;

        if self.match_kind(TokenKind::Jeongsin) {
            let name = self.parse_identifier()?;
            return Ok(Expr::Var { name, line, column });
        }

        if self.match_kind(TokenKind::Menhera) {
            if self.peek_kind() == TokenKind::Ipwon
                && self.tokens.get(self.pos + 1).map(|t| t.kind) == Some(TokenKind::Menhera)
            {
                self.advance();
                self.advance();
                return Ok(Expr::Bool {
                    value: true,
                    line,
                    column,
                });
            }
            if self.peek_kind() == TokenKind::Boho
                && self.tokens.get(self.pos + 1).map(|t| t.kind) == Some(TokenKind::Menhera)
            {
                self.advance();
                self.advance();
                return Ok(Expr::Bool {
                    value: false,
                    line,
                    column,
                });
            }
            if self.is_int_literal_after_menhera() {
                return self.parse_int_literal(line, column);
            }
            let inner = self.parse_expression()?;
            self.expect(TokenKind::Menhera, "expected closing 멘헤라 for group")?;
            return Ok(Expr::Group {
                inner: Box::new(inner),
                line,
                column,
            });
        }

        if self.match_kind(TokenKind::Won) {
            let mut count = 1i64;
            while self.match_kind(TokenKind::Won) {
                count += 1;
            }
            return Ok(Expr::Int {
                value: count,
                line,
                column,
            });
        }

        if self.match_kind(TokenKind::Jeoeongsin) {
            return self.parse_string_literal(line, column);
        }

        if self.match_kind(TokenKind::Ipwon) {
            let name = self.parse_identifier()?;
            let mut args = Vec::new();
            if self.is_call_arg_start() {
                args.push(self.parse_additive()?);
            }
            return Ok(Expr::Call {
                name,
                args,
                line,
                column,
            });
        }

        Err(CompileError::parser("expected expression", self.current()))
    }

    fn parse_int_literal(&mut self, line: usize, column: usize) -> Result<Expr> {
        let mut bits = String::new();
        while self.peek_kind() == TokenKind::Jeongsin || self.peek_kind() == TokenKind::Ppyung {
            bits.push(match self.advance().kind {
                TokenKind::Jeongsin => '0',
                TokenKind::Ppyung => '1',
                _ => unreachable!(),
            });
        }
        self.expect(TokenKind::Menhera, "expected closing 멘헤라 for int literal")?;
        if bits.is_empty() {
            return Ok(Expr::Int {
                value: 0,
                line,
                column,
            });
        }
        let value = i64::from_str_radix(&bits, 2).map_err(|_| {
            CompileError::parser("invalid binary integer literal", self.current())
        })?;
        Ok(Expr::Int {
            value,
            line,
            column,
        })
    }

    fn parse_string_literal(&mut self, line: usize, column: usize) -> Result<Expr> {
        let mut bits = String::new();
        while self.peek_kind() == TokenKind::Jeongsin || self.peek_kind() == TokenKind::Ppyung {
            bits.push(match self.advance().kind {
                TokenKind::Jeongsin => '0',
                TokenKind::Ppyung => '1',
                _ => unreachable!(),
            });
        }
        self.expect(TokenKind::Jeoeongsin, "expected closing 저엉신저엉신")?;

        if !bits.len().is_multiple_of(8) {
            return Err(CompileError::parser(
                "string literal bits must be multiple of 8",
                self.current(),
            ));
        }

        let mut bytes = Vec::new();
        for chunk in bits.as_bytes().chunks(8) {
            let byte_str = std::str::from_utf8(chunk).unwrap();
            let byte = u8::from_str_radix(byte_str, 2).map_err(|_| {
                CompileError::parser("invalid byte in string literal", self.current())
            })?;
            bytes.push(byte);
        }
        let value = String::from_utf8(bytes).map_err(|_| {
            CompileError::parser("invalid UTF-8 in string literal", self.current())
        })?;

        Ok(Expr::String {
            value,
            line,
            column,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_ok(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize().unwrap();
        Parser::new(tokens).parse().unwrap()
    }

    #[test]
    fn parse_decl_and_assign() {
        let program = parse_ok("행정 정신 뼝 ! 뼝 정신 뼝 원 원 !");
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn parse_if() {
        let src = "? 멘헤라 입원 멘헤라 멘헤라 자의 행정 ! 원 ! 동의 행정 ! 원 원 ! 뼝";
        let program = parse_ok(src);
        assert!(matches!(program.statements[0], Stmt::If { .. }));
    }

    #[test]
    fn parse_function() {
        let src = "입원 뼝 소년 정신 멘헤라 행정 뼝 정신 정신 원 원 ! 뼝";
        let program = parse_ok(src);
        assert!(matches!(program.statements[0], Stmt::FuncDef { .. }));
    }
}
