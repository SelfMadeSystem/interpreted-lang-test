use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct GenericIdent {
    pub ident: TokenIdent,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum TokenIdent {
    Ident(String, Option<Vec<GenericIdent>>),
    Macro(String, Option<Vec<GenericIdent>>),
    Type(String, Option<Vec<GenericIdent>>),
}

impl TokenIdent {
    pub fn without_generics(&self) -> Self {
        match self {
            Self::Ident(s, _) => Self::Ident(s.to_owned(), None),
            Self::Macro(s, _) => Self::Macro(s.to_owned(), None),
            Self::Type(s, _) => Self::Type(s.to_owned(), None),
        }
    }

    pub fn get_generics(&self) -> Option<&Vec<GenericIdent>> {
        match self {
            Self::Ident(_, g) | Self::Macro(_, g) | Self::Type(_, g) => g.as_ref(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Ident(s, _) | Self::Macro(s, _) | Self::Type(s, _) => s,
        }
    }

    pub fn base_to_string(&self) -> String {
        match self {
            Self::Ident(s, _) => s.to_owned(),
            Self::Macro(s, _) => format!("@{}", s),
            Self::Type(s, _) => format!("${}", s),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Ident(_, None) | Self::Macro(_, None) | Self::Type(_, None) => {
                self.base_to_string()
            }
            Self::Ident(_, Some(g)) | Self::Macro(_, Some(g)) | Self::Type(_, Some(g)) => format!(
                "{}[{}]",
                self.base_to_string(),
                g.iter()
                    .map(|v| v.ident.base_to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
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
    pub fn new_ident(ident: &str, generics: Option<Vec<Token>>) -> Result<Self> {
        let generics = if let Some(generics) = generics {
            let mut g = Vec::new();
            for token in generics {
                match token.ty {
                    TokenType::Ident(ident) => g.push(GenericIdent {
                        ident,
                        line: token.line,
                        col: token.col,
                    }),
                    TokenType::Comma | TokenType::Colon => {}
                    // TODO: Make error enum when we have more TokenType-specific errors
                    _ => return Err(anyhow!("Invalid generic type")),
                }
            }
            Some(g)
        } else {
            None
        };

        Ok(match ident {
            "true" => Self::Boolean(true),
            "false" => Self::Boolean(false),
            c if c.starts_with('@') => Self::Ident(TokenIdent::Macro(c[1..].to_owned(), generics)),
            c if c.starts_with('$') => Self::Ident(TokenIdent::Type(c[1..].to_owned(), generics)),
            c => Self::Ident(TokenIdent::Ident(c.to_owned(), generics)),
        })
    }
}
