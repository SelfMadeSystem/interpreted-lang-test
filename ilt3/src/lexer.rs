// TODO: Add struct parsing

use std::{iter::Peekable, vec::IntoIter};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum LexError {
    #[error("Unexpected character `{0}` at line {1}, column {2}")]
    UnexpectedChar(char, usize, usize),
    #[error("Invalid instruction type `{0}` at line {1}")]
    InvalidInstructionType(String, usize),
    #[error("Invalid number `{0}` at line {1}, column {2}")]
    InvalidNumber(String, usize, usize),
    #[error("Invalid unicode escape sequence `{0}` at line {1}, column {2}")]
    InvalidUnicodeEscape(String, usize, usize),
    #[error("Unexpected end of input")]
    UnexpectedEof,
}

#[derive(Debug, Clone)]
pub enum LineType {
    /// Any number of leading spaces
    Comment { text: String },
    /// 0 leading spaces
    Function { name: String, args: Vec<String> },
    /// 2 leading spaces
    Label { name: String },
    /// 4 leading spaces
    Instruction {
        ty: InstructionType,
        tokens: Vec<InstructionToken>,
    },
}

impl LineType {
    pub fn to_string(&self) -> String {
        match self {
            LineType::Comment { text } => format!("#{}", text),
            LineType::Function { name, args } => {
                let args = args.join(" ");
                format!("{} {}", name, args)
            }
            LineType::Label { name } => format!("{}", name),
            LineType::Instruction { ty, tokens } => {
                let tokens = tokens
                    .iter()
                    .map(|token| token.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{} {}", ty.to_string(), tokens)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub ty: LineType,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum InstructionType {
    Call,
    CallAssign,
    Assign,
    Jump,
    JumpIf,
    Return,
}

impl InstructionType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "call" => Some(Self::Call),
            "call_assign" => Some(Self::CallAssign),
            "assign" => Some(Self::Assign),
            "jump" => Some(Self::Jump),
            "jump_if" => Some(Self::JumpIf),
            "return" => Some(Self::Return),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Call => "call".to_string(),
            Self::CallAssign => "call_assign".to_string(),
            Self::Assign => "assign".to_string(),
            Self::Jump => "jump".to_string(),
            Self::JumpIf => "jump_if".to_string(),
            Self::Return => "return".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InstructionTokenType {
    Identifier(String),
    /// Starts with a digit, or minus sign
    Int(i64),
    /// Starts with a digit, period, or minus sign
    Float(f64),
    /// Exactly `true` or `false`
    Boolean(bool),
    /// Starts and ends with a double quote
    String(String),
    /// Starts with an open square bracket and ends with a close square bracket
    Array(Vec<InstructionToken>),
    /// Exactly `void`
    Void,
}

#[derive(Debug, Clone)]
pub struct InstructionToken {
    pub ty: InstructionTokenType,
    pub line: usize,
    pub col: usize,
}

impl InstructionToken {
    pub fn to_string(&self) -> String {
        match &self.ty {
            InstructionTokenType::Identifier(s) => s.clone(),
            InstructionTokenType::Int(n) => n.to_string(),
            InstructionTokenType::Float(f) => f.to_string(),
            InstructionTokenType::Boolean(b) => b.to_string(),
            InstructionTokenType::String(s) => format!("\"{}\"", s),
            InstructionTokenType::Array(tokens) => {
                let tokens = tokens
                    .iter()
                    .map(|token| token.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("[{}]", tokens)
            }
            InstructionTokenType::Void => "void".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    pub iter: Peekable<IntoIter<char>>,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            iter: input.chars().collect::<Vec<_>>().into_iter().peekable(),
            line: 1,
            col: 1,
        }
    }

    fn next(&mut self) -> Option<char> {
        let c = self.iter.next();

        if let Some(c) = c {
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }

        c
    }

    fn new_unexpected_char(&self, c: char) -> anyhow::Error {
        LexError::UnexpectedChar(c, self.line, self.col).into()
    }

    fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if *c == '\n' {
                break;
            } else if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn count_spaces(&mut self) -> usize {
        let mut count = 0;

        while let Some(c) = self.peek() {
            match c {
                ' ' => {
                    count += 1;
                    self.next();
                }
                '\n' => {
                    count = 0;
                    self.next();
                }
                _ => break,
            }
        }

        count
    }

    fn parse_comment(&mut self) -> Result<LineType> {
        let mut text = String::new();

        while let Some(c) = self.peek() {
            if *c == '\n' {
                break;
            }

            text.push(*c);
            self.next();
        }

        Ok(LineType::Comment { text })
    }

    fn parse_function(&mut self) -> Result<LineType> {
        let mut name = String::new();
        let mut args = Vec::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                break;
            }

            name.push(*c);
            self.next();
        }

        self.skip_whitespace();

        while let Some(c) = self.peek() {
            if *c == '\n' {
                break;
            }

            let mut arg = String::new();

            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    break;
                }

                arg.push(*c);
                self.next();
            }

            args.push(arg);
            self.skip_whitespace();
        }

        Ok(LineType::Function { name, args })
    }

    fn parse_label(&mut self) -> Result<LineType> {
        let mut name = String::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                break;
            }

            name.push(*c);
            self.next();
        }

