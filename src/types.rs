use crate::de::{check_buf, check_buf_zero, read_len};
use crate::error::{DecodeError, DecodeResult};
use bytes::{Buf, BufMut};
use std::fmt::{Display, Formatter};

pub const BYTE: u8 = 0; // i8 / u8
pub const SHORT: u8 = 1; // i16 / u16
pub const INT: u8 = 2; // i32 / u32
pub const LONG: u8 = 3; // i64 / u64
pub const FLOAT: u8 = 4; // f32
pub const DOUBLE: u8 = 5; // f64
pub const SHORT_BYTES: u8 = 6;
pub const LONG_BYTES: u8 = 7;
pub const MAP: u8 = 8; // Map<String, *Any>
pub const LIST: u8 = 9; // Vec<*Any>
pub const STRUCT_START: u8 = 10;
pub const STRUCT_END: u8 = 11;
pub const EMPTY: u8 = 12; // *Any
pub const SINGLE_LIST: u8 = 13; // Vec<T>

#[derive(Debug)]
pub struct JceHeader {
    pub(crate) val_type: u8,
    pub(crate) tag: u8,
}

impl JceHeader {
    #[inline]
    pub fn tag(&self) -> u8 {
        self.tag
    }

    #[inline]
    pub fn value_type(&self) -> u8 {
        self.val_type
    }
}

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

pub trait JceType: Sized {
    fn read<B: Buf>(
        buf: &mut B,
        t: u8,
        struct_name: &'static str,
        field: &'static str,
    ) -> DecodeResult<Self>;

    fn write<B: BufMut>(&self, buf: &mut B, tag: u8);
}

macro_rules! primitive_type {
    (
        $type:ident,
        $jce_type:ident,
        $read:ident,
        $write:ident
    ) => {
        mod $type {
            impl $crate::types::JceType for $type {
                fn read<B: ::bytes::Buf>(
                    buf: &mut B,
                    t: u8,
                    struct_name: &'static str,
                    field: &'static str,
                ) -> $crate::error::DecodeResult<$type> {
                    if t == $crate::types::EMPTY {
                        return ::core::result::Result::Ok(0 as $type);
                    }

                    if t != $crate::types::$jce_type {
                        return ::core::result::Result::Err(
                            $crate::error::DecodeError::TypeIncorrect {
                                struct_name,
                                field,
                                val_type: t,
                            },
                        );
                    }

                    if ::std::mem::size_of::<$type>() > buf.remaining() {
                        return ::core::result::Result::Err($crate::error::DecodeError::Eof);
                    }

                    Ok(buf.$read())
                }

                fn write<B: ::bytes::BufMut>(&self, buf: &mut B, tag: u8) {
                    let t = *self;

                    if t == 0 as $type {
                        $crate::ser::write_empty(buf, tag);
                        return;
                    }

                    $crate::ser::write_header(
                        buf,
                        $crate::types::JceHeader {
                            val_type: $crate::types::$jce_type,
                            tag,
                        },
                    );

                    buf.$write(t);
                }
            }
        }
    };
}

primitive_type! {
    i8,
    BYTE,
    get_i8,
    put_i8
}

primitive_type! {
    u8,
    BYTE,
    get_u8,
    put_u8
}

primitive_type! {
    i16,
    SHORT,
    get_i16,
    put_i16
}

primitive_type! {
    u16,
    SHORT,
    get_u16,
    put_u16
}

primitive_type! {
    i32,
    INT,
    get_i32,
    put_i32
}

primitive_type! {
    u32,
    INT,
    get_u32,
    put_u32
}

primitive_type! {
    i64,
    LONG,
    get_i64,
    put_i64
}

primitive_type! {
    u64,
    LONG,
    get_u64,
    put_u64
}

primitive_type! {
    f32,
    FLOAT,
    get_f32,
    put_f32
}

primitive_type! {
    f64,
    DOUBLE,
    get_f64,
    put_f64
}

mod byte_array {
    use crate::de::{check_buf, check_buf_zero};
    use crate::error::{DecodeError, DecodeResult};
    use crate::ser::{write_empty, write_header};
    use crate::types::{JceHeader, JceType};
    use bytes::{Buf, BufMut};

    fn read_bytes_len<B: Buf>(
        buf: &mut B,
        t: u8,
        struct_name: &'static str,
        field: &'static str,
    ) -> DecodeResult<usize> {
        let len = match t {
            super::SHORT_BYTES => {
                check_buf_zero(buf)?;

                buf.get_u8() as usize
            }
            super::LONG_BYTES => {
                check_buf(buf, 4)?;

                buf.get_u32() as usize
            }
            _ => {
                return Err(DecodeError::TypeIncorrect {
                    struct_name,
                    field,
                    val_type: t,
                })
            }
        };

        Ok(len)
    }

