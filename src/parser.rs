use crate::lexer::{Token, TokenKind};
use std::iter::Peekable;

#[derive(Debug)]
enum Expr {
    I32Number(i32),
    String(String),
}

#[derive(Debug)]
enum Type {
    String,
    I32,
}

#[derive(Debug)]
struct Params {
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

fn parse_expression<T: Iterator<Item = Token>>(lexer: &mut T) -> Option<Expr> {
    if let Some(token) = lexer.next() {
        match token.kind {
            TokenKind::Number(num) => Some(Expr::I32Number(
                num.parse::<i32>().expect("Could not parse i32 number"),
            )),
            TokenKind::String(str) => Some(Expr::String(str)),
            _ => {
                eprintln!("Could not parse {:?} as an expression", token);
                std::process::exit(1);
            }
        }
    } else {
        None
    }
}

fn get_and_expect<T: Iterator<Item = Token>>(token_kind: TokenKind, lexer: &mut T) {
    if let Some(token) = lexer.next() {
        if token.kind != token_kind {
            eprintln!(
                "Expected token of kind {:?} but found {:?}",
                token_kind, token.kind
            );
            std::process::exit(1);
        }
    } else {
        eprintln!("Expected token of kind {:?} but found nothing", token_kind);
        std::process::exit(1);
    }
}

fn get_and_expect_ident<T: Iterator<Item = Token>>(lexer: &mut T) -> String {
    if let Some(token) = lexer.next() {
        match token.kind {
            TokenKind::Ident(name) => name,
            _ => {
                eprintln!("Expected identifier but found: {:?}", token.kind);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Expected identifier but found end of input");
        std::process::exit(1);
    }
}

#[allow(unreachable_code)]
fn parse_type<T: Iterator<Item = Token>>(lexer: &mut T) -> Type {
    if let Some(token) = lexer.next() {
        match token.kind {
            TokenKind::I32 => Type::I32,
            _ => {
                eprintln!("Unknown type {:?}", token);
                std::process::exit(1);
                unreachable!();
            }
        }
    } else {
        eprintln!("Expected a type but found end of input");
        std::process::exit(1);
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
                    name: name,
                    return_type: return_type,
                    stmts: stmts,
                    params: Vec::new(),
                });
            }
            TokenKind::Return => {
                let expr = parse_expression(lexer);
                get_and_expect(TokenKind::Semicolon, lexer);
                statements.push(Stmts::Return(expr));
            }
            _ => {
                eprintln!("Unexpected token {:?}", token);
                std::process::exit(1);
            }
        }
    }
    return statements;
}