        Ok(LineType::Label { name })
    }

    fn parse_tokens(&mut self) -> Result<LineType> {
        let mut ty = String::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
                break;
            }

            ty.push(*c);
            self.next();
        }

        let ty = InstructionType::from_string(&ty)
            .ok_or(LexError::InvalidInstructionType(ty, self.line))?;

        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            if *c == '\n' {
                break;
            }

            let token = self.parse_token()?;
            tokens.push(token);
            self.skip_whitespace();
        }

        Ok(LineType::Instruction { ty, tokens })
    }

    fn parse_token(&mut self) -> Result<InstructionToken> {
        let ty = if let Some(c) = self.peek() {
            if c.is_digit(10) || *c == '.' || *c == '-' {
                self.parse_number()?
            } else if *c == '"' {
                self.parse_string()?
            } else if *c == '[' {
                self.parse_array()?
            } else {
                self.parse_identifier()?
            }
        } else {
            return Err(LexError::UnexpectedEof.into());
        };

        Ok(InstructionToken {
            ty,
            line: self.line,
            col: self.col,
        })
    }

    fn parse_identifier(&mut self) -> Result<InstructionTokenType> {
        let mut identifier = String::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                break;
            }

            identifier.push(*c);
            self.next();
        }

        match identifier.as_str() {
            "true" => Ok(InstructionTokenType::Boolean(true)),
            "false" => Ok(InstructionTokenType::Boolean(false)),
            "void" => Ok(InstructionTokenType::Void),
            _ => Ok(InstructionTokenType::Identifier(identifier)),
        }
    }

    fn parse_number(&mut self) -> Result<InstructionTokenType> {
        let mut number = String::new();
        let mut float = false;

        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                break;
            }
            if *c == '.' {
                if float {
                    return Err(self.new_unexpected_char('.'));
                }
                float = true;
            }

            number.push(*c);
            self.next();
        }

        if float {
            number
                .parse::<f64>()
                .map_err(|_| LexError::InvalidNumber(number.clone(), self.line, self.col).into())
                .map(InstructionTokenType::Float)
        } else {
            number
                .parse::<i64>()
                .map_err(|_| LexError::InvalidNumber(number.clone(), self.line, self.col).into())
                .map(InstructionTokenType::Int)
        }
    }

    fn parse_string(&mut self) -> Result<InstructionTokenType> {
        let mut string = String::new();

        self.next();

        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.next();
                    break;
                }
                '\\' => {
                    let c = self.next().ok_or(LexError::UnexpectedEof)?;
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
                                let c = self.next().ok_or(LexError::UnexpectedEof)?;
                                hex.push(c);
                            }
                            u32::from_str_radix(&hex, 16)
                                .map_err(|_| {
                                    LexError::InvalidUnicodeEscape(hex.clone(), self.line, self.col)
                                })?
                                .try_into()
                                .map_err(|_| {
                                    LexError::InvalidUnicodeEscape(hex.clone(), self.line, self.col)
                                })?
                        }
                        _ => return Err(self.new_unexpected_char(c)),
                    })
                }
                _ => {
                    string.push(*c);
                    self.next();
                }
            }
        }

        Ok(InstructionTokenType::String(string))
    }

    fn parse_array(&mut self) -> Result<InstructionTokenType> {
        let mut tokens = Vec::new();

        self.next();

        while let Some(c) = self.peek() {
            if *c == ']' {
                self.next();
                break;
            }

            self.skip_whitespace();
            let token = self.parse_token()?;
            tokens.push(token);
            self.skip_whitespace();
        }

        Ok(InstructionTokenType::Array(tokens))
    }

    fn parse_line(&mut self) -> Result<Line> {
        self.skip_whitespace();

        let spaces = self.count_spaces();

        if let Some(c) = self.peek() {
            if *c == '#' {
                self.next();
                return Ok(Line {
                    ty: self.parse_comment()?,
                    line: self.line,
                });
            }
        }

        let ty = match spaces {
            0 => self.parse_function()?,
            2 => self.parse_label()?,
            4 => self.parse_tokens()?,
            _ => return Err(self.new_unexpected_char(' ')),
        };

        Ok(Line {
            ty,
            line: self.line,
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Line>> {
        let mut lines = Vec::new();

        while let Some(_) = self.peek() {
            let line = self.parse_line()?;
            lines.push(line);
        }

        Ok(lines)
    }
}
