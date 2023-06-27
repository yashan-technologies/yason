//! Base64 codec.

const ENCODE_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const DECODE_TABLE: [u8; 256] = build_decode_table();
const ENCODE_CHUNK_SIZE: usize = 3;
const DECODE_CHUNK_SIZE: usize = 4;
const ENCODE_CHUNK_MASK: u32 = 0x3F;
const PADDING: u8 = b'=';
const INVALID_VALUE: u8 = 255;

const fn build_decode_table() -> [u8; 256] {
    let mut table = [INVALID_VALUE; 256];
    let mut index = 0;
    while index < 64 {
        table[ENCODE_TABLE[index] as usize] = index as u8;
        index += 1;
    }
    table
}

/// Calculate the base64 encoded length for a given input length.
///
/// Notes that we always add padding bytes.
#[inline]
pub const fn encoded_len(input_len: usize) -> usize {
    let len = input_len / ENCODE_CHUNK_SIZE * DECODE_CHUNK_SIZE;
    let rem = input_len % ENCODE_CHUNK_SIZE;
    if rem > 0 {
        len + DECODE_CHUNK_SIZE
    } else {
        len
    }
}

#[inline]
pub const fn decoded_len_estimate(input_len: usize) -> usize {
    let rem = input_len % DECODE_CHUNK_SIZE;
    // When "aaaab", last "b" is useless and discarded
    let chunk_len = input_len / DECODE_CHUNK_SIZE + (rem > 1) as usize;
    chunk_len * ENCODE_CHUNK_SIZE
}

/// Notes that we always add padding bytes.
pub fn encode(input: &[u8], output: &mut [u8]) -> usize {
    debug_assert!(encoded_len(input.len()) <= output.len());

    let iter = input.chunks_exact(ENCODE_CHUNK_SIZE);
    let rem = iter.remainder();

    let mut i = 0;
    for chunk in iter {
        let int = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        output[i] = ENCODE_TABLE[(int >> 18) as usize];
        output[i + 1] = ENCODE_TABLE[((int >> 12) & ENCODE_CHUNK_MASK) as usize];
        output[i + 2] = ENCODE_TABLE[((int >> 6) & ENCODE_CHUNK_MASK) as usize];
        output[i + 3] = ENCODE_TABLE[(int & ENCODE_CHUNK_MASK) as usize];
        i += 4;
    }

    let rem_len = rem.len();
    if rem_len == 2 {
        let int = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
        output[i] = ENCODE_TABLE[(int >> 18) as usize];
        output[i + 1] = ENCODE_TABLE[((int >> 12) & ENCODE_CHUNK_MASK) as usize];
        output[i + 2] = ENCODE_TABLE[((int >> 6) & ENCODE_CHUNK_MASK) as usize];
        output[i + 3] = PADDING;
        i += 4
    } else if rem_len == 1 {
        let int = rem[0];
        output[i] = ENCODE_TABLE[(int >> 2) as usize];
        output[i + 1] = ENCODE_TABLE[(((int << 4) as u32) & ENCODE_CHUNK_MASK) as usize];
        output[i + 2] = PADDING;
        output[i + 3] = PADDING;
        i += 4;
    }

    i
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidByte(u8),
}

#[inline]
fn decode_byte(byte: u8) -> Result<u32, DecodeError> {
    let dec = DECODE_TABLE[byte as usize];
    if dec == INVALID_VALUE {
        Err(DecodeError::InvalidByte(byte))
    } else {
        Ok(dec as u32)
    }
}

