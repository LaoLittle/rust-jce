use crate::de::read_len;
use crate::error::{DecodeError, DecodeResult};
use bytes::Buf;
use std::fmt::{Display, Formatter};

pub const BYTE: u8 = 0;
pub const SHORT: u8 = 1;
pub const INT: u8 = 2;
pub const LONG: u8 = 3;
pub const FLOAT: u8 = 4;
pub const DOUBLE: u8 = 5;
pub const SHORT_BYTES: u8 = 6;
pub const LONG_BYTES: u8 = 7;
pub const MAP: u8 = 8;
pub const LIST: u8 = 9;
pub const STRUCT_START: u8 = 10;
pub const STRUCT_END: u8 = 11;
pub const EMPTY: u8 = 12;
pub const SINGLE_LIST: u8 = 13;

#[derive(Debug)]
pub enum Type {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Bytes,
    Map,
    List,
    Struct,
    Empty,
    SimpleList,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Type {
    pub fn from_u8(t: u8) -> Option<Self> {
        let t = match t {
            BYTE => Self::Byte,
            SHORT => Self::Short,
            INT => Self::Int,
            LONG => Self::Long,
            FLOAT => Self::Float,
            DOUBLE => Self::Double,
            SHORT_BYTES | LONG_BYTES => Self::Bytes,
            MAP => Self::Map,
            LIST => Self::List,
            STRUCT_START => Self::Struct,
            EMPTY => Self::Empty,
            _ => return None,
        };

        Some(t)
    }
}

macro_rules! primitive_type {
    (
        $type:ident,
        $jce_type:ident,
        $fun:ident
    ) => {
        pub mod $type {
            pub fn read<B: ::bytes::Buf>(
                buf: &mut B,
                t: u8,
                struct_name: &'static str,
                field: &'static str,
            ) -> $crate::error::DecodeResult<$type> {
                if t != $crate::types::$jce_type {
                    return ::core::result::Result::Err($crate::error::DecodeError::WrongType {
                        struct_name,
                        field,
                        val_type: t,
                    });
                }

                if ::std::mem::size_of::<$type>() > buf.remaining() {
                    return ::core::result::Result::Err($crate::error::DecodeError::Eof);
                }

                Ok(buf.$fun())
            }
        }
    };
}

macro_rules! byte_type {
    (
        $type:ident,
        $jce_type:ident,
        $fun:ident
    ) => {
        pub mod $type {
            pub fn read<B: ::bytes::Buf>(
                buf: &mut B,
                t: u8,
                struct_name: &'static str,
                field: &'static str,
            ) -> $crate::error::DecodeResult<$type> {
                if t == $crate::types::EMPTY {
                    return ::core::result::Result::Ok(0);
                }

                if t != $crate::types::$jce_type {
                    return ::core::result::Result::Err($crate::error::DecodeError::WrongType {
                        struct_name,
                        field,
                        val_type: t,
                    });
                }

                if ::std::mem::size_of::<$type>() > buf.remaining() {
                    return ::core::result::Result::Err($crate::error::DecodeError::Eof);
                }

                ::core::result::Result::Ok(buf.$fun())
            }
        }
    };
}

byte_type! {
    i8, BYTE, get_i8
}

byte_type! {
    u8, BYTE, get_u8
}

primitive_type! {
    i16, SHORT, get_i16
}

primitive_type! {
    u16, SHORT, get_u16
}

primitive_type! {
    i32, INT, get_i32
}

primitive_type! {
    u32, INT, get_u32
}

primitive_type! {
    i64, LONG, get_i64
}

primitive_type! {
    u64, LONG, get_u64
}

primitive_type! {
    f32, FLOAT, get_f32
}

primitive_type! {
    f64, DOUBLE, get_f64
}

pub(crate) fn bytes_from_buf<B: Buf>(buf: &mut B, len: usize) -> Vec<u8> {
    if len > 0 {
        let b = Vec::from(&buf.chunk()[..len]);
        buf.advance(len);
        b
    } else {
        vec![]
    }
}

pub mod byte_array {
    use crate::error::{DecodeError, DecodeResult};
    use bytes::Buf;

    pub fn read_bytes_len<B: Buf>(
        buf: &mut B,
        t: u8,
        struct_name: &'static str,
        field: &'static str,
    ) -> DecodeResult<usize> {
        let len = match t {
            super::SHORT_BYTES => {
                if buf.remaining() < 1 {
                    return Err(DecodeError::Eof);
                }

                buf.get_u8() as usize
            }
            super::LONG_BYTES => {
                if buf.remaining() < 4 {
                    return Err(DecodeError::Eof);
                }

                buf.get_u32() as usize
            }
            _ => {
                return Err(DecodeError::WrongType {
                    struct_name,
                    field,
                    val_type: t,
                })
            }
        };

        Ok(len)
    }

    pub fn read<B: Buf>(
        buf: &mut B,
        t: u8,
        struct_name: &'static str,
        field: &'static str,
    ) -> DecodeResult<Vec<u8>> {
        let len = read_bytes_len(buf, t, struct_name, field)?;

        let mut v = vec![0u8; len];
        read_slice(buf, &mut v, len)?;

        Ok(v)
    }

    pub fn read_slice<B: Buf>(buf: &mut B, value: &mut [u8], len: usize) -> DecodeResult<()> {
        if buf.remaining() < len {
            return Err(DecodeError::Eof);
        }

        if value.len() < len {
            return Err(DecodeError::WrongLength);
        }

        value[..len].copy_from_slice(&buf.chunk()[..len]);

        Ok(())
    }
}

pub mod jce_struct {
    use crate::error::{DecodeError, DecodeResult};
    use crate::JceStruct;
    use bytes::Buf;

    pub fn read<B, S>(
        buf: &mut B,
        t: u8,
        struct_name: &'static str,
        field: &'static str,
    ) -> DecodeResult<S>
    where
        B: Buf,
        S: JceStruct,
    {
        if t != super::STRUCT_START {
            return Err(DecodeError::WrongType {
                struct_name,
                field,
                val_type: t,
            });
        }

        S::decode_raw(buf, false)
    }
}

pub fn skip_field<B: Buf>(buf: &mut B, t: u8) -> DecodeResult<()> {
    fn read_type<B: Buf>(buf: &mut B) -> u8 {
        buf.get_u8() & 0xF
    }

    match t {
        BYTE => buf.advance(1),
        SHORT => buf.advance(2),
        INT | FLOAT => buf.advance(4),
        LONG | DOUBLE => buf.advance(8),
        SHORT_BYTES => {
            let len = buf.get_u8() as usize;
            buf.advance(len);
        }
        LONG_BYTES => {
            let len = buf.get_u32() as usize;
            buf.advance(len);
        }
        STRUCT_START => {
            let t = read_type(buf);
            skip_field(buf, t)?;
        }
        STRUCT_END | EMPTY => {}
        MAP => {
            let len = read_len(buf)?;

            for _ in 0..len * 2 {
                // skip key and value
                let t = read_type(buf);
                skip_field(buf, t)?;
            }
        }
        LIST => {
            let len = read_len(buf)?;

            for _ in 0..len {
                let t = read_type(buf);
                skip_field(buf, t)?;
            }
        }
        SINGLE_LIST => {
            let tt = read_type(buf);

            let len = read_len(buf)?;

            let single = match tt {
                BYTE => 1,
                SHORT => 2,
                INT | FLOAT => 4,
                LONG | DOUBLE => 8,
                _ => 1,
            };

            buf.advance(len * single);
        }
        _ => return Err(DecodeError::FieldTypeIncorrect),
    }

    Ok(())
}
