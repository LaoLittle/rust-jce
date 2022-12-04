use crate::error::{DecodeError, DecodeResult};
use crate::types::JceHeader;
use crate::{check_type, types};
use bytes::Buf;

#[inline]
pub(crate) fn check_buf<B: Buf>(buf: &mut B, min: usize) -> DecodeResult<()> {
    if buf.remaining() < min {
        return Err(DecodeError::Eof);
    }

    Ok(())
}

#[inline]
pub(crate) fn check_buf_zero<B: Buf>(buf: &mut B) -> DecodeResult<()> {
    check_buf(buf, 1)
}

pub fn read_header<B: Buf>(buf: &mut B) -> DecodeResult<JceHeader> {
    check_buf_zero(buf)?;

    let head = buf.get_u8();

    let t = head & 0xF;
    let mut tag = head >> 4; // 直接获取高四位

    if tag == 0xF {
        check_buf_zero(buf)?;

        tag = buf.get_u8();
    }

    Ok(JceHeader { val_type: t, tag })
}

pub(crate) fn read_len<B: Buf>(buf: &mut B) -> DecodeResult<usize> {
    check_buf_zero(buf)?;

    let len_type = buf.get_u8();

    let len = match len_type {
        types::BYTE => {
            check_type!(u8, buf);
            buf.get_u8() as usize
        }
        types::SHORT => {
            check_type!(u16, buf);
            buf.get_u16() as usize
        }
        types::INT => {
            check_type!(u32, buf);
            buf.get_u32() as usize
        }
        types::LONG => {
            check_type!(u64, buf);
            buf.get_u64() as usize
        }
        types::EMPTY => 0usize,
        _ => return Err(DecodeError::InvalidType),
    };

    Ok(len)
}

#[cfg(test)]
mod tests {
    use crate::de::read_header;

    #[test]
    fn int() {
        let head = [58];
        let header = read_header(&mut head.as_ref()).unwrap();

        assert_eq!(header.value_type(), 10);
        assert_eq!(header.tag(), 3);
    }
}
