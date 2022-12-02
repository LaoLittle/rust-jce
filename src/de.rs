use crate::error::{DecodeError, DecodeResult};
use crate::{check_type, types};
use bytes::Buf;

#[derive(Debug)]
pub struct JceHeader {
    val_type: u8,
    tag: u8,
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

pub fn read_header<B>(buf: &mut B) -> DecodeResult<JceHeader>
where
    B: Buf,
{
    check_buf_zero(buf)?;

    let head = buf.get_u8();

    let t = head & 0xF;
    let mut tag = head >> 4; // 直接获取尾四个bits

    if tag == 0xF {
        check_buf_zero(buf)?;

        tag = buf.get_u8();
    }

    Ok(JceHeader { val_type: t, tag })
}

pub fn read_len<B: Buf>(buf: &mut B) -> DecodeResult<usize> {
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
        _ => return Err(DecodeError::FieldTypeIncorrect),
    };

    Ok(len)
}

#[cfg(test)]
mod tests {
    use crate::de::read_header;

    #[test]
    fn int() {
        let head = [58];
        let header = read_header(&mut head.as_ref());

        println!("{:?}", header);
    }
}
