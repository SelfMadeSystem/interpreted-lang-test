use anyhow::Result;
use std::{iter::Peekable, vec::IntoIter};
use thiserror::Error;

use crate::ast::AstNode;
use crate::lexer::Lexer;
use crate::token::{Keyword, Token};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("Unexpected end of file")]
    UnexpectedEof,
}

impl ParseError {
    pub fn new_unexpected(token: Token) -> Self {
        match token {
            Token::Eof => Self::UnexpectedEof,
            _ => Self::UnexpectedToken(token),
        }
    }

    pub fn new_opt_ref(token: Option<&Token>) -> Self {
        match token {
            Some(token) => Self::new_unexpected(token.to_owned()),
            None => Self::UnexpectedEof,
        }
    }

    pub fn new_opt(token: Option<Token>) -> Self {
        match token {
            Some(token) => Self::new_unexpected(token.to_owned()),
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

    fn expect(&mut self, expected: Token) -> Result<()> {
        if let Some(token) = self.tokens.next() {
            if token == expected {
                Ok(())
            } else {
                Err(ParseError::new_unexpected(token).into())
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
        match self.tokens.peek() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.tokens.next();
                Ok(Some(AstNode::String(s)))
            }
            Some(Token::Keyword(k)) => match k {
                Keyword::Fn => self.parse_fn(true),
                Keyword::Const => self.parse_declaration(Keyword::Const),
                Keyword::Let => self.parse_declaration(Keyword::Let),
                Keyword::Main => self.parse_main(),
                _ => Err(ParseError::UnexpectedToken(Token::Keyword(k.clone())).into()),
            },
            Some(Token::LParen) => self.parse_call(),
            Some(Token::Eof) => Ok(None),
            None => Ok(None),
            t => Err(ParseError::UnexpectedToken(t.cloned().unwrap()).into()),
        }
    }

    fn parse_ast_node(&mut self) -> Result<Option<AstNode>> {
        match self.tokens.peek() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.tokens.next();
                Ok(Some(AstNode::String(s)))
            }
            Some(Token::Int(i)) => {
                let i = *i;
                self.tokens.next();
                Ok(Some(AstNode::Int(i)))
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.tokens.next();
                Ok(Some(AstNode::Float(f)))
            }
            Some(Token::Comma) => {
                // comma is ignored
                self.tokens.next();
                self.parse_ast_node()
            }
            Some(Token::Keyword(Keyword::Const)) => self.parse_declaration(Keyword::Const),
            Some(Token::Keyword(Keyword::Let)) => self.parse_declaration(Keyword::Let),
            Some(Token::Keyword(Keyword::Set)) => self.parse_declaration(Keyword::Set),
            Some(Token::Keyword(Keyword::If)) => self.parse_if(),
            Some(Token::Keyword(Keyword::While)) => self.parse_while(),
            Some(Token::Keyword(Keyword::True)) => {
                self.tokens.next();
                Ok(Some(AstNode::Bool(true)))
            }
            Some(Token::Keyword(Keyword::False)) => {
                self.tokens.next();
                Ok(Some(AstNode::Bool(false)))
            }
            Some(Token::Keyword(k)) => {
                let k = k.clone();
                self.tokens.next();
                Ok(Some(AstNode::Keyword(k)))
            }
            Some(Token::Ident(i)) => {
                let i = i.clone();
                self.tokens.next();
                Ok(Some(AstNode::Ident(i)))
            }
            Some(Token::LParen) => self.parse_call(),
            Some(Token::LBrace) => self.parse_block(),
            Some(Token::LBracket) => self.parse_array(),
            Some(Token::Eof) => Ok(None),
            t => Err(ParseError::UnexpectedToken(t.cloned().unwrap()).into()),
        }
    }

    fn parse_call(&mut self) -> Result<Option<AstNode>> {
        self.expect(Token::LParen)?;

        let name = match self.tokens.peek() {
            Some(Token::Ident(i)) => {
                let i = i.to_owned();
                self.tokens.next();
                i
            }
            Some(Token::Keyword(Keyword::Fn)) => {
                let result = self.parse_fn(false);
                self.expect(Token::RParen)?;
                return result;
            }
            Some(Token::Keyword(Keyword::If)) => {
                let result = self.parse_if();
                self.expect(Token::RParen)?;
                return result;
            }
            Some(Token::Keyword(Keyword::While)) => {
                let result = self.parse_while();
                self.expect(Token::RParen)?;
                return result;
            }
            t => return Err(ParseError::new_opt_ref(t).into()),
        };

        let mut params = Vec::new();

        loop {
            match self.tokens.peek() {
                Some(Token::RParen) => break,
                Some(Token::Comma) => {
                    self.tokens.next();
                }
                Some(_) => params.push(self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?),
                None => return Err(ParseError::UnexpectedEof.into()),
            }
        }

        self.expect(Token::RParen)?;

        Ok(Some(AstNode::Call { name, params }))
    }

