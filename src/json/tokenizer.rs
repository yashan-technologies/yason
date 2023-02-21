use std::borrow::Cow;

use crate::{
    util::{decode_hex_val, is_escape},
    vec::VecExt,
};

use super::{
    error::{ErrorCode, JsonParseError, Result},
    read::{Position, SliceRead},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Null,
    Comma,
    Colon,
    BracketOn,
    BracketOff,
    BraceOn,
    BraceOff,
    String(Cow<'a, str>),
    Number(&'a str),
    Boolean(bool),
}

pub struct Tokenizer<'a> {
    source: SliceRead<'a>,
}

impl<'a> Tokenizer<'a> {
    #[inline]
    pub fn new(s: &'a str) -> Self {
        Self {
            source: SliceRead::new(s.as_bytes()),
        }
    }

    #[inline]
    fn step(&mut self) -> Result<u8> {
        self.source
            .next()
            .ok_or_else(|| self.error(ErrorCode::EofWhileParsingString))
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.source.peek()
    }

    #[inline]
    fn advance(&mut self) {
        self.source.advance()
    }

    #[inline]
    fn interval(&self, start: usize, end: usize) -> Result<&'a [u8]> {
        if let Some(slice) = self.source.get_interval(start, end) {
            Ok(slice)
        } else {
            Err(self.error(ErrorCode::EofWhileParsingValue))
        }
    }

    #[inline]
    fn read_ident(&mut self, expect_ident: &[u8]) -> Result<()> {
        let start = self.source.index();
        let end = start + expect_ident.len();

        if expect_ident != self.interval(start, end)? {
            return Err(self.error(ErrorCode::ExpectedSomeIdent));
        }

        self.source.step_forward(expect_ident.len());

        Ok(())
    }

    #[inline]
    fn str_from_utf8(&self, s: &'a [u8], pos: Position) -> Result<&'a str> {
        std::str::from_utf8(s).map_err(|_| self.error_with_position(ErrorCode::FromUtf8Error, pos))
    }

    #[inline]
    fn string_from_utf8(&self, s: Vec<u8>, pos: Position) -> Result<String> {
        String::from_utf8(s).map_err(|_| self.error_with_position(ErrorCode::FromUtf8Error, pos))
    }

    #[inline]
    fn read_string(&mut self) -> Result<Token<'a>> {
        let mut start = self.source.index();
        let mut str_cow: Cow<[u8]> = Cow::Borrowed(&[]);
        let old_position = self.source.position();

        loop {
            let ch = self.step()?;

            if ch == b'"' {
                break;
            } else if ch == b'\\' {
                str_cow
                    .to_mut()
                    .try_extend_from_slice(self.interval(start, self.source.index() - 1)?)?;

                self.read_escape_string(&mut str_cow)?;

                start = self.source.index();
            } else if is_escape(ch) {
                if ch == b'\n' {
                    self.source.new_line();
                }
                return Err(self.error(ErrorCode::ControlCharacterWhileParsingString));
            }
        }

        match str_cow {
            Cow::Borrowed(_) => {
                let str = self.str_from_utf8(self.interval(start, self.source.index() - 1)?, old_position)?;
                Ok(Token::String(Cow::Borrowed(str)))
            }
            Cow::Owned(mut str) => {
                str.try_extend_from_slice(self.interval(start, self.source.index() - 1)?)?;
                Ok(Token::String(Cow::Owned(self.string_from_utf8(str, old_position)?)))
            }
        }
    }

    #[inline]
    fn next_expect_char(&mut self, expect: u8) -> Result<()> {
        match self.source.next() {
            Some(c) => {
                if c != expect {
                    return Err(self.error(ErrorCode::UnexpectedEndOfHexEscape));
                }
            }
            None => {
                return Err(self.error(ErrorCode::EofWhileParsingString));
            }
        }

        Ok(())
    }

    #[inline]
    fn read_escape_string(&mut self, scratch: &mut Cow<[u8]>) -> Result<()> {
        if let Some(ch) = self.source.next() {
            match ch {
                b'"' => scratch.to_mut().try_push(b'"')?,
                b'\\' => scratch.to_mut().try_push(b'\\')?,
                b'/' => scratch.to_mut().try_push(b'/')?,
                b'b' => scratch.to_mut().try_push(b'\x08')?,
                b'f' => scratch.to_mut().try_push(b'\x0c')?,
                b'n' => scratch.to_mut().try_push(b'\n')?,
                b'r' => scratch.to_mut().try_push(b'\r')?,
                b't' => scratch.to_mut().try_push(b'\t')?,
                b'u' => {
                    let c = match self.decode_hex_escape()? {
                        0xDC00..=0xDFFF => {
                            return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                        }
                        n1 @ 0xD800..=0xDBFF => {
                            self.next_expect_char(b'\\')?;
                            self.next_expect_char(b'u')?;

                            let n2 = self.decode_hex_escape()?;

                            if !(0xDC00..=0xDFFF).contains(&n2) {
                                return Err(self.error(ErrorCode::LoneLeadingSurrogateInHexEscape));
                            }

                            let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;
                            match char::from_u32(n) {
                                Some(c) => c,
                                None => {
                                    return Err(self.error(ErrorCode::InvalidUnicodeCodePoint));
                                }
                            }
                        }
                        // Every u16 outside of the surrogate ranges above is guaranteed
                        // to be a legal char.
                        n => char::from_u32(n as u32).unwrap(),
                    };

                    scratch
                        .to_mut()
                        .try_extend_from_slice(c.encode_utf8(&mut [0_u8; 4]).as_bytes())?;
                }
                _ => {
                    return Err(self.error(ErrorCode::InvalidEscape));
                }
            }
        } else {
            return Err(self.error(ErrorCode::EofWhileParsingString));
        }

        Ok(())
    }

    #[inline]
    fn decode_hex_escape(&mut self) -> Result<u16> {
        let mut n: u16 = 0;

        for _ in 0..4 {
            let ch = self.step()?;

            if let Some(val) = decode_hex_val(ch) {
                n = (n << 4) + val as u16;
            } else {
                return Err(self.error(ErrorCode::InvalidEscape));
            }
        }

        Ok(n)
    }

    #[inline]
    fn get_number_from_interval(&self, start: usize, end: usize) -> Result<&'a str> {
        if let Some(n) = self.source.get_interval(start, end) {
            if let Ok(s) = std::str::from_utf8(n) {
                return Ok(s);
            }
        }

        Err(self.error(ErrorCode::InvalidNumber))
    }

    #[inline]
    fn read_number(&mut self, mut begin: u8) -> Result<Token<'a>> {
        let start = self.source.index() - 1;

        if begin == b'-' {
            begin = self.step()?;
        }
        match begin {
            b'0' => match self.peek() {
                // There can be only one leading '0'.
                Some(b'0'..=b'9') => Err(self.peek_error(ErrorCode::InvalidNumber)),
                _ => self.read_decimal(start),
            },
            b'1'..=b'9' => loop {
                match self.peek() {
                    Some(b'0'..=b'9') => self.advance(),
                    _ => {
                        return self.read_decimal(start);
                    }
                }
            },
            _ => Err(self.error(ErrorCode::InvalidNumber)),
        }
    }

    #[inline]
    fn read_decimal(&mut self, start: usize) -> Result<Token<'a>> {
        match self.peek() {
            Some(b'.') => self.read_fraction(start),
            Some(b'e' | b'E') => self.read_exponent(start),
            _ => Ok(Token::Number(
                self.get_number_from_interval(start, self.source.index())?,
            )),
        }
    }

    #[inline]
    fn read_fraction(&mut self, start: usize) -> Result<Token<'a>> {
        self.advance();

        // if there is exponent next to decimal point, we return failed
        match self.peek() {
            Some(b'0'..=b'9') => {
                self.advance();
                while let Some(b'0'..=b'9') = self.peek() {
                    self.advance();
                }
            }
            Some(_) => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        }

        match self.peek() {
            Some(b'e' | b'E') => self.read_exponent(start),
            _ => Ok(Token::Number(
                self.get_number_from_interval(start, self.source.index())?,
            )),
        }
    }

    #[inline]
    fn read_exponent(&mut self, start: usize) -> Result<Token<'a>> {
        self.advance();

        // read - or +, 0..9 do nothing
        match self.peek() {
            Some(b'+' | b'-') => {
                self.advance();
            }
            Some(b'0'..=b'9') => {}
            Some(_) => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        }

        match self.peek() {
            Some(b'0'..=b'9') => {
                self.advance();
                while let Some(b'0'..=b'9') = self.peek() {
                    self.advance();
                }
            }
            Some(_) => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        }

        Ok(Token::Number(
            self.get_number_from_interval(start, self.source.index())?,
        ))
    }

    #[inline]
    pub fn end(&mut self) -> Result<()> {
        while let Some(ch) = self.source.next() {
            if !ch.is_ascii_whitespace() {
                return Err(self.error(ErrorCode::TrailingCharacters));
            }
        }

        Ok(())
    }

    #[inline]
    pub fn error(&self, reason: ErrorCode) -> JsonParseError {
        let position = self.source.position();
        JsonParseError::new(reason, position.line, position.column)
    }

    #[inline]
    fn peek_error(&self, reason: ErrorCode) -> JsonParseError {
        let position = self.source.peek_position();
        JsonParseError::new(reason, position.line, position.column)
    }

    #[inline]
    pub fn error_with_position(&self, reason: ErrorCode, pos: Position) -> JsonParseError {
        JsonParseError::new(reason, pos.line, pos.column)
    }

    #[inline]
    pub fn next(&mut self) -> Option<Result<Token<'a>>> {
        while let Some(ch) = self.source.next() {
            return Some(match ch {
                b',' => Ok(Token::Comma),
                b':' => Ok(Token::Colon),
                b'[' => Ok(Token::BracketOn),
                b']' => Ok(Token::BracketOff),
                b'{' => Ok(Token::BraceOn),
                b'}' => Ok(Token::BraceOff),
                b'"' => self.read_string(),
                b'0'..=b'9' | b'-' => self.read_number(ch),
                b'n' => match self.read_ident(b"ull") {
                    Ok(_) => Ok(Token::Null),
                    Err(e) => Err(e),
                },
                b't' => match self.read_ident(b"rue") {
                    Ok(_) => Ok(Token::Boolean(true)),
                    Err(e) => Err(e),
                },
                b'f' => match self.read_ident(b"alse") {
                    Ok(_) => Ok(Token::Boolean(false)),
                    Err(e) => Err(e),
                },
                b'\n' => {
                    self.source.new_line();
                    continue;
                }
                _ => {
                    if ch.is_ascii_whitespace() {
                        continue;
                    } else {
                        Err(self.error(ErrorCode::ExpectedSomeValue))
                    }
                }
            });
        }

        //None means eof
        None
    }
}
