#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
pub enum TokenKind {
    Keyword,
    Identifier,
    StringLiteral,
    Seperator,
    Operator,
    EOF,
}

// 代表一个Token的数据结构
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}