    fn parse_block(&mut self) -> Result<Option<AstNode>> {
        self.expect(Token::LBrace)?;

        let mut nodes = Vec::new();

        loop {
            if let Some(Token::RBrace) = self.tokens.peek() {
                break;
            }

            if let Some(ast) = self.parse_ast_node()? {
                nodes.push(ast);
            } else {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Some(AstNode::Block(nodes)))
    }

    fn parse_array(&mut self) -> Result<Option<AstNode>> {
        self.expect(Token::LBracket)?;

        let mut nodes = Vec::new();

        loop {
            if let Some(Token::RBracket) = self.tokens.peek() {
                break;
            }

            if let Some(ast) = self.parse_ast_node()? {
                nodes.push(ast);
            } else {
                break;
            }
        }

        self.expect(Token::RBracket)?;

        Ok(Some(AstNode::Array(nodes)))
    }

    fn parse_fn(&mut self, top_level: bool) -> Result<Option<AstNode>> {
        self.expect(Token::Keyword(Keyword::Fn))?;

        let name = match self.tokens.peek() {
            Some(Token::Ident(i)) => {
                let s = i.to_owned();
                self.tokens.next();
                s
            }
            Some(t) => {
                if top_level {
                    return Err(ParseError::new_unexpected(t.to_owned()).into());
                } else {
                    "Anonymous Function".to_owned()
                }
            }
            None => return Err(ParseError::UnexpectedEof.into()),
        };

        let mut params = Vec::new();

        match self.tokens.peek() {
            Some(Token::LParen) => {
                self.tokens.next();
                loop {
                    if let Some(Token::RParen) = self.tokens.peek() {
                        break;
                    }

                    match self.tokens.next() {
                        Some(Token::Comma) => {}
                        Some(Token::Ident(i)) => {
                            params.push(AstNode::Ident(i));
                        }
                        t => return Err(ParseError::new_opt(t).into()),
                    }
                }
                self.expect(Token::RParen)?;
            }
            Some(Token::LBrace) => {}
            t => return Err(ParseError::new_opt_ref(t).into()),
        }

        let body = self.parse_block()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode::Fn {
            name,
            params,
            body: match body {
                AstNode::Block(nodes) => nodes,
                _ => unreachable!(),
            },
        }))
    }

    fn parse_declaration(&mut self, keyword: Keyword) -> Result<Option<AstNode>> {
        self.expect(Token::Keyword(keyword))?;

        let name = match self.tokens.next() {
            Some(Token::Ident(i)) => i,
            t => return Err(ParseError::new_opt(t).into()),
        };

        let value = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode::declaration(keyword, name, value)))
    }

    fn parse_main(&mut self) -> Result<Option<AstNode>> {
        self.expect(Token::Keyword(Keyword::Main))?;

        let body = self.parse_block()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode::Main(match body {
            AstNode::Block(nodes) => nodes,
            _ => unreachable!(),
        })))
    }

    fn parse_if(&mut self) -> Result<Option<AstNode>> {
        self.expect(Token::Keyword(Keyword::If))?;

        let condition = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        let body = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        let else_body = match self.tokens.peek() {
            Some(Token::Keyword(Keyword::Else)) => {
                self.tokens.next();
                Some(self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?)
            }
            _ => None,
        };

        Ok(Some(AstNode::If {
            condition: Box::new(condition),
            body: Box::new(body),
            else_body: else_body.map(Box::new),
        }))
    }

    fn parse_while(&mut self) -> Result<Option<AstNode>> {
        self.expect(Token::Keyword(Keyword::While))?;

        let condition = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        let body = self.parse_ast_node()?.ok_or(ParseError::UnexpectedEof)?;

        Ok(Some(AstNode::While {
            condition: Box::new(condition),
            body: Box::new(body),
        }))
    }
}
