use crate::types::Type;
use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug)]
pub enum DecodeError {
    IncorrectType {
        struct_name: &'static str,
        field: &'static str,
        val_type: u8,
    },
    InvalidType,
    Eof,
    InvalidLength,
    String(FromUtf8Error),
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncorrectType {
                struct_name,
                field,
                val_type,
            } => {
                f.write_str("incorrect type(")?;
                if let Some(t) = Type::from_u8(*val_type) {
                    Display::fmt(&t, f)?;
                } else {
                    Display::fmt(&val_type, f)?;
                }

                write!(f, ") of field {} in struct {}", field, struct_name)?;

                Ok(())
            }
            Self::InvalidType => f.write_str("invalid type"),
            Self::Eof => f.write_str("unexpected eof"),
            Self::InvalidLength => f.write_str("invalid length"),
            Self::String(e) => Display::fmt(e, f),
        }
    }
}

impl From<FromUtf8Error> for DecodeError {
    fn from(err: FromUtf8Error) -> Self {
        Self::String(err)
    }
}
