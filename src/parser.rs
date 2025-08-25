use crate::lexer::{Lexer, TokenKind};
use std::iter::Peekable;

#[derive(Debug)]
pub enum Expr {
    I32Number(i32),
    U32Number(u32),
    String(String),
}

#[derive(Debug)]
pub enum Type {
    Nat,
    String,
    Int,
}

#[derive(Debug)]
pub struct Param {
    pub param_type: Type,
    pub name: String,
}

#[derive(Debug)]
pub enum Stmts {
    Write(Option<Expr>),
    Return(Option<Expr>),
    SubProgramDef {
        name: String,
        return_type: Type,
        params: Vec<Param>,
        stmts: Vec<Stmts>,
    },
}

macro_rules! compiler_error {
    ($error_msg:expr) => {
        eprintln!("\x1b[31merror:\x1b[0m {}", $error_msg);
        std::process::exit(1);
    };
    ($token:expr, $error_msg:expr) => {
        eprintln!(
            "{}:{}:{}: \x1b[31merror:\x1b[0m {}",
            $token.filename, $token.row, $token.column, $error_msg
        );
        std::process::exit(1);
    };
}

pub struct Parser {
    lexer: Peekable<Lexer>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmts> {
        self.parse_statements()
    }

    fn get_and_expect(&mut self, token_kind: TokenKind) {
        if let Some(token) = self.lexer.next() {
            if token.kind != token_kind {
                compiler_error!(
                    token,
                    format!("expected {} but found {}", token_kind, token.kind)
                );
            }
        } else {
            compiler_error!(format!("expected {} but found eof", token_kind));
        }
    }

    fn get_and_expect_ident(&mut self) -> String {
        if let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Ident(name) => name,
                _ => {
                    compiler_error!(
                        token,
                        format!("expected identifier but found {}", token.kind)
                    );
                }
            }
        } else {
            compiler_error!("expected identifier but found eof");
        }
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        if let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Number(ref num) => {
                    let num = num
                        .parse::<i128>()
                        .expect("Could not parse into i128 number");
                    if num >= i32::MIN as i128 && num <= i32::MAX as i128 {
                        Some(Expr::I32Number(num as i32))
                    } else {
                        compiler_error!(
                            token,
                            format!("could not parse {} as an i32 number", token.kind)
                        );
                    }
                }
                TokenKind::String(str) => Some(Expr::String(str)),
                _ => {
                    compiler_error!(
                        token,
                        format!("could not parse {} as an expression", token.kind)
                    );
                }
            }
        } else {
            None
        }
    }

    #[allow(unreachable_code)]
    fn parse_type(&mut self) -> Type {
        if let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Int => Type::Int,
                _ => {
                    compiler_error!(token, format!("unknown type \"{}\"", token.kind));
                    unreachable!();
                }
            }
        } else {
            compiler_error!("expected a type but found eof");
            unreachable!();
        }
    }

    fn parse_return_stmt(&mut self) -> Stmts {
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::Semicolon);
        Stmts::Return(expr)
    }

    fn parse_write_stmt(&mut self) -> Stmts {
        self.get_and_expect(TokenKind::LParen);
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::RParen);
        self.get_and_expect(TokenKind::Semicolon);
        Stmts::Write(expr)
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while let Some(token) = self.lexer.peek() {
            match token.kind {
                TokenKind::RParen => break,
                TokenKind::Ident(_) => {
                    let name = self.get_and_expect_ident();
                    self.get_and_expect(TokenKind::Colon);
                    let param_type = self.parse_type();
                    params.push(Param { name, param_type });
                }
                TokenKind::Comma => {
                    self.get_and_expect(TokenKind::Comma);
                    continue;
                }
                _ => {
                    compiler_error!(
                        token,
                        format!("unexpect token {} in function parameters", token.kind)
                    );
                }
            }
        }
        params
    }

    fn parse_func_stmt(&mut self) -> Stmts {
        let name = self.get_and_expect_ident();

        self.get_and_expect(TokenKind::LParen);
        let params = self.parse_params();
        self.get_and_expect(TokenKind::RParen);

        if name == "main" && !params.is_empty() {
            compiler_error!("main function doesn't take any parameters");
        }

        self.get_and_expect(TokenKind::Colon);
        let return_type = self.parse_type();

        self.get_and_expect(TokenKind::Start);
        let stmts = self.parse_statements();
        self.get_and_expect(TokenKind::Stop);

        Stmts::SubProgramDef {
            name,
            return_type,
            stmts,
            params,
        }
    }

    fn parse_statements(&mut self) -> Vec<Stmts> {
        let mut statements = Vec::new();
        while let Some(token) = self.lexer.peek() {
            if matches!(token.kind, TokenKind::Stop) {
                break;
            }
            let token = self.lexer.next().unwrap();
            match token.kind {
                TokenKind::Write => {
                    statements.push(self.parse_write_stmt());
                }
                TokenKind::Func => {
                    statements.push(self.parse_func_stmt());
                }
                TokenKind::Return => {
                    statements.push(self.parse_return_stmt());
                }
                _ => {
                    compiler_error!(token, format!("unexpected token {}", token.kind));
                }
            }
        }
        statements
    }
}
