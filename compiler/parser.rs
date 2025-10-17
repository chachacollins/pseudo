use crate::lexer::{Lexer, Token, TokenKind};
use std::iter::Peekable;

//TODO: add all binary operators
//TODO: use let some thing
//TODO: Typecheck 1: binary ops 2: return statements 3: func arguements
//TODO: Improve error messages
//TODO: separate ast from parser

#[derive(Debug)]
pub enum Op {
    Equal,
    NotEqual,
    Or,
    Add,
    Minus,
    Div,
    Mult,
    Mod,
    And,
}

impl Op {
    pub fn from(token_kind: TokenKind) -> Self {
        match token_kind {
            TokenKind::EqualEqual => Op::Equal,
            TokenKind::NotEqual => Op::NotEqual,
            TokenKind::Or => Op::Or,
            TokenKind::Minus => Op::Minus,
            TokenKind::Plus => Op::Add,
            TokenKind::Star => Op::Mult,
            TokenKind::Slash => Op::Div,
            TokenKind::And => Op::And,
            TokenKind::Percent => Op::Mod,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Number(i128),
    String(String),
    Variable(String),
    Bool(bool),
    SubprogramCall {
        name: String,
        args: Vec<AstNode<Expr>>,
    },
    Binary {
        op: Op,
        lhs: Box<AstNode<Expr>>,
        rhs: Box<AstNode<Expr>>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    Nat,
    String,
    Int,
    Bool,
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
    Write {
        type_: Type, //Filled by sem analysis
        expr: AstNode<Expr>,
    },
    Return {
        return_type: Type, //Filled by sem analysis
        expr: AstNode<Expr>,
    },
    Set {
        name: String,
        var_type: Type,
        mutable: bool,
        expr: AstNode<Expr>,
    },
    Assign {
        name: String,
        expr: AstNode<Expr>,
    },
    SubProgramDef {
        name: String,
        return_type: Type,
        params: Vec<Param>,
        stmts: Vec<AstNode<Stmts>>,
    },
    If {
        expr: AstNode<Expr>,
        stmts: Vec<AstNode<Stmts>>,
    },
    Else(Vec<AstNode<Stmts>>),
    SubProgramCall {
        name: String,
        args: Vec<AstNode<Expr>>,
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

#[derive(Debug, Clone)]
pub struct Position {
    pub filename: String,
    pub column: usize,
    pub row: usize,
}

impl Position {
    fn from(token: &Token) -> Position {
        Position {
            filename: token.filename.clone(),
            column: token.column,
            row: token.row,
        }
    }
}

#[derive(Debug)]
pub struct AstNode<T> {
    pub value: T,
    pub position: Position,
}

pub struct Parser {
    lexer: Peekable<Lexer>,
    curr_token: Option<Token>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer: lexer.peekable(),
            curr_token: None,
        }
    }

    pub fn parse_program(&mut self) -> Vec<AstNode<Stmts>> {
        self.parse_statements()
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
            } else {
                self.curr_token = Some(token);
            }
        } else {
            compiler_error!(
                self.curr_token(),
                format!("expected {} but found eof", token_kind)
            );
        }
    }

    fn get_maybe(&mut self, token_kind: TokenKind) -> bool {
        if let Some(token) = self.lexer.peek() {
            if token.kind != token_kind {
                return false;
            } else {
                self.get_and_expect(token_kind);
                return true;
            }
        } else {
            compiler_error!(
                self.curr_token(),
                format!("expected {} but found eof", token_kind)
            );
        }
    }

    fn get_and_return_ident(&mut self) -> String {
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

    //TODO: ADD PRECEDENCE OF SOME KIND
    fn parse_expression(&mut self) -> AstNode<Expr> {
        if let Some(token) = self.lexer.next() {
            let mut lhs = match token.kind {
                TokenKind::Number(ref num) => {
                    let num = num.parse::<i128>().unwrap_or_else(|err| {
                        compiler_error!(
                            token,
                            format!("could not parse {} as a  number because {err}", token.kind,)
                        );
                    });
                    AstNode {
                        value: Expr::Number(num),
                        position: Position::from(&token),
                    }
                }
                TokenKind::String(ref str) => AstNode {
                    value: Expr::String(str.clone()),
                    position: Position::from(&token),
                },
                TokenKind::True => AstNode {
                    value: Expr::Bool(true),
                    position: Position::from(&token),
                },
                TokenKind::False => AstNode {
                    value: Expr::Bool(false),
                    position: Position::from(&token),
                },
                TokenKind::Ident(ref name) => {
                    if let Some(token) = self.lexer.peek() {
                        if token.kind == TokenKind::LParen {
                            let position = Position::from(&token);
                            self.get_and_expect(TokenKind::LParen);
                            let args = self.parse_subprog_args();
                            self.get_and_expect(TokenKind::RParen);
                            AstNode {
                                value: Expr::SubprogramCall {
                                    name: name.to_string(),
                                    args,
                                },
                                position,
                            }
                        } else {
                            AstNode {
                                value: Expr::Variable(name.clone()),
                                position: Position::from(&token),
                            }
                        }
                    } else {
                        AstNode {
                            value: Expr::Variable(name.clone()),
                            position: Position::from(&token),
                        }
                    }
                }
                _ => {
                    compiler_error!(
                        token,
                        format!("could not parse {} as an expression", token.kind)
                    );
                }
            };
            while let Some(token) = self.lexer.peek() {
                match token.kind {
                    TokenKind::Equal
                    | TokenKind::EqualEqual
                    | TokenKind::NotEqual
                    | TokenKind::Or
                    | TokenKind::And
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::Percent => {
                        let position = Position::from(&token);
                        let tok = self.lexer.next().unwrap();
                        let op = Op::from(tok.kind);
                        let rhs = self.parse_expression();
                        lhs = AstNode {
                            value: Expr::Binary {
                                op,
                                lhs: Box::new(lhs),
                                rhs: Box::new(rhs),
                            },
                            position,
                        }
                    }
                    _ => break,
                }
            }
            lhs
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
                TokenKind::Str => Type::String,
                TokenKind::Bool => Type::Bool,
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
        Stmts::Return {
            return_type: Type::Unknown,
            expr,
        }
    }

    fn parse_if_stmt(&mut self) -> Stmts {
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::Then);
        let stmts = self.parse_statements();
        self.get_and_expect(TokenKind::End);
        Stmts::If { expr, stmts }
    }

    //TODO: Ensure it is within an if block
    fn parse_else_stmt(&mut self) -> Stmts {
        let stmts = self.parse_statements();
        Stmts::Else(stmts)
    }

    fn parse_write_stmt(&mut self) -> Stmts {
        self.get_and_expect(TokenKind::LParen);
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::RParen);
        self.get_and_expect(TokenKind::Semicolon);
        Stmts::Write {
            type_: Type::Unknown,
            expr,
        }
    }

    fn parse_set_stmt(&mut self) -> Stmts {
        let mut mutable = false;
        let mut var_type = Type::Unknown;
        if self.get_maybe(TokenKind::Mut) {
            mutable = true;
        }
        let name = self.get_and_return_ident();
        if !self.get_maybe(TokenKind::Walrus) {
            self.get_and_expect(TokenKind::Colon);
            var_type = self.parse_type();
            self.get_and_expect(TokenKind::Equal);
        }
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::Semicolon);

        Stmts::Set {
            name,
            var_type,
            expr,
            mutable,
        }
    }
    fn parse_varassign_stmt(&mut self) -> Stmts {
        let name = match self.curr_token().kind {
            TokenKind::Ident(ref name) => name.clone(),
            _ => unreachable!(),
        };
        self.get_and_expect(TokenKind::Equal);
        let expr = self.parse_expression();
        self.get_and_expect(TokenKind::Semicolon);
        Stmts::Assign { name, expr }
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while let Some(token) = self.lexer.peek() {
            match token.kind {
                TokenKind::RParen => break,
                TokenKind::Ident(_) => {
                    let name = self.get_and_return_ident();
                    self.get_and_expect(TokenKind::Colon);
                    let param_type = self.parse_type();
                    params.push(Param {
                        name: name.clone(),
                        param_type,
                    });
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
        let name = self.get_and_return_ident();
        self.get_and_expect(TokenKind::LParen);
        let params = self.parse_params();
        self.get_and_expect(TokenKind::RParen);
        self.get_and_expect(TokenKind::Colon);
        let return_type = self.parse_type();
        self.get_and_expect(TokenKind::Start);
        let stmts = self.parse_statements();
        self.get_and_expect(TokenKind::Stop);
        //TODO: Check return value
        // let has_return = stmts.iter().any(|stmt| matches!(stmt, Stmts::Return(_)));
        // if !has_return {
        //     compiler_error!(self.curr_token(), "each function must have a return value");
        // }
        Stmts::SubProgramDef {
            name,
            return_type,
            stmts,
            params,
        }
    }

    fn parse_subprog_args(&mut self) -> Vec<AstNode<Expr>> {
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
        let name = self.get_and_return_ident();
        self.get_and_expect(TokenKind::LParen);
        let params = self.parse_params();
        self.get_and_expect(TokenKind::RParen);
        self.get_and_expect(TokenKind::Start);
        let stmts = self.parse_statements();
        self.get_and_expect(TokenKind::Stop);
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
        self.get_and_expect(TokenKind::LParen);
        let args = self.parse_subprog_args();
        self.get_and_expect(TokenKind::RParen);
        self.get_and_expect(TokenKind::Semicolon);
        Stmts::SubProgramCall {
            name: name.to_string(),
            args,
        }
    }

    fn parse_statements(&mut self) -> Vec<AstNode<Stmts>> {
        let mut statements = Vec::new();
        while let Some(token) = self.lexer.peek() {
            if token.kind == TokenKind::Stop || token.kind == TokenKind::End {
                break;
            }
            let token = self.lexer.next().unwrap();
            self.curr_token = Some(token);
            match self.curr_token().kind {
                TokenKind::Write => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_write_stmt(),
                        position,
                    });
                }
                TokenKind::Func => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_func_stmt(),
                        position,
                    });
                }
                TokenKind::Proc => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_proc_stmt(),
                        position,
                    });
                }
                TokenKind::Return => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_return_stmt(),
                        position,
                    });
                }
                TokenKind::If => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_if_stmt(),
                        position,
                    });
                }
                TokenKind::Else => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_else_stmt(),
                        position,
                    });
                }
                TokenKind::Set => {
                    let position = Position::from(&self.curr_token());
                    statements.push(AstNode {
                        value: self.parse_set_stmt(),
                        position,
                    });
                }
                TokenKind::Ident(_) => {
                    if let Some(token) = self.lexer.peek() {
                        match token.kind {
                            TokenKind::LParen => {
                                let position = Position::from(&token);
                                statements.push(AstNode {
                                    value: self.parse_subprogcall_stmt(),
                                    position,
                                });
                            }
                            TokenKind::Equal => {
                                let position = Position::from(&token);
                                statements.push(AstNode {
                                    value: self.parse_varassign_stmt(),
                                    position,
                                })
                            }
                            //TODO: CHANGE THIS ERROR
                            _ => {
                                compiler_error!(
                                    self.curr_token(),
                                    format!("unknown identifier {}", self.curr_token().kind)
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
