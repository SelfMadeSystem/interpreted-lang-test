#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum TokenIdent {
    Ident(String),
    Macro(String),
    Type(String),
}

impl TokenIdent {
    pub fn to_string(&self) -> String {
        match self {
            Self::Ident(s) => s.to_owned(),
            Self::Macro(s) => format!("@{}", s),
            Self::Type(s) => format!("${}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Eof,

    // Identifiers + literals
    Ident(TokenIdent),
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),

    // Delimiters
    Comma,
    Colon,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
}
impl TokenType {
    pub fn new_ident(ident: &str) -> TokenType {
        match ident {
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            c => TokenType::Ident(match c {
                c if c.starts_with('@') => TokenIdent::Macro(c[1..].to_string()),
                c if c.starts_with('$') => TokenIdent::Type(c[1..].to_string()),
                _ => TokenIdent::Ident(ident.to_string()),
            }),
        }
    }
}
