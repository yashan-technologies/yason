use core::fmt::{self, Debug, Display};
use core::result;
use std::collections::TryReserveError;

pub type Result<T> = result::Result<T, JsonParseError>;

/// This type represents error that can be raised during parsing json string.
#[derive(Debug)]
pub struct JsonParseError {
    code: ErrorCode,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// EOF while parsing a string.
    EofWhileParsingString,

    /// EOF while parsing a JSON value.
    EofWhileParsingValue,

    /// Expected this character to be a `':'`.
    ExpectedColon,

    /// Expected this character to be either a `','` or a `']'`.
    ExpectedListCommaOrEnd,

    /// Expected this character to be either a `','` or a `'}'`.
    ExpectedObjectCommaOrEnd,

    /// Expected to parse either a `true`, `false`, or a `null`.
    ExpectedSomeIdent,

    /// Expected this character to start a JSON value.
    ExpectedSomeValue,

    /// Invalid hex escape code.
    InvalidEscape,

    /// Invalid number.
    InvalidNumber,

    /// Invalid unicode code point.
    InvalidUnicodeCodePoint,

    /// Control character found while parsing a string.
    ControlCharacterWhileParsingString,

    /// Object key is not a string.
    KeyMustBeAString,

    /// Lone leading surrogate in hex escape.
    LoneLeadingSurrogateInHexEscape,

    /// JSON has non-whitespace trailing characters after the value.
    TrailingCharacters,

    /// Unexpected end of hex escape.
    UnexpectedEndOfHexEscape,

    /// alloc vec failed
    TryReserveError,

    /// string from utf8 failed
    FromUtf8Error,

    /// Out of children counts of object/array
    TooManyChildren,
}

impl JsonParseError {
    #[inline]
    pub(crate) const fn new(code: ErrorCode, line: usize, column: usize) -> Self {
        JsonParseError { code, line, column }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) const fn errcode(&self) -> ErrorCode {
        self.code
    }
}

impl Display for ErrorCode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::EofWhileParsingString => f.write_str("EOF while parsing a string"),
            ErrorCode::EofWhileParsingValue => f.write_str("EOF while parsing a value"),
            ErrorCode::ExpectedColon => f.write_str("expected `:`"),
            ErrorCode::ExpectedListCommaOrEnd => f.write_str("expected `,` or `]`"),
            ErrorCode::ExpectedObjectCommaOrEnd => f.write_str("expected `,` or `}`"),
            ErrorCode::ExpectedSomeIdent => f.write_str("expected ident"),
            ErrorCode::ExpectedSomeValue => f.write_str("expected value"),
            ErrorCode::InvalidEscape => f.write_str("invalid escape"),
            ErrorCode::InvalidNumber => f.write_str("invalid number"),
            ErrorCode::InvalidUnicodeCodePoint => f.write_str("invalid unicode code point"),
            ErrorCode::KeyMustBeAString => f.write_str("key must be a string"),
            ErrorCode::LoneLeadingSurrogateInHexEscape => f.write_str("lone leading surrogate in hex escape"),
            ErrorCode::TrailingCharacters => f.write_str("trailing characters"),
            ErrorCode::UnexpectedEndOfHexEscape => f.write_str("unexpected end of hex escape"),
            ErrorCode::TryReserveError => f.write_str("TryReserveError"),
            ErrorCode::FromUtf8Error => f.write_str("string FromUtf8Error"),
            ErrorCode::ControlCharacterWhileParsingString => {
                f.write_str("control character (\\u0000-\\u001F) found while parsing a string")
            }
            ErrorCode::TooManyChildren => f.write_str("too many children in the current object or array"),
        }
    }
}

impl Display for JsonParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "JSON syntax error at line {} column {}: {}",
            self.line, self.column, self.code
        )
    }
}

impl From<TryReserveError> for JsonParseError {
    #[inline]
    fn from(_: TryReserveError) -> Self {
        Self {
            code: ErrorCode::TryReserveError,
            line: 0,
            column: 0,
        }
    }
}
