use anyhow::Result;
use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};
use thiserror::Error;

use crate::token::{Token, TokenType};

const DELIMITERS: [char; 8] = [',', ':', '(', ')', '{', '}', '[', ']'];

#[derive(Debug, PartialEq, Clone, Error)]
pub enum LexError {
    #[error("Unexpected character: {0} at {1}:{2}")]
    UnexpectedChar(char, usize, usize),
    #[error("Unexpected end of file")]
    UnexpectedEOF,
}

/// Lexes a string into tokens.
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    chars: Peekable<Enumerate<Chars<'a>>>,
    current: Option<char>,
    line: usize,
    col: usize,
    saved_line: usize,
    saved_col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().enumerate().peekable();
        Self {
            chars,
            current: None,
            line: 1,
            col: 0,
            saved_line: 1,
            saved_col: 0,
        }
    }

    /// Creates a new Token from a TokenType
    fn new_token(&mut self, ty: TokenType) -> Token {
        Token {
            ty,
            line: self.saved_line,
            col: self.saved_col,
        }
    }

    /// Creates a new UnexpectedChar error with the current line and column.
    /// If the current character is None, returns an UnexpectedEOF error.
    fn unexpected_char(&self) -> LexError {
        match self.current {
            Some(c) => LexError::UnexpectedChar(c, self.line, self.col),
            None => LexError::UnexpectedEOF,
        }
    }

    /// Saves the current line and column.
    fn save(&mut self) {
        self.saved_line = self.line;
        self.saved_col = self.col;
    }

    /// Gets the current character in the input. Consumes the character if the
    /// current character is None.
    fn current_char(&mut self) -> Option<char> {
        if self.current.is_none() {
            return self.next_char();
        }
        self.current
    }

    /// Gets the next character in the input. Consumes the character. Increments
    /// the line and column numbers accordingly. Assumes newlines are \n.
    fn next_char(&mut self) -> Option<char> {
        let (_, char) = match self.chars.next() {
            Some(it) => it,
            None => {
                self.current = None;
                return None;
            }
        };
        if char == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        self.current = Some(char);
        self.current
    }

    // Peeks the next character in the input. Does not consume the character.
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    /// Parse the next token from the input.
    fn next_token(&mut self) -> Result<Token> {
        let c = {
            match self.current_char() {
                Some(c) => {
                    if c.is_whitespace() || c == '/' {
                        let mut comment = c == '/' && self.peek_char() == Some('/');
                        loop {
                            if comment {
                                match self.next_char() {
                                    Some('\n') => {
                                        comment = false;
                                        continue;
                                    }
                                    Some(_) => continue,
                                    None => return Ok(self.new_token(TokenType::Eof)),
                                }
                            }
                            match self.next_char() {
                                Some(c) => {
                                    if !c.is_whitespace() {
                                        if c == '/' && self.peek_char() == Some('/') {
                                            comment = true;
                                            continue;
                                        }
                                        break c;
                                    }
                                }
                                None => return Ok(self.new_token(TokenType::Eof)),
                            }
                        }
                    } else {
                        c
                    }
                }
                None => return Ok(self.new_token(TokenType::Eof)),
            }
        };

        let token = match c {
            c if DELIMITERS.contains(&c) => {
                self.save();
                self.next_char();
                match c {
                    ',' => self.new_token(TokenType::Comma),
                    ':' => self.new_token(TokenType::Colon),
                    '(' => self.new_token(TokenType::LParen),
                    ')' => self.new_token(TokenType::RParen),
                    '{' => self.new_token(TokenType::LBrace),
                    '}' => self.new_token(TokenType::RBrace),
                    '[' => self.new_token(TokenType::LBracket),
                    ']' => self.new_token(TokenType::RBracket),
                    _ => unreachable!(),
                }
            }
            '"' => self.parse_string()?,
            c if c.is_digit(10) || c == '.' => self.parse_number().ok_or(self.unexpected_char())?,
            c if c == '-' => {
                let next = self.peek_char();
                let Some(next) = next else {
                    return Err(self.unexpected_char().into());
                };
                if next.is_digit(10) || next == '.' {
                    self.parse_number().ok_or(self.unexpected_char())?
                } else {
                    self.parse_ident()?
                }
            }
            _ => self.parse_ident()?,
        };

        Ok(token)
    }

    /// Parse a string. Assumes the first character is a double quote.
    /// Escaped characters will be unescaped (e.g. \" will be parsed as ").
    fn parse_string(&mut self) -> Result<Token> {
        self.save();
        let mut string = String::new();

        loop {
            match self.next_char() {
                Some('"') => {
                    self.next_char();
                    break;
                }
                Some('\\') => {
                    let c = self.next_char().ok_or(LexError::UnexpectedEOF)?;
                    string.push(match c {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\x08',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\n' => {
                            self.line += 1;
                            self.col = 0;
                            continue;
                        }
                        'u' => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                let c = self.next_char().ok_or(LexError::UnexpectedEOF)?;
                                hex.push(c);
                            }
                            u32::from_str_radix(&hex, 16)
                                .map_err(|_| self.unexpected_char())?
                                .try_into()
                                .map_err(|_| self.unexpected_char())?
                        }
                        _ => return Err(self.unexpected_char().into()),
                    });
                }
                Some(c) => string.push(c),
                None => return Err(LexError::UnexpectedEOF.into()),
            }
        }

        Ok(self.new_token(TokenType::String(string)))
    }

    /// Parse a number. Assumes the first character is a digit.
    /// I'm lazy so this doesn't support scientific notation or hex numbers.
    fn parse_number(&mut self) -> Option<Token> {
        self.save();
        let mut number = String::new();
        let mut found_dot = false;

        number.push(self.current.unwrap());

        loop {
            if let Some(c) = self.next_char() {
                if c.is_digit(10) {
                    number.push(c);
                    continue;
                } else if c == '.' {
                    if found_dot {
                        return None;
                    }
                    found_dot = true;
                    number.push(c);
                    continue;
                } else if c == '_' {
                    number.push(c);
                    continue;
                } else if c.is_whitespace() || DELIMITERS.contains(&c) {
                    break;
                } else {
                    return None;
                }
            }
            break;
        }

        if number.contains('.') {
            number
                .parse::<f64>()
                .map(TokenType::Float)
                .map(|t| self.new_token(t))
                .ok()
        } else {
            number
                .parse::<i64>()
                .map(TokenType::Int)
                .map(|t| self.new_token(t))
                .ok()
        }
    }

    /// Parse an identifier.
    fn parse_ident(&mut self) -> Result<Token> {
        self.save();
        let mut ident = String::new();
        let mut params = None;
        ident.push(self.current.unwrap());

        loop {
            match self.next_char() {
                Some(c) if !c.is_whitespace() && !DELIMITERS.contains(&c) => ident.push(c),
                Some(_) => {
                    break;
                }
                None => break,
            }
        }

        if self.current_char() == Some('[') {
            let mut p = Vec::new();
            self.next_char();
            loop {
                match self.next_token() {
                    Ok(Token {
                        ty: TokenType::RBracket,
                        ..
                    }) => break,
                    Ok(t) => p.push(t),
                    Err(e) => return Err(e.into()),
                }
            }
            params = Some(p);
        }

        Ok(self.new_token(TokenType::new_ident(ident.as_str(), params)?))
    }

    /// Parse all tokens from the input.
    pub fn parse(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            match self.next_token() {
                Ok(Token {
                    ty: TokenType::Eof, ..
                }) => break,
                Ok(token) => tokens.push(token),
                Err(e) => return Err(e.into()),
            }
        }

        Ok(tokens)
    }
}
