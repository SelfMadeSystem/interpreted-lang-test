use crate::token::Keyword;

/// An abstract syntax tree node
#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    pub ty: AstNodeType,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Keyword(Keyword),
    Fn {
        name: String,
        params: Vec<AstNode>,
        body: Box<AstNode>,
    },
    Const {
        name: String,
        value: Box<AstNode>,
    },
    Let {
        name: String,
        value: Box<AstNode>,
    },
    Set {
        name: String,
        value: Box<AstNode>,
    },
    If {
        condition: Box<AstNode>,
        body: Box<AstNode>,
        else_body: Option<Box<AstNode>>,
    },
    While {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    Main(Box<AstNode>),
    Call {
        name: String,
        params: Vec<AstNode>,
    },
    Block(Vec<AstNode>),
    Array(Vec<AstNode>),
}

impl AstNodeType {
    pub fn declaration(keyword: Keyword, name: String, value: AstNode) -> Self {
        match keyword {
            Keyword::Const => Self::Const {
                name,
                value: Box::new(value),
            },
            Keyword::Let => Self::Let {
                name,
                value: Box::new(value),
            },
            Keyword::Set => Self::Set {
                name,
                value: Box::new(value),
            },
            _ => unreachable!(),
        }
    }
}
