use crate::token::TokenIdent;

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
    Ident(TokenIdent),
    Call {
        name: TokenIdent,
        params: Vec<AstNode>,
    },
    Array(Vec<AstNode>),
}
