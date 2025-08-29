use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    //Symbols
    Colon,
    Comma,
    LParen,
    RParen,
    Semicolon,
    Equal,
    EqualEqual,
    NotEqual,
    Not,
    Minus,
    Plus,
    Slash,
    Percent,
    Star,

    //Keywords
    Func,
    Proc,
    Start,
    Stop,
    Write,
    Return,
    And,
    If,
    Else,
    End,
    Then,
    Or,

    //Types
    Int,
    Nat,

    Number(String),
    String(String),
    Ident(String),
    Eof,
    Illegal(char),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Func => write!(f, "func"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Proc => write!(f, "proc"),
            TokenKind::Start => write!(f, "start"),
            TokenKind::Stop => write!(f, "stop"),
            TokenKind::Write => write!(f, "write"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Nat => write!(f, "nat"),
            TokenKind::Number(num) => write!(f, "number \"{num}\""),
            TokenKind::String(string) => write!(f, "string \"{string}\""),
            TokenKind::Ident(string) => write!(f, "identifier \"{string}\""),
            TokenKind::Eof => write!(f, "eof"),
            TokenKind::Illegal(tok) => write!(f, "illegal {tok}"),
        }
    }
}

#[derive(Debug, Clone)]
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

    fn peek_next(&self) -> char {
        if self.read_pos >= self.source.len() {
            return '\0';
        }
        self.source[self.read_pos + 1]
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
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' {
                            let _ = self.advance();
                        }
                    } else {
                        break;
                    }
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
            "proc" => TokenKind::Proc,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "or" => TokenKind::Or,
            "and" => TokenKind::And,
            "then" => TokenKind::Then,
            "end" => TokenKind::End,
            "int" => TokenKind::Int,
            "nat" => TokenKind::Nat,
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
            '/' => self.make_token(TokenKind::Slash),
            '*' => self.make_token(TokenKind::Star),
            '%' => self.make_token(TokenKind::Percent),
            ',' => self.make_token(TokenKind::Comma),
            '(' => self.make_token(TokenKind::LParen),
            ')' => self.make_token(TokenKind::RParen),
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.make_token(TokenKind::NotEqual)
                } else {
                    self.make_token(TokenKind::Not)
                }
            }
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.make_token(TokenKind::EqualEqual)
                } else {
                    self.make_token(TokenKind::Equal)
                }
            }
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
            c => self.make_token(TokenKind::Illegal(c)),
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
