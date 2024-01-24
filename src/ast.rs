use crate::token::Keyword;

/// An abstract syntax tree node
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Keyword(Keyword),
    Fn {
        name: String,
        params: Vec<AstNode>,
        body: Vec<AstNode>,
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
    Main(Vec<AstNode>),
    Call {
        name: String,
        params: Vec<AstNode>,
    },
    Block(Vec<AstNode>),
    Array(Vec<AstNode>),
}

impl AstNode {
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
