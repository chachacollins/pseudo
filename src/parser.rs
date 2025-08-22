use crate::lexer::{Token, TokenKind};
use std::iter::Peekable;

#[derive(Debug)]
pub enum Expr {
    I32Number(i32),
    String(String),
}

#[derive(Debug)]
pub enum Type {
    String,
    I32,
}

#[derive(Debug)]
pub struct Params {
    param_type: Type,
    name: String,
}

#[derive(Debug)]
pub enum Stmts {
    Write(Option<Expr>),
    Return(Option<Expr>),
    SubProgramDef {
        name: String,
        return_type: Type,
        params: Vec<Params>,
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

fn parse_expression<T: Iterator<Item = Token>>(lexer: &mut T) -> Option<Expr> {
    if let Some(token) = lexer.next() {
        match token.kind {
            TokenKind::Number(num) => Some(Expr::I32Number(
                num.parse::<i32>().expect("Could not parse i32 number"),
            )),
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

fn get_and_expect<T: Iterator<Item = Token>>(token_kind: TokenKind, lexer: &mut T) {
    if let Some(token) = lexer.next() {
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

fn get_and_expect_ident<T: Iterator<Item = Token>>(lexer: &mut T) -> String {
    if let Some(token) = lexer.next() {
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

#[allow(unreachable_code)]
fn parse_type<T: Iterator<Item = Token>>(lexer: &mut T) -> Type {
    if let Some(token) = lexer.next() {
        match token.kind {
            TokenKind::I32 => Type::I32,
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

pub fn parse_statements<T: Iterator<Item = Token>>(lexer: &mut Peekable<T>) -> Vec<Stmts> {
    let mut statements = Vec::new();
    while let Some(token) = lexer.peek() {
        if matches!(token.kind, TokenKind::Stop) {
            break;
        }
        let token = lexer.next().unwrap();
        match token.kind {
            TokenKind::Write => {
                get_and_expect(TokenKind::LParen, lexer);
                let expr = parse_expression(lexer);
                get_and_expect(TokenKind::RParen, lexer);
                get_and_expect(TokenKind::Semicolon, lexer);
                statements.push(Stmts::Write(expr));
            }
            TokenKind::Func => {
                let name = get_and_expect_ident(lexer);
                get_and_expect(TokenKind::LParen, lexer);
                get_and_expect(TokenKind::RParen, lexer);
                get_and_expect(TokenKind::Colon, lexer);
                let return_type = parse_type(lexer);
                get_and_expect(TokenKind::Start, lexer);
                let stmts = parse_statements(lexer);
                get_and_expect(TokenKind::Stop, lexer);
                statements.push(Stmts::SubProgramDef {
                    name,
                    return_type,
                    stmts,
                    params: Vec::new(),
                });
            }
            TokenKind::Return => {
                let expr = parse_expression(lexer);
                get_and_expect(TokenKind::Semicolon, lexer);
                statements.push(Stmts::Return(expr));
            }
            _ => {
                compiler_error!(token, format!("unexpected token {}", token.kind));
            }
        }
    }
    statements
}
