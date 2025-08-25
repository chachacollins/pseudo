use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashMap;
use std::iter::Peekable;

#[derive(Debug)]
pub enum Expr {
    I32Number(i32),
    U32Number(u32),
    String(String),
    SubprogramCall {
        name: String,
        args: Vec<Expr>,
        return_type: Type,
    },
}
impl Expr {
    pub fn get_type(&self) -> Type {
        match self {
            Expr::I32Number(_) => Type::Int,
            Expr::U32Number(_) => Type::Nat,
            Expr::String(_) => Type::String,
            Expr::SubprogramCall { return_type, .. } => return_type.clone(),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::I32Number(n) => write!(f, "{n}"),
            Expr::U32Number(n) => write!(f, "{n}"),
            Expr::String(s) => write!(f, "\"{s}\""),
            Expr::SubprogramCall { name, args, .. } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Nat,
    String,
    Int,
    Unknown,
}

#[derive(Debug)]
pub struct Param {
    pub param_type: Type,
    pub name: String,
}

#[derive(Debug)]
pub enum Stmts {
    Write(Expr),
    Return(Expr),
    SubProgramDef {
        name: String,
        return_type: Type,
        params: Vec<Param>,
        stmts: Vec<Stmts>,
    },
}

macro_rules! compiler_error {
    ($token:expr, $error_msg:expr) => {
        eprintln!(
            "{}:{}:{}: \x1b[31merror:\x1b[0m {}",
            $token.filename, $token.row, $token.column, $error_msg
        );
        std::process::exit(1);
    };
}

struct SubProgCtx {
    arity: u8,
    return_type: Type,
}

struct ParserCtx {
    is_subprogram: bool,
    expected_type: Type,
    subprogram_table: HashMap<String, SubProgCtx>,
}

pub struct Parser {
    lexer: Peekable<Lexer>,
    curr_token: Option<Token>,
    ctx: Option<ParserCtx>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.peekable(),
            curr_token: None,
            ctx: None,
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmts> {
        self.parse_statements()
    }

    fn ctx_mut(&mut self) -> &mut ParserCtx {
        self.ctx.get_or_insert_with(|| ParserCtx {
            is_subprogram: false,
            expected_type: Type::Unknown,
            subprogram_table: HashMap::new(),
        })
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
            compiler_error!(
                self.curr_token(),
                format!("expected {} but found eof", token_kind)
            );
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
            compiler_error!(self.curr_token(), "expected identifier but found eof");
        }
    }

    fn parse_expression(&mut self) -> Expr {
        if let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Number(ref num) => {
                    let num = num
                        .parse::<i128>()
                        .expect("Could not parse into i128 number");
                    if self.ctx_mut().expected_type == Type::Int {
                        Expr::I32Number(num as i32)
                    } else if self.ctx_mut().expected_type == Type::Nat {
                        Expr::U32Number(num as u32)
                    } else {
                        compiler_error!(
                            token,
                            format!("could not parse {} as an i32 number", token.kind)
                        );
                    }
                }
                TokenKind::String(str) => Expr::String(str),
                TokenKind::Ident(ref name) => {
                    if self.ctx_mut().subprogram_table.contains_key(name) {
                        let return_type = self
                            .ctx_mut()
                            .subprogram_table
                            .get(name)
                            .unwrap()
                            .return_type
                            .clone();

                        self.get_and_expect(TokenKind::LParen);
                        let mut args = Vec::new();
                        while let Some(token) = self.lexer.peek() {
                            if token.kind == TokenKind::RParen {
                                break;
                            }
                            let expr = self.parse_expression();
                            args.push(expr)
                        }
                        self.get_and_expect(TokenKind::RParen);
                        Expr::SubprogramCall {
                            name: name.to_string(),
                            args,
                            return_type,
                        }
                    } else {
                        compiler_error!(self.curr_token(), format!("Unknown {}", token.kind));
                    }
                }
                _ => {
                    compiler_error!(
                        token,
                        format!("could not parse {} as an expression", token.kind)
                    );
                }
            }
        } else {
            compiler_error!(self.curr_token(), "expected expression but found none");
        }
    }

    #[allow(unreachable_code)]
    fn parse_type(&mut self) -> Type {
        if let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Int => Type::Int,
                TokenKind::Nat => Type::Nat,
                _ => {
                    compiler_error!(token, format!("unknown type \"{}\"", token.kind));
                    unreachable!();
                }
            }
        } else {
            compiler_error!(self.curr_token(), "expected a type but found eof");
            unreachable!();
        }
    }

    fn parse_return_stmt(&mut self) -> Stmts {
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::Semicolon);
        if self.ctx_mut().expected_type != expr.get_type() {
            compiler_error!(
                self.curr_token().clone(),
                format!(
                    "return type mismatch: expected {:?}, found {:?}",
                    self.ctx_mut().expected_type,
                    expr.get_type()
                )
            );
        }
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
        if self.ctx_mut().is_subprogram {
            compiler_error!(
                self.curr_token(),
                "cannot define a function within another subprogram"
            );
        }
        self.ctx_mut().is_subprogram = true;
        let name = self.get_and_expect_ident();

        if self.ctx_mut().subprogram_table.contains_key(&name) {
            compiler_error!(
                self.curr_token(),
                format!("redefinition of function {}", name)
            );
        }

        let is_main_func = name == "main";

        self.get_and_expect(TokenKind::LParen);
        let params = self.parse_params();
        self.get_and_expect(TokenKind::RParen);

        if is_main_func && !params.is_empty() {
            compiler_error!(
                self.curr_token(),
                "main function doesn't take any parameters"
            );
        }

        self.get_and_expect(TokenKind::Colon);
        let return_type = self.parse_type();

        if is_main_func && return_type != Type::Int {
            compiler_error!(self.curr_token(), "main function MUST return an integer");
        }

        self.ctx_mut().expected_type = return_type.clone();

        self.get_and_expect(TokenKind::Start);
        let stmts = self.parse_statements();
        self.get_and_expect(TokenKind::Stop);
        let has_return = stmts.iter().any(|stmt| matches!(stmt, Stmts::Return(_)));
        if !has_return {
            compiler_error!(self.curr_token(), "each function must have a return value");
        }
        self.ctx_mut().is_subprogram = false;
        self.ctx_mut().subprogram_table.insert(
            name.clone(),
            SubProgCtx {
                arity: params.len() as u8,
                return_type: return_type.clone(),
            },
        );
        Stmts::SubProgramDef {
            name,
            return_type,
            stmts,
            params,
        }
    }

    fn curr_token(&self) -> &Token {
        self.curr_token
            .as_ref()
            .expect("There should be a valid token in here always")
    }

    fn parse_statements(&mut self) -> Vec<Stmts> {
        let mut statements = Vec::new();
        while let Some(token) = self.lexer.peek() {
            if matches!(token.kind, TokenKind::Stop) {
                break;
            }
            let token = self.lexer.next().unwrap();
            self.curr_token = Some(token);
            match self.curr_token().kind {
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
                    compiler_error!(
                        self.curr_token(),
                        format!("unexpected token {}", self.curr_token().kind)
                    );
                }
            }
        }
        statements
    }
}
