//! Basic functions.

use crate::binary::MAX_DATA_LENGTH_SIZE;
use crate::vec::VecExt;
use crate::yason::YasonResult;
use crate::YasonError;
use std::cmp::Ordering;

#[rustfmt::skip]
static HEX: [Option<u8>; 256] = {
    use Option::Some as S;
    const __: Option<u8> = None; // not a hex digit
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        S(0), S(1), S(2), S(3), S(4), S(5), S(6), S(7), S(8), S(9), __, __, __, __, __, __, // 3
        __, S(10), S(11), S(12), S(13), S(14), S(15), __, __, __, __, __, __, __, __, __,   // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
        __, S(10), S(11), S(12), S(13), S(14), S(15), __, __, __, __, __, __, __, __, __,   // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

// Lookup table of bytes that must be escaped. A value of true at index i means
// that byte i requires an escape sequence in the input.
static ESCAPE: [bool; 256] = {
    const CT: bool = true; // control character \x00..=\x1F
    const QU: bool = true; // quote \x22
    const BS: bool = true; // backslash \x5C
    const __: bool = false; // allow unescaped
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

#[inline(always)]
pub fn decode_hex_val(val: u8) -> Option<u8> {
    HEX[val as usize]
}

#[inline(always)]
pub fn is_escape(val: u8) -> bool {
    ESCAPE[val as usize]
}

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
