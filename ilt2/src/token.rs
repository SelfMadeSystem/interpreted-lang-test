#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenIdent {
    Ident(String),
    Macro(String),
    Type(String),
}

impl TokenIdent {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ident(s) => s,
            Self::Macro(s) => s,
            Self::Type(s) => s,
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
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
