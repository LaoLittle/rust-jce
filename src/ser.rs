use crate::types;
use crate::types::JceHeader;
use bytes::BufMut;

pub fn write_header<B: BufMut>(buf: &mut B, JceHeader { val_type, tag }: JceHeader) {
    if tag < 0xF {
        buf.put_u8((tag << 4) | val_type)
    } else {
        buf.put_u8(val_type | 0xF0);
        buf.put_u8(tag);
    }
}

pub fn write_empty<B: BufMut>(buf: &mut B, tag: u8) {
    write_header(
        buf,
        JceHeader {
            val_type: types::EMPTY,
            tag,
        },
    );
}

pub fn write_type<B: BufMut>(buf: &mut B, t: u8) {
    buf.put_u8(t);
}

pub fn write_len<B: BufMut>(buf: &mut B, len: usize) {
    if let Ok(len) = u8::try_from(len) {
        buf.put_u8(types::BYTE);
        buf.put_u8(len);
    } else if let Ok(len) = u16::try_from(len) {
        buf.put_u8(types::SHORT);
        buf.put_u16(len);
    } else if let Ok(len) = u32::try_from(len) {
        buf.put_u8(types::INT);
        buf.put_u32(len);
    } else if let Ok(len) = u64::try_from(len) {
        buf.put_u8(types::LONG);
        buf.put_u64(len);
    } else {
        unreachable!();
    }
}

pub fn len_bytes(len: usize) -> usize {
    const U8: usize = u8::MAX as usize;
    const U81: usize = U8 + 1;
    const U16: usize = u16::MAX as usize;
    const U161: usize = U16 + 1;
    const U32: usize = u32::MAX as usize;
    const U321: usize = U32 + 1;
    const U64: usize = u64::MAX as usize;

    match len {
        0..=U8 => 1,
        U81..=U16 => 2,
        U161..=U32 => 4,
        U321..=U64 => 8,
        _ => unreachable!(),
    }
}
