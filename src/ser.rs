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
            val_type: crate::types::EMPTY,
            tag,
        },
    );
}

pub fn write_type<B: BufMut>(buf: &mut B, t: u8) {
    buf.put_u8(t);
}
