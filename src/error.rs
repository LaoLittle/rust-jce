use crate::types::Type;
use std::fmt::{Display, Formatter};

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug)]
pub enum DecodeError {
    WrongType {
        struct_name: &'static str,
        field: &'static str,
        val_type: u8,
    },
    FieldTypeIncorrect,
    Eof,
    WrongLength,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongType {
                struct_name,
                field,
                val_type,
            } => {
                f.write_str("wrong type(")?;
                if let Some(t) = Type::from_u8(*val_type) {
                    Display::fmt(&t, f)?;
                } else {
                    Display::fmt(&val_type, f)?;
                }

                write!(f, ") of field {} in struct {}", field, struct_name)?;

                Ok(())
            }
            Self::FieldTypeIncorrect => f.write_str("field type is incorrect"),
            Self::Eof => f.write_str("unexpected eof"),
            Self::WrongLength => f.write_str("wrong length while decoding array types"),
        }
    }
}
