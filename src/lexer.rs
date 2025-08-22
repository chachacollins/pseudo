use std::fmt;

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Colon,
    LParen,
    RParen,
    Semicolon,
    Func,
    Start,
    Stop,
    Write,
    Return,
    I32,
    Number(String),
    String(String),
    Ident(String),
    Eof,
    Illegal,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Colon => write!(f, ":"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Func => write!(f, "func"),
            TokenKind::Start => write!(f, "start"),
            TokenKind::Stop => write!(f, "stop"),
            TokenKind::Write => write!(f, "write"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::Number(num) => write!(f, "number \"{num}\""),
            TokenKind::String(string) => write!(f, "string \"{string}\""),
            TokenKind::Ident(string) => write!(f, "identifier \"{string}\""),
            TokenKind::Eof => write!(f, "eof"),
            TokenKind::Illegal => write!(f, "illegal"),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub filename: String,
    pub column: usize,
    pub row: usize,
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    read_pos: usize,
    row: usize,
    column: usize,
    filename: String,
}

impl Lexer {
    pub fn new(filename: String, source: String) -> Self {
        Self {
            source: source.chars().collect(),
            column: 0,
            row: 1,
            pos: 0,
            read_pos: 0,
            filename,
        }
    }

    fn peek(&self) -> char {
        if self.read_pos >= self.source.len() {
            return '\0';
        }
        self.source[self.read_pos]
    }

    fn advance(&mut self) -> char {
        if self.read_pos >= self.source.len() {
            return '\0';
        }
        self.column += 1;
        self.pos = self.read_pos;
        self.read_pos += 1;
        self.source[self.pos]
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                '\t' | ' ' | '\r' => {
                    let _ = self.advance();
                }
                '\n' => {
                    self.column = 0;
                    self.row += 1;
                    let _ = self.advance();
                }
                _ => break,
            }
        }
    }

    fn make_token(&self, token_kind: TokenKind) -> Token {
        Token {
            kind: token_kind,
            column: self.column,
            row: self.row,
            filename: self.filename.clone(),
        }
    }

    fn classify_ident(&self, ident: &str) -> TokenKind {
        match ident {
            "start" => TokenKind::Start,
            "stop" => TokenKind::Stop,
            "func" => TokenKind::Func,
            "i32" => TokenKind::I32,
            "write" => TokenKind::Write,
            "return" => TokenKind::Return,
            _ => TokenKind::Ident(ident.to_string()),
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let c = self.advance();
        match c {
            ':' => self.make_token(TokenKind::Colon),
            ';' => self.make_token(TokenKind::Semicolon),
            '(' => self.make_token(TokenKind::LParen),
            ')' => self.make_token(TokenKind::RParen),
            '0'..='9' => {
                let mut num = String::new();
                num.push(c);
                while self.peek().is_ascii_digit() {
                    num.push(self.advance());
                }
                self.make_token(TokenKind::Number(num))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                ident.push(c);
                while self.peek().is_alphanumeric() || self.peek() == '_' {
                    ident.push(self.advance());
                }
                self.make_token(self.classify_ident(&ident))
            }
            '\0' => self.make_token(TokenKind::Eof),
            '"' => {
                let mut string = String::new();
                while self.peek() != '"' {
                    string.push(self.advance());
                }
                let _ = self.advance();
                self.make_token(TokenKind::String(string))
            }
            _ => self.make_token(TokenKind::Illegal),
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let next_token = self.next_token();
        if next_token.kind == TokenKind::Eof {
            None
        } else {
            Some(next_token)
        }
    }
}
