#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Fn,
    Main,
    Const,
    Let,
    Set,
    True,
    False,
    If,
    Else,
    While,
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "fn" => Ok(Self::Fn),
            "main" => Ok(Self::Main),
            "const" => Ok(Self::Const),
            "let" => Ok(Self::Let),
            "set" => Ok(Self::Set),
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "if" => Ok(Self::If),
            "else" => Ok(Self::Else),
            "while" => Ok(Self::While),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Eof,

    // Keywords
    Keyword(Keyword),

    // Identifiers + literals
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),

    // Delimiters
    Comma,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
}

impl TokenType {
    pub fn new_ident(ident: &str) -> Self {
        ident.try_into()
            .map(Self::Keyword)
            .unwrap_or_else(|_| Self::Ident(ident.to_string()))
    }
}
