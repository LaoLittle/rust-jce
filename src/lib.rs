use crate::error::DecodeResult;

pub mod bytes;
pub mod de;
pub mod error;
mod macros;
pub mod types;
pub mod value;

use ::bytes::{Buf, BufMut};
pub use jce_derive::JceStruct;

pub trait JceStruct: Sized {
    fn encode_raw<B: BufMut>(&self, buf: &mut B);

    fn encoded_len(&self) -> usize;

    fn decode_raw<B: Buf>(buf: &mut B, to_end: bool) -> DecodeResult<Self>;

    fn decode<B: Buf>(mut buf: B) -> DecodeResult<Self> {
        Self::decode_raw(&mut buf, true)
    }
}
