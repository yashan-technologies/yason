//! Basic functions.

use crate::vec::VecExt;
use crate::yason::YasonResult;
use crate::{YasonError, MAX_DATA_LENGTH_SIZE};
use std::cmp::Ordering;

#[inline]
pub fn cmp_key(left: &str, right: &str) -> Ordering {
    match left.len().cmp(&right.len()) {
        Ordering::Equal => left.cmp(right),
        Ordering::Greater => Ordering::Greater,
        Ordering::Less => Ordering::Less,
    }
}

#[inline]
pub fn encode_varint(mut value: u32, buf: &mut Vec<u8>) {
    if value < 0x80 {
        buf.push_u8(value as u8);
        return;
    }

    const SHIFT: [u8; 4] = [24, 16, 8, 0];

    let mut res: u32 = 0;
    let mut len = 0;

    for (i, shift) in SHIFT.iter().enumerate() {
        let mut ch = value & 0x7f;

        value >>= 7;
        if value != 0 {
            ch |= 0x80
        }

        ch <<= shift;
        res |= ch;

        if value == 0 {
            len = i + 1;
            break;
        }
    }

    debug_assert!(value == 0);

    let bytes = &res.to_be_bytes()[..len];
    buf.push_bytes(bytes);
}

#[inline]
pub fn decode_varint(buf: &[u8], index: usize) -> YasonResult<(u32, usize)> {
    debug_assert!(index < buf.len());

    let mut data_length: u32 = 0;
    for i in 0..MAX_DATA_LENGTH_SIZE {
        // Get the next 7 bits of the length.
        let byte = buf.get(index + i).map_or_else(
            || {
                Err(YasonError::IndexOutOfBounds {
                    len: buf.len(),
                    index: index + i,
                })
            },
            |v| Ok(*v),
        )?;
        data_length |= (byte as u32 & 0x7f) << (7 * i);
        if (byte & 0x80) == 0 {
            // This was the last byte. Return successfully.
            return Ok((data_length, i + 1));
        }
    }
    unreachable!("data length read error");
}

#[cfg(test)]
mod tests {
    use crate::util::{decode_varint, encode_varint};

    fn assert_varint(value: u32, expected: &[u8]) {
        let mut buf = Vec::with_capacity(4);
        encode_varint(value, &mut buf);
        assert_eq!(&buf, expected);

        let (val, len) = decode_varint(&buf, 0).unwrap();
        assert_eq!(val, value);
        assert_eq!(len, expected.len());
    }

    #[test]
    fn test_varint() {
        assert_varint(10, &[10]);
        assert_varint(500, &[244, 3]);
        assert_varint(20000, &[160, 156, 1]);
        assert_varint(250000000, &[128, 229, 154, 119]);
    }
}
