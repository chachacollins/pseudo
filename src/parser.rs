use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashMap;
use std::iter::Peekable;

#[derive(Debug)]
pub enum Expr {
    I32Number(i32),
    U32Number(u32),
    String(String),
    Variable {
        name: String,
        var_type: Type,
    },
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
            Expr::Variable { var_type, .. } => *var_type,
            Expr::SubprogramCall { return_type, .. } => *return_type,
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::I32Number(n) => write!(f, "{n}"),
            Expr::U32Number(n) => write!(f, "{n}"),
            Expr::String(s) => write!(f, "\"{s}\""),
            Expr::Variable { name, .. } => write!(f, "{name}"),
            Expr::SubprogramCall { name, args, .. } => {
                write!(f, "{name}(")?;
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    Nat,
    String,
    Int,
    Void,
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
    SubProgramCall {
        name: String,
        args: Vec<Expr>,
        return_type: Type,
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

struct VarCtx {
    var_type: Type,
    mutable: bool,
}

struct ParserCtx {
    is_subprogram: bool,
    expected_type: Type,
    subprogram_table: HashMap<String, SubProgCtx>,
    local_var_table: HashMap<String, VarCtx>,
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
            local_var_table: HashMap::new(),
        })
    }

    fn curr_token(&self) -> &Token {
        self.curr_token
            .as_ref()
            .expect("There should be a valid token in here always")
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
                    let num = num.parse::<i128>().unwrap_or_else(|err| {
                        compiler_error!(
                            token,
                            format!(
                                "could not parse {} as a {:?} number because {err}",
                                token.kind,
                                self.ctx_mut().expected_type
                            )
                        );
                    });

                    if self.ctx_mut().expected_type == Type::Int {
                        Expr::I32Number(num as i32)
                    } else if self.ctx_mut().expected_type == Type::Nat {
                        Expr::U32Number(num as u32)
                    } else if self.ctx_mut().expected_type == Type::Unknown {
                        if num >= i32::MIN as i128 && num <= i32::MAX as i128 {
                            Expr::I32Number(num as i32)
                        } else if num >= u32::MIN as i128 && num <= u32::MAX as i128 {
                            Expr::U32Number(num as u32)
                        } else {
                            compiler_error!(
                                token,
                                format!("could not determine type of {} number", token.kind)
                            );
                        }
                    } else {
                        compiler_error!(
                            token,
                            format!(
                                "could not parse {} as a {:?} number",
                                token.kind,
                                self.ctx_mut().expected_type
                            )
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
                            .return_type;

                        self.get_and_expect(TokenKind::LParen);
                        let args = self.parse_subprog_args();
                        self.get_and_expect(TokenKind::RParen);
                        Expr::SubprogramCall {
                            name: name.to_string(),
                            args,
                            return_type,
                        }
                    } else if self.ctx_mut().local_var_table.contains_key(name) {
                        let var_type = self.ctx_mut().local_var_table.get(name).unwrap().var_type;
                        Expr::Variable {
                            name: name.to_string(),
                            var_type,
                        }
                    } else {
                        compiler_error!(self.curr_token(), format!("unknown {}", token.kind));
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
        let curr_type = self.ctx_mut().expected_type;
        self.ctx_mut().expected_type = Type::Unknown;
        let expr = self.parse_expression();
        self.ctx_mut().expected_type = curr_type;
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
                    params.push(Param {
                        name: name.clone(),
                        param_type,
                    });
                    self.ctx_mut().local_var_table.insert(
                        name,
                        VarCtx {
                            var_type: param_type,
                            mutable: false,
                        },
                    );
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

        self.ctx_mut().expected_type = return_type;

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
                return_type,
            },
        );
        self.ctx_mut().local_var_table.clear();
        Stmts::SubProgramDef {
            name,
            return_type,
            stmts,
            params,
        }
    }

    fn parse_subprog_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        while let Some(token) = self.lexer.peek() {
            if token.kind == TokenKind::RParen {
                break;
            } else if token.kind == TokenKind::Comma {
                self.get_and_expect(TokenKind::Comma);
                continue;
            }
            let expr = self.parse_expression();
            args.push(expr)
        }
        args
    }

    fn parse_proc_stmt(&mut self) -> Stmts {
        if self.ctx_mut().is_subprogram {
            compiler_error!(
                self.curr_token(),
                "cannot define a procedure within another subprogram"
            );
        }
        self.ctx_mut().is_subprogram = true;
        let name = self.get_and_expect_ident();

        if self.ctx_mut().subprogram_table.contains_key(&name) {
            compiler_error!(
                self.curr_token(),
                format!("redefinition of subprogram {}", name)
            );
        }

        if name == "main" {
            compiler_error!(
                self.curr_token(),
                "main MUST be a function which returns an integer"
            );
        }

        self.get_and_expect(TokenKind::LParen);
        let params = self.parse_params();
        self.get_and_expect(TokenKind::RParen);

        self.ctx_mut().expected_type = Type::Void;

        self.get_and_expect(TokenKind::Start);
        let stmts = self.parse_statements();
        self.get_and_expect(TokenKind::Stop);
        self.ctx_mut().is_subprogram = false;
        self.ctx_mut().subprogram_table.insert(
            name.clone(),
            SubProgCtx {
                arity: params.len() as u8,
                return_type: Type::Void,
            },
        );
        self.ctx_mut().local_var_table.clear();
        Stmts::SubProgramDef {
            name,
            return_type: Type::Void,
            stmts,
            params,
        }
    }

    fn parse_subprogcall_stmt(&mut self) -> Stmts {
        let name = match self.curr_token().kind {
            TokenKind::Ident(ref name) => name.clone(),
            _ => unreachable!(),
        };
        if self.ctx_mut().subprogram_table.contains_key(&name) {
            let return_type = self
                .ctx_mut()
                .subprogram_table
                .get(&name)
                .unwrap()
                .return_type;

            self.get_and_expect(TokenKind::LParen);
            let args = self.parse_subprog_args();
            self.get_and_expect(TokenKind::RParen);
            self.get_and_expect(TokenKind::Semicolon);
            Stmts::SubProgramCall {
                name: name.to_string(),
                args,
                return_type,
            }
        } else {
            compiler_error!(
                self.curr_token(),
                format!("Unknown {}", self.curr_token().kind)
            );
        }
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
                TokenKind::Write => statements.push(self.parse_write_stmt()),
                TokenKind::Func => statements.push(self.parse_func_stmt()),
                TokenKind::Proc => statements.push(self.parse_proc_stmt()),
                TokenKind::Return => statements.push(self.parse_return_stmt()),
                TokenKind::Ident(_) => {
                    if let Some(token) = self.lexer.peek() {
                        match token.kind {
                            TokenKind::LParen => statements.push(self.parse_subprogcall_stmt()),
                            _ => {
                                compiler_error!(
                                    self.curr_token(),
                                    format!("free standing {}", self.curr_token().kind)
                                );
                            }
                        }
                    }
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
