use crate::de::{read_header, read_len};
use crate::error::{DecodeError, DecodeResult};
use crate::types;
use bytes::{Buf, Bytes};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bytes(Bytes),
    Struct(HashMap<u8, Value>),
    Map(HashMap<String, Value>),
    List(Vec<Value>),
    Empty,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Byte(a), Value::Byte(b)) => a == b,
            (Value::Short(a), Value::Short(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Long(a), Value::Long(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Double(a), Value::Double(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::Struct(a), Value::Struct(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Empty, Value::Empty) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Byte(b) => b.hash(state),
            Value::Short(s) => s.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Long(l) => l.hash(state),
            Value::Float(f) => {
                state.write_u8(b'f');
                state.write_u32(f.to_bits());
            }
            Value::Double(d) => {
                state.write_u8(b'd');
                state.write_u64(d.to_bits());
            }
            Value::Bytes(b) => b.hash(state),
            Value::Struct(s) => {
                state.write_usize(s.len());
                state.write_u8(b's');

                for (k, v) in s.iter() {
                    state.write_u8(*k);
                    v.hash(state);
                }
            }
            Value::Map(m) => {
                state.write_usize(m.len());
                state.write_u8(b'm');

                for (k, v) in m.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::List(v) => {
                for val in v {
                    val.hash(state);
                }
            }
            Value::Empty => state.write(b"empty"),
        }
    }
}

fn read_value<B: Buf>(buf: &mut B, t: u8) -> DecodeResult<Value> {
    fn bytes_from_buf<B: Buf>(buf: &mut B, len: usize) -> Bytes {
        if len > 0 {
            let b = Bytes::from(buf.chunk()[..len].to_vec());
            buf.advance(len);
            b
        } else {
            Bytes::new()
        }
    }

    let val = match t {
        types::BYTE => Value::Byte(buf.get_i8()),
        types::SHORT => Value::Short(buf.get_i16()),
        types::INT => Value::Int(buf.get_i32()),
        types::LONG => Value::Long(buf.get_i64()),
        types::FLOAT => Value::Float(buf.get_f32()),
        types::DOUBLE => Value::Double(buf.get_f64()),
        types::SHORT_BYTES => Value::Bytes({
            let len = buf.get_u8() as usize;

            bytes_from_buf(buf, len)
        }),
        types::LONG_BYTES => Value::Bytes({
            let len = buf.get_u32() as usize;

            bytes_from_buf(buf, len)
        }),
        types::MAP => Value::Map({
            let len = read_len(buf)?;

            let mut map = HashMap::new();

            for _ in 0..len {
                let key = read_elem(buf)?;
                let value = read_elem(buf)?;

                let s = if let Value::Bytes(b) = key {
                    let str = std::str::from_utf8(&b)?;

                    str.to_owned()
                } else {
                    return Err(DecodeError::InvalidType);
                };

                map.insert(s, value);
            }

            map
        }),
        types::LIST => Value::List({
            let len = read_len(buf)?;

            let mut list = Vec::with_capacity(len);

            for _ in 0..len {
                list.push(read_elem(buf)?);
            }

            list
        }),
        types::EMPTY => Value::Empty,
        _ => return Err(DecodeError::InvalidType),
    };

    Ok(val)
}

pub fn read_elem<B: Buf>(buf: &mut B) -> DecodeResult<Value> {
    let t = buf.get_u8() & 0xF;
    read_value(buf, t)
}

pub fn read_to_hashmap<B: Buf>(mut buf: B) -> DecodeResult<HashMap<u8, Value>> {
    let mut map = HashMap::new();

    while buf.remaining() > 0 {
        let header = read_header(&mut buf)?;
        let value = read_value(&mut buf, header.value_type())?;

        map.insert(header.tag(), value);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use crate::value::read_to_hashmap;

    #[test]
    fn de() {
        let bytes: &[u8] = &[
            25, 0, 5, 0, 1, 0, 2, 6, 6, 232, 182, 133, 229, 184, 130, 6, 6, 49, 49, 52, 53, 49, 52,
            2, 0, 29, 75, 66, 40, 0, 2, 6, 1, 49, 18, 0, 1, 191, 82, 6, 4, 49, 57, 49, 57, 17, 3,
            42,
        ];

        let s = read_to_hashmap(bytes);

        assert!(s.is_ok());
    }

    #[test]
    fn de2() {
        let bytes = [0, 127, 24, 12];

        assert!(read_to_hashmap(bytes.as_ref()).is_ok());
    }
}
