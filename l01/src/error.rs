use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum DecodeError {
    TryNext,
    Fatal(String),
}
impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::TryNext => write!(f, "Please try next method"),
            DecodeError::Fatal(message) => write!(f, "{}", message),
        }
    }
}
impl From<&str> for DecodeError {
    fn from(message: &str) -> Self {
        DecodeError::Fatal(message.to_string())
    }
}
impl From<String> for DecodeError {
    fn from(message: String) -> Self {
        DecodeError::Fatal(message)
    }
}