    fn read_slice<B: Buf>(buf: &mut B, value: &mut [u8], len: usize) -> DecodeResult<()> {
        if buf.remaining() < len {
            return Err(DecodeError::Eof);
        }

        value[..len].copy_from_slice(&buf.chunk()[..len]);

        Ok(())
    }

    fn write_slice<B: BufMut>(buf: &mut B, value: &[u8], tag: u8) {
        if value.is_empty() {
            write_empty(buf, tag);
            return;
        }

        write_header(buf, JceHeader { val_type: 0, tag });

        match u8::try_from(value.len()) {
            Ok(len) => {
                buf.put_u8(len);
            }
            Err(_) => {
                let len = value.len() as u32;
                buf.put_u32(len);
            }
        }

        buf.put_slice(value);
    }

    impl JceType for Vec<u8> {
        fn read<B: Buf>(
            buf: &mut B,
            t: u8,
            struct_name: &'static str,
            field: &'static str,
        ) -> DecodeResult<Vec<u8>> {
            if t == super::EMPTY {
                return Ok(vec![]);
            }

            let len = read_bytes_len(buf, t, struct_name, field)?;

            let mut v = vec![0u8; len];
            read_slice(buf, &mut v, len)?;

            Ok(v)
        }

        fn write<B: BufMut>(&self, buf: &mut B, tag: u8) {
            write_slice(buf, self, tag);
        }
    }

    impl JceType for bytes::Bytes {
        fn read<B: Buf>(
            buf: &mut B,
            t: u8,
            struct_name: &'static str,
            field: &'static str,
        ) -> DecodeResult<Self> {
            <Vec<u8> as JceType>::read(buf, t, struct_name, field).map(bytes::Bytes::from)
        }

        fn write<B: BufMut>(&self, buf: &mut B, tag: u8) {
            write_slice(buf, self, tag);
        }
    }

    impl<const N: usize> JceType for [u8; N] {
        fn read<B: Buf>(
            buf: &mut B,
            t: u8,
            struct_name: &'static str,
            field: &'static str,
        ) -> DecodeResult<Self> {
            let len = read_bytes_len(buf, t, struct_name, field)?;

            if len != N {
                return Err(DecodeError::WrongLength);
            }

            let mut arr = [0u8; N];
            read_slice(buf, &mut arr, N)?;

            Ok(arr)
        }

        fn write<B: BufMut>(&self, buf: &mut B, tag: u8) {
            write_slice(buf, self, tag);
        }
    }
}

mod jce_struct {
    use crate::error::{DecodeError, DecodeResult};
    use crate::ser::{write_header, write_type};
    use crate::types::{JceHeader, JceType};
    use crate::JceStruct;
    use bytes::{Buf, BufMut};

    impl<S: JceStruct> JceType for S {
        fn read<B: Buf>(
            buf: &mut B,
            t: u8,
            struct_name: &'static str,
            field: &'static str,
        ) -> DecodeResult<Self> {
            if t == super::EMPTY {
                return S::decode_raw(&mut &[super::STRUCT_END][..], false);
            }

            if t != super::STRUCT_START {
                return Err(DecodeError::TypeIncorrect {
                    struct_name,
                    field,
                    val_type: t,
                });
            }

            S::decode_raw(buf, false)
        }

        fn write<B: BufMut>(&self, buf: &mut B, tag: u8) {
            write_header(
                buf,
                JceHeader {
                    val_type: super::STRUCT_START,
                    tag,
                },
            );
            self.encode_raw(buf);
            write_type(buf, super::STRUCT_END);
        }
    }
}

pub fn skip_field<B: Buf>(buf: &mut B, t: u8) -> DecodeResult<()> {
    fn read_type<B: Buf>(buf: &mut B) -> DecodeResult<u8> {
        check_buf_zero(buf)?;
        Ok(buf.get_u8() & 0xF)
    }

    fn skip_elem<B: Buf>(buf: &mut B) -> DecodeResult<()> {
        let t = read_type(buf)?;

        skip_field(buf, t)
    }

    match t {
        BYTE => buf.advance(1),
        SHORT => buf.advance(2),
        INT | FLOAT => buf.advance(4),
        LONG | DOUBLE => buf.advance(8),
        SHORT_BYTES => {
            let len = buf.get_u8() as usize;
            check_buf(buf, len)?;
            buf.advance(len);
        }
        LONG_BYTES => {
            let len = buf.get_u32() as usize;
            check_buf(buf, len)?;
            buf.advance(len);
        }
        STRUCT_START => {
            skip_elem(buf)?;
        }
        STRUCT_END | EMPTY => {}
        MAP => {
            let len = read_len(buf)?;

            for _ in 0..len * 2 {
                // skip key and value
                skip_elem(buf)?;
            }
        }
        LIST => {
            let len = read_len(buf)?;

            for _ in 0..len {
                skip_elem(buf)?;
            }
        }
        SINGLE_LIST => {
            let tt = read_type(buf)?;

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
        _ => return Err(DecodeError::NoSuchType),
    }

    Ok(())
}