pub fn decode(input: &[u8], output: &mut [u8]) -> Result<usize, DecodeError> {
    debug_assert!(decoded_len_estimate(input.len()) <= output.len());

    let mut iter = input.chunks_exact(DECODE_CHUNK_SIZE);
    let rem = iter.remainder();

    let mut rem_bytes = if input.len() >= DECODE_CHUNK_SIZE && rem.is_empty() && Some(&PADDING) == input.last() {
        // SAFETY: iter should have at least 1 chunk.
        iter.next_back().unwrap()
    } else {
        rem
    };

    let mut i = 0;
    for chunk in iter {
        let (c0, c1, c2, c3) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        let bit24 = (decode_byte(c0)? << 18) | (decode_byte(c1)? << 12) | (decode_byte(c2)? << 6) | decode_byte(c3)?;
        output[i] = (bit24 >> 16) as u8;
        output[i + 1] = ((bit24 >> 8) & 0xFF) as u8;
        output[i + 2] = (bit24 & 0xFF) as u8;
        i += 3;
    }

    if !rem_bytes.is_empty() {
        while Some(&PADDING) == rem_bytes.last() {
            rem_bytes = rem_bytes.split_last().unwrap().1;
        }
        let rem_len = rem_bytes.len();
        if rem_len == 3 {
            let (c0, c1, c2) = (rem_bytes[0], rem_bytes[1], rem_bytes[2]);
            let bit24 = (decode_byte(c0)? << 18) | (decode_byte(c1)? << 12) | (decode_byte(c2)? << 6);
            output[i] = (bit24 >> 16) as u8;
            output[i + 1] = ((bit24 >> 8) & 0xFF) as u8;
            i += 2;
        } else if rem_len == 2 {
            let (c0, c1) = (rem_bytes[0], rem_bytes[1]);
            output[i] = (decode_byte(c0)? << 2) as u8 | (decode_byte(c1)? >> 4) as u8;
            i += 1;
        }

        // When "aaaab", last "b" is useless and discarded.  Compatibility with Oracle & Mongo.  Called "Lax"
        // decoding in yason.
    }

    Ok(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn print_decode_table() {
        let m = build_decode_table();
        for c in m.chunks(16) {
            println!("{:?}", c);
        }
    }

    #[test]
    fn test_codec_len() {
        fn assert_codec_len(input_len: usize, expected_len: usize) {
            let encoded_len = encoded_len(input_len);
            assert_eq!(encoded_len, expected_len);
            let decoded_len_estimate = decoded_len_estimate(encoded_len);
            assert!(decoded_len_estimate >= input_len);
            assert!(decoded_len_estimate - input_len < 3);
        }

        assert_codec_len(0, 0);
        assert_codec_len(1, 4);
        assert_codec_len(2, 4);
        assert_codec_len(3, 4);
        assert_codec_len(4, 8);
        assert_codec_len(5, 8);
        assert_codec_len(6, 8);
        assert_codec_len(7, 12);
        assert_codec_len(8, 12);
        assert_codec_len(9, 12);
        assert_codec_len(54, 72);
        assert_codec_len(55, 76);
        assert_codec_len(56, 76);
        assert_codec_len(57, 76);
        assert_codec_len(58, 80);
    }

    #[test]
    fn test_codec() {
        fn assert_codec(input: &[u8], expected_encoded: &[u8]) {
            let calculated_encoded_len = encoded_len(input.len());
            let mut encode_buf = vec![0u8; calculated_encoded_len];
            let encoded_len = encode(input, &mut encode_buf[0..calculated_encoded_len]);
            assert_eq!(encoded_len, calculated_encoded_len);
            assert_eq!(expected_encoded, &encode_buf[0..encoded_len]);

            let estimated_decoded_len = decoded_len_estimate(encoded_len);
            let mut decode_buf = vec![0u8; estimated_decoded_len];
            let decoded_len = decode(&encode_buf[0..encoded_len], &mut decode_buf[0..estimated_decoded_len]).unwrap();
            assert!(decoded_len <= estimated_decoded_len);
            assert!(estimated_decoded_len - decoded_len < 3);
            assert_eq!(input, &decode_buf[0..decoded_len]);
        }

        fn assert_decode(input: &[u8], expected_encoded: &[u8]) {
            let estimated_decoded_len = decoded_len_estimate(input.len());
            let mut decode_buf = vec![0u8; estimated_decoded_len];
            let decoded_len = decode(input, &mut decode_buf[0..estimated_decoded_len]).unwrap();
            assert!(decoded_len <= estimated_decoded_len);
            assert!(estimated_decoded_len - decoded_len < 3);
            assert_eq!(expected_encoded, &decode_buf[0..decoded_len]);
        }

        assert_codec(b"", b"");
        assert_codec(b"f", b"Zg==");
        assert_codec(b"fo", b"Zm8=");
        assert_codec(b"foo", b"Zm9v");
        assert_codec(b"foob", b"Zm9vYg==");
        assert_codec(b"fooba", b"Zm9vYmE=");
        assert_codec(b"foobar", b"Zm9vYmFy");
        assert_codec(b">>>>>>", b"Pj4+Pj4+");
        assert_codec(b"??????", b"Pz8/Pz8/");
        assert_codec(b"HTML", b"SFRNTA==");
        assert_codec(b"hello!", b"aGVsbG8h");
        assert_codec(b"JavaScript", b"SmF2YVNjcmlwdA==");
        assert_codec("你好世界".as_bytes(), b"5L2g5aW95LiW55WM"); // UTF-8
        assert_codec("★😔".as_bytes(), b"4piF8J+YlA==");
        assert_codec("🏆☕️💤".as_bytes(), b"8J+PhuKYle+4j/CfkqQ=");
        assert_decode(b"SmF2YVNjcmlwdA", b"JavaScript");
        assert_decode(b"s", b"");

        {
            const DE: &[u8; 9] = b"JavaScrip";
            assert_decode(b"SmF2YVNjcmlwd", DE);
            assert_decode(b"SmF2YVNjcmlw", DE);
        }
    }
}
