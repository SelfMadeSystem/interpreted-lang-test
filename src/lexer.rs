use anyhow::Result;
use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};
use thiserror::Error;

use crate::token::Token;

const DELIMITERS: [char; 7] = [',', '(', ')', '{', '}', '[', ']'];

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
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().enumerate().peekable();
        Self {
            chars,
            current: None,
            line: 0,
            col: 0,
        }
    }

    /// Creates a new UnexpectedChar error with the current line and column.
    /// Assumes the current character is Some.
    fn unexpected_char(&self) -> LexError {
        LexError::UnexpectedChar(self.current.unwrap(), self.line, self.col)
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

    /// Parse the next token from the input.
    fn next_token(&mut self) -> Result<Token> {
        let c = {
            match self.current_char() {
                Some(c) => {
                    if c.is_whitespace() {
                        loop {
                            match self.next_char() {
                                Some(c) => {
                                    if !c.is_whitespace() {
                                        break c;
                                    }
                                }
                                None => return Ok(Token::Eof),
                            }
                        }
                    } else {
                        c
                    }
                }
                None => return Ok(Token::Eof),
            }
        };

        let token = match c {
            c if DELIMITERS.contains(&c) => {
                self.next_char();
                match c {
                    ',' => Token::Comma,
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    '{' => Token::LBrace,
                    '}' => Token::RBrace,
                    '[' => Token::LBracket,
                    ']' => Token::RBracket,
                    _ => unreachable!(),
                }
            }
            '"' => self.parse_string()?,
            c if c.is_digit(10) || c == '.' => self.parse_number().ok_or(self.unexpected_char())?,
            c if c == '-' => {
                let next = self.chars.peek();
                if next.is_none() {
                    return Err(self.unexpected_char().into());
                }
                let (_, next) = next.unwrap();
                if next.is_digit(10) || next == &'.' {
                    self.parse_number().ok_or(self.unexpected_char())?
                } else {
                    self.parse_ident()?
                }
            },
            _ => self.parse_ident()?,
        };

        Ok(token)
    }

    /// Parse a string. Assumes the first character is a double quote.
    /// Escaped characters will be unescaped (e.g. \" will be parsed as ").
    fn parse_string(&mut self) -> Result<Token> {
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

        Ok(Token::String(string))
    }

    /// Parse a number. Assumes the first character is a digit.
    /// I'm lazy so this doesn't support scientific notation or hex numbers.
    fn parse_number(&mut self) -> Option<Token> {
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
                .map(Token::Float)
                .ok()
        } else {
            number
                .parse::<i64>()
                .map(Token::Int)
                .ok()
        }
    }

    /// Parse an identifier. Accepts any character that is not whitespace or a
    /// delimiter.
    fn parse_ident(&mut self) -> Result<Token> {
        let mut ident = String::new();
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

        Ok(Token::new_ident(ident.as_str()))
    }

    /// Parse all tokens from the input.
    pub fn parse(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            match self.next_token() {
                Ok(Token::Eof) => break,
                Ok(token) => tokens.push(token),
                Err(e) => return Err(e.into()),
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut lexer = Lexer::new(r#""Hello, world!\nNext line\u0420""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("Hello, world!\nNext line\u{0420}".to_string())
        );

        let mut lexer = Lexer::new(r#""Invalid escape sequence: \z""#);
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_parse_number() {
        let mut lexer = Lexer::new("1234");
        assert_eq!(lexer.next_token().unwrap(), Token::Int(1234));

        let mut lexer = Lexer::new("1234.5678");
        assert_eq!(lexer.next_token().unwrap(), Token::Float(1234.5678));

        let mut lexer = Lexer::new("1234.");
        assert_eq!(lexer.next_token().unwrap(), Token::Float(1234.0));

        let mut lexer = Lexer::new("1234.5678.91011");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_parse_ident() {
        let mut lexer = Lexer::new("hello");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Ident("hello".to_string())
        );

        let mut lexer = Lexer::new("hello world");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Ident("hello".to_string())
        );

        let mut lexer = Lexer::new("hello, world");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Ident("hello".to_string())
        );

        let mut lexer = Lexer::new("hello_world");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Ident("hello_world".to_string())
        );

        let mut lexer = Lexer::new("hello_world_1234");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Ident("hello_world_1234".to_string())
        );

        let mut lexer = Lexer::new("hello world 1234");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Ident("hello".to_string())
        );
    }

    #[test]
    fn test_parse() {
        let mut lexer = Lexer::new(
            r#"
            {
                "hello": "world",
                "foo": 1234,
                "bar": 1234.5678,
                "baz": [
                    "hello",
                    "world",
                    1234,
                    1234.5678,
                    true,
                    false,
                    null
                ]
            }
        "#,
        );

        fn next(lexer: &mut Lexer) -> Token {
            lexer.next_token().unwrap()
        }

        assert_eq!(next(&mut lexer), Token::LBrace);
        assert_eq!(next(&mut lexer), Token::String("hello".to_string()));
        assert_eq!(next(&mut lexer), Token::Ident(":".to_string()));
        assert_eq!(next(&mut lexer), Token::String("world".to_string()));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::String("foo".to_string()));
        assert_eq!(next(&mut lexer), Token::Ident(":".to_string()));
        assert_eq!(next(&mut lexer), Token::Int(1234));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::String("bar".to_string()));
        assert_eq!(next(&mut lexer), Token::Ident(":".to_string()));
        assert_eq!(next(&mut lexer), Token::Float(1234.5678));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::String("baz".to_string()));
        assert_eq!(next(&mut lexer), Token::Ident(":".to_string()));
        assert_eq!(next(&mut lexer), Token::LBracket);
        assert_eq!(next(&mut lexer), Token::String("hello".to_string()));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::String("world".to_string()));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::Int(1234));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::Float(1234.5678));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::Ident("true".to_string()));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::Ident("false".to_string()));
        assert_eq!(next(&mut lexer), Token::Comma);
        assert_eq!(next(&mut lexer), Token::Ident("null".to_string()));
        assert_eq!(next(&mut lexer), Token::RBracket);
        assert_eq!(next(&mut lexer), Token::RBrace);
        assert_eq!(next(&mut lexer), Token::Eof);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_this_shouldnt_be_invalid_character() {
        let mut lexer = Lexer::new("@");
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("@".to_string()));
    }

    #[test]
    fn test_unexpected_eof() {
        let mut lexer = Lexer::new("\"Hello");
        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_large_input() {
        let input = std::iter::repeat("a").take(1000000).collect::<String>();
        let mut lexer = Lexer::new(&input);
        assert_eq!(lexer.next_token().unwrap(), Token::Ident(input));
    }

    #[test]
    fn test_nested_structures() {
        let mut lexer = Lexer::new("[[1, 2, 3], [4, 5, 6]]");
        assert_eq!(lexer.next_token().unwrap(), Token::LBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::LBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::Int(1));
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Int(2));
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Int(3));
        assert_eq!(lexer.next_token().unwrap(), Token::RBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::LBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::Int(4));
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Int(5));
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Int(6));
        assert_eq!(lexer.next_token().unwrap(), Token::RBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::RBracket);
    }
}
