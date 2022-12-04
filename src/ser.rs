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
