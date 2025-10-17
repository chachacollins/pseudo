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
    Walrus,

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
    Set,
    Mut,
    True,
    False,

    //Types
    Int,
    Nat,
    Str,
    Bool,

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
            TokenKind::Walrus => write!(f, ":="),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
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
            TokenKind::Set => write!(f, "set"),
            TokenKind::Write => write!(f, "write"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Nat => write!(f, "nat"),
            TokenKind::Str => write!(f, "string"),
            TokenKind::Bool => write!(f, "bool"),
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

    fn make_token(&self, token_kind: TokenKind, start_row: usize, start_col: usize) -> Token {
        Token {
            kind: token_kind,
            column: start_col,
            row: start_row,
            filename: self.filename.clone(),
        }
    }

    fn classify_ident(&self, ident: &str) -> TokenKind {
        match ident {
            "start" => TokenKind::Start,
            "stop" => TokenKind::Stop,
            "string" => TokenKind::Str,
            "set" => TokenKind::Set,
            "mut" => TokenKind::Mut,
            "func" => TokenKind::Func,
            "proc" => TokenKind::Proc,
            "if" => TokenKind::If,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "else" => TokenKind::Else,
            "or" => TokenKind::Or,
            "and" => TokenKind::And,
            "then" => TokenKind::Then,
            "end" => TokenKind::End,
            "int" => TokenKind::Int,
            "nat" => TokenKind::Nat,
            "bool" => TokenKind::Bool,
            "write" => TokenKind::Write,
            "return" => TokenKind::Return,
            _ => TokenKind::Ident(ident.to_string()),
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let c = self.advance();
        match c {
            ':' => {
                if self.peek() == '=' {
                    self.advance();
                    self.make_token(TokenKind::Walrus, self.row, self.column)
                } else {
                    self.make_token(TokenKind::Colon, self.row, self.column)
                }
            }
            ';' => self.make_token(TokenKind::Semicolon, self.row, self.column),
            '/' => self.make_token(TokenKind::Slash, self.row, self.column),
            '*' => self.make_token(TokenKind::Star, self.row, self.column),
            '%' => self.make_token(TokenKind::Percent, self.row, self.column),
            ',' => self.make_token(TokenKind::Comma, self.row, self.column),
            '(' => self.make_token(TokenKind::LParen, self.row, self.column),
            ')' => self.make_token(TokenKind::RParen, self.row, self.column),
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.make_token(TokenKind::NotEqual, self.row, self.column)
                } else {
                    self.make_token(TokenKind::Not, self.row, self.column)
                }
            }
            '-' => self.make_token(TokenKind::Minus, self.row, self.column),
            '+' => self.make_token(TokenKind::Plus, self.row, self.column),
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.make_token(TokenKind::EqualEqual, self.row, self.column)
                } else {
                    self.make_token(TokenKind::Equal, self.row, self.column)
                }
            }
            '0'..='9' => {
                let start_row = self.row;
                let start_col = self.column - 1; // Already consumed the first num
                let mut num = String::new();
                num.push(c);
                while self.peek().is_ascii_digit() {
                    num.push(self.advance());
                }
                self.make_token(TokenKind::Number(num), start_row, start_col)
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_row = self.row;
                let start_col = self.column;
                let mut ident = String::new();
                ident.push(c);
                while self.peek().is_alphanumeric() || self.peek() == '_' {
                    ident.push(self.advance());
                }
                self.make_token(self.classify_ident(&ident), start_row, start_col)
            }
            '\0' => self.make_token(TokenKind::Eof, self.row, self.column),
            '"' => {
                let start_row = self.row;
                let start_col = self.column - 1; //This is because i'm skipping the quotes
                let mut string = String::new();
                while self.peek() != '"' {
                    string.push(self.advance());
                }
                let _ = self.advance();
                self.make_token(TokenKind::String(string), start_row, start_col)
            }
            c => self.make_token(TokenKind::Illegal(c), self.row, self.column),
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
