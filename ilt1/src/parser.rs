use anyhow::Result;
use std::{iter::Peekable, vec::IntoIter};
use thiserror::Error;

use crate::ast::{AstNode, AstNodeType};
use crate::lexer::Lexer;
use crate::token::{Keyword, Token, TokenType};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {0:?} at {1}:{2}")]
    UnexpectedToken(TokenType, usize, usize),
    #[error("Unexpected end of file")]
    UnexpectedEof,
}

impl ParseError {
    pub fn new_unexpected(token: &Token) -> Self {
        match token.ty {
            TokenType::Eof => Self::UnexpectedEof,
            _ => Self::UnexpectedToken(token.ty.to_owned(), token.line, token.col),
        }
    }

    pub fn new_opt_ref(token: Option<&Token>) -> Self {
        match token {
            Some(token) => Self::new_unexpected(token),
            None => Self::UnexpectedEof,
        }
    }

    pub fn new_opt(token: Option<Token>) -> Self {
        match token {
            Some(token) => Self::new_unexpected(&token),
            None => Self::UnexpectedEof,
        }
    }
}

/// Parses the output of the lexer into an AST.
pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn try_new(lexer: Lexer) -> Result<Self> {
        Ok(Self {
            tokens: lexer.parse()?.into_iter().peekable(),
        })
    }

    fn expect(&mut self, expected: TokenType) -> Result<(usize, usize)> {
        if let Some(token) = self.tokens.next() {
            if token.ty == expected {
                Ok((token.line, token.col))
            } else {
                Err(ParseError::new_unexpected(&token).into())
            }
        } else {
            Err(ParseError::UnexpectedEof.into())
        }
    }

    pub fn parse(&mut self) -> Result<Vec<AstNode>> {
        let mut nodes = Vec::new();

        loop {
            if let Some(ast) = self.parse_top_level_ast()? {
                nodes.push(ast);
            } else {
                break;
            }
        }

        Ok(nodes)
    }

    fn parse_top_level_ast(&mut self) -> Result<Option<AstNode>> {
        let token = self.tokens.peek();
        let Some(token) = token else {
            return Ok(None);
        };
        let Token { ty, line, col } = token;
        let line = *line;
        let col = *col;
        match ty {
            TokenType::String(s) => {
                let s = s.clone();
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::String(s),
                    line,
                    col,
                }))
            }
            TokenType::Keyword(k) => match k {
                Keyword::Fn => self.parse_fn(true),
                Keyword::Const => self.parse_declaration(Keyword::Const),
                Keyword::Let => self.parse_declaration(Keyword::Let),
                Keyword::Main => self.parse_main(),
                _ => Err(ParseError::new_unexpected(token).into()),
            },
            TokenType::LParen => self.parse_call(),
            TokenType::Eof => Ok(None),
            _ => Err(ParseError::new_unexpected(token).into()),
        }
    }

    fn parse_ast_node(&mut self) -> Result<Option<AstNode>> {
        let token = self.tokens.peek();
        let Some(token) = token else {
            return Ok(None);
        };
        let Token { ty, line, col } = token;
        let line = *line;
        let col = *col;
        match ty {
            TokenType::String(s) => {
                let s = s.clone();
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::String(s),
                    line,
                    col,
                }))
            }
            TokenType::Int(i) => {
                let i = *i;
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::Int(i),
                    line,
                    col,
                }))
            }
            TokenType::Float(f) => {
                let f = *f;
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::Float(f),
                    line,
                    col,
                }))
            }
            TokenType::Comma => {
                // comma is ignored
                self.tokens.next();
                self.parse_ast_node()
            }
            TokenType::Keyword(Keyword::Const) => self.parse_declaration(Keyword::Const),
            TokenType::Keyword(Keyword::Let) => self.parse_declaration(Keyword::Let),
            TokenType::Keyword(Keyword::Set) => self.parse_declaration(Keyword::Set),
            TokenType::Keyword(Keyword::If) => self.parse_if(),
            TokenType::Keyword(Keyword::While) => self.parse_while(),
            TokenType::Keyword(Keyword::Fn) => self.parse_fn(false),
            TokenType::Keyword(Keyword::True) => {
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::Bool(true),
                    line,
                    col,
                }))
            }
            TokenType::Keyword(Keyword::False) => {
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::Bool(false),
                    line,
                    col,
                }))
            }
            TokenType::Keyword(k) => {
                let k = k.clone();
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::Keyword(k),
                    line,
                    col,
                }))
            }
            TokenType::Ident(i) => {
                let i = i.clone();
                self.tokens.next();
                Ok(Some(AstNode {
                    ty: AstNodeType::Ident(i),
                    line,
                    col,
                }))
            }
            TokenType::LParen => self.parse_call(),
            TokenType::LBrace => self.parse_block(),
            TokenType::LBracket => self.parse_array(),
            TokenType::Eof => Ok(None),
            _ => Err(ParseError::new_unexpected(token).into()),
        }
    }

    fn parse_call(&mut self) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::LParen)?;

        let name = match self.tokens.peek() {
            Some(Token {
                ty: TokenType::Ident(i),
                ..
            }) => {
                let i = i.to_owned();
                self.tokens.next();
                i
            }
            t => return Err(ParseError::new_opt_ref(t).into()),
        };

        let mut params = Vec::new();

        loop {
            match self.tokens.peek() {
                Some(Token {
                    ty: TokenType::RParen,
                    ..
                }) => break,
                Some(Token {
                    ty: TokenType::Comma,
                    ..
                }) => {
                    self.tokens.next();
                }
                Some(_) => params.push(self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?),
                None => return Err(ParseError::UnexpectedEof.into()),
            }
        }

        self.expect(TokenType::RParen)?;

        Ok(Some(AstNode {
            ty: AstNodeType::Call { name, params },
            line,
            col,
        }))
    }

    fn parse_block(&mut self) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::LBrace)?;

        let mut nodes = Vec::new();

        loop {
            if let Some(Token {
                ty: TokenType::RBrace,
                ..
            }) = self.tokens.peek()
            {
                break;
            }

            if let Some(ast) = self.parse_ast_node()? {
                nodes.push(ast);
            } else {
                break;
            }
        }

        self.expect(TokenType::RBrace)?;

        Ok(Some(AstNode {
            ty: AstNodeType::Block(nodes),
            line,
            col,
        }))
    }

    fn parse_array(&mut self) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::LBracket)?;

        let mut nodes = Vec::new();

        loop {
            if let Some(Token {
                ty: TokenType::RBracket,
                ..
            }) = self.tokens.peek()
            {
                break;
            }

            if let Some(ast) = self.parse_ast_node()? {
                nodes.push(ast);
            } else {
                break;
            }
        }

        self.expect(TokenType::RBracket)?;

        Ok(Some(AstNode {
            ty: AstNodeType::Array(nodes),
            line,
            col,
        }))
    }

    fn parse_fn(&mut self, top_level: bool) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::Keyword(Keyword::Fn))?;

        let name = match self.tokens.peek() {
            Some(Token {
                ty: TokenType::Ident(i),
                ..
            }) => {
                let s = i.to_owned();
                self.tokens.next();
                s
            }
            Some(t) => {
                if top_level {
                    return Err(ParseError::new_unexpected(t).into());
                } else {
                    "Anonymous Function".to_owned()
                }
            }
            None => return Err(ParseError::UnexpectedEof.into()),
        };

        let mut params = Vec::new();

        match self.tokens.peek() {
            Some(Token {
                ty: TokenType::LParen,
                ..
            }) => {
                self.tokens.next();
                loop {
                    if let Some(Token {
                        ty: TokenType::RParen,
                        ..
                    }) = self.tokens.peek()
                    {
                        break;
                    }

                    match self.tokens.next() {
                        Some(Token {
                            ty: TokenType::Comma,
                            ..
                        }) => {}
                        Some(Token {
                            ty: TokenType::Ident(i),
                            ..
                        }) => {
                            params.push(AstNode {
                                ty: AstNodeType::Ident(i.to_owned()),
                                line: 0,
                                col: 0,
                            });
                        }
                        t => return Err(ParseError::new_opt(t).into()),
                    }
                }
                self.expect(TokenType::RParen)?;
            }
            Some(Token {
                ty: TokenType::LBrace,
                ..
            }) => {}
            t => return Err(ParseError::new_opt_ref(t).into()),
        }

        let body = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode {
            ty: AstNodeType::Fn {
                name,
                params,
                body: Box::new(body),
            },
            line,
            col,
        }))
    }

    fn parse_declaration(&mut self, keyword: Keyword) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::Keyword(keyword))?;

        let name = match self.tokens.next() {
            Some(Token {
                ty: TokenType::Ident(i),
                ..
            }) => i,
            t => return Err(ParseError::new_opt(t).into()),
        };

        let value = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode {
            ty: AstNodeType::declaration(keyword, name, value),
            line,
            col,
        }))
    }

    fn parse_main(&mut self) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::Keyword(Keyword::Main))?;

        let body = self.parse_block()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode {
            ty: AstNodeType::Main(Box::new(body)),
            line,
            col,
        }))
    }

    fn parse_if(&mut self) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::Keyword(Keyword::If))?;

        let condition = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        let body = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        let else_body = match self.tokens.peek() {
            Some(Token {
                ty: TokenType::Keyword(Keyword::Else),
                ..
            }) => {
                self.tokens.next();
                Some(self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?)
            }
            _ => None,
        };

        Ok(Some(AstNode {
            ty: AstNodeType::If {
                condition: Box::new(condition),
                body: Box::new(body),
                else_body: else_body.map(Box::new),
            },
            line,
            col,
        }))
    }

    fn parse_while(&mut self) -> Result<Option<AstNode>> {
        let (line, col) = self.expect(TokenType::Keyword(Keyword::While))?;

        let condition = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        let body = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode {
            ty: AstNodeType::While {
                condition: Box::new(condition),
                body: Box::new(body),
            },
            line,
            col,
        }))
    }
}
