//! Path Parser.

use crate::PathExpression;
use std::collections::TryReserveError;
use std::error::Error;
use std::fmt::{Display, Formatter};

const ROOT: u8 = b'$';
const DOT: u8 = b'.';
const COMMA: u8 = b',';
const BEGIN_ARRAY: u8 = b'[';
const END_ARRAY: u8 = b']';
const LEFT_BRACKET: u8 = b'(';
const RIGHT_BRACKET: u8 = b')';
const DOUBLE_QUOTE: u8 = b'"';
const WILDCARD: u8 = b'*';
const MINUS: u8 = b'-';
const CTRL_CHAR_LEN: usize = 1;

const LAST: &[u8] = b"last";
const TO: &[u8] = b"to";

const COUNT: &[u8] = b"count";
const SIZE: &[u8] = b"size";
const TYPE: &[u8] = b"type";

/// This type represents error that can arise during parsing path expression.
#[derive(Debug)]
pub struct PathParseError {
    kind: PathParseErrorKind,
    pos: usize,
}

impl PathParseError {
    #[inline]
    fn new(kind: PathParseErrorKind, pos: usize) -> Self {
        Self { kind, pos }
    }
}

impl Display for PathParseError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at position {}", self.kind, self.pos)
    }
}

/// Possible errors that can arise during parsing path expression.
#[derive(Debug, PartialEq)]
enum PathParseErrorKind {
    NotStartWithDollar,
    MissingSquareBracket,
    ArrayStepSyntaxError,
    ArrayIndexTooLong,
    InvalidEscapeSequence,
    UnclosedQuotedStep,
    InvalidKeyStep,
    InvalidFunction,
    UnexpectedCharacterAtEnd,
    InvalidCharacterAtStepStart,
    EmptyArrayStep,
    TryReserveError(TryReserveError),
}

impl Display for PathParseErrorKind {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathParseErrorKind::NotStartWithDollar => write!(f, "path must start with a dollar sign ($) character"),
            PathParseErrorKind::MissingSquareBracket => write!(f, "missing square bracket in array step"),
            PathParseErrorKind::ArrayStepSyntaxError => write!(f, "array step contains unexpected characters"),
            PathParseErrorKind::ArrayIndexTooLong => write!(f, "array subscript too long"),
            PathParseErrorKind::InvalidEscapeSequence => write!(f, "invalid escape sequence"),
            PathParseErrorKind::UnclosedQuotedStep => write!(f, "unclosed quoted step"),
            PathParseErrorKind::InvalidKeyStep => write!(f, "key step contains unexpected characters"),
            PathParseErrorKind::InvalidFunction => write!(f, "invalid function at the end of path"),
            PathParseErrorKind::UnexpectedCharacterAtEnd => write!(f, "unexpected characters after end of path"),
            PathParseErrorKind::InvalidCharacterAtStepStart => write!(f, "invalid character at start of step"),
            PathParseErrorKind::EmptyArrayStep => write!(f, "empty array subscript"),
            PathParseErrorKind::TryReserveError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for PathParseError {}

pub type PathParseResult<T> = std::result::Result<T, PathParseError>;

#[derive(Debug, PartialEq)]
pub enum SingleIndex {
    /// \[1]
    Index(usize),
    /// \[last - 1]
    Last(usize),
}

#[derive(Debug, PartialEq)]
pub enum SingleStep {
    /// \[1] \ [last - 1]
    Single(SingleIndex),
    /// \[1 to 4]
    Range(SingleIndex, SingleIndex),
}

#[derive(Debug, PartialEq)]
pub enum ArrayStep {
    /// \[1]
    Index(usize),
    /// \[last]
    Last(usize),
    /// \[1 to 4]
    Range(SingleIndex, SingleIndex),
    /// \[1, last, 1 to 4]
    Multiple(Vec<SingleStep>),
    /// \[*]
    Wildcard,
}

#[derive(Debug, PartialEq)]
pub enum ObjectStep {
    /// .key
    Key(String),
    /// .*
    Wildcard,
}

#[derive(Debug, PartialEq)]
pub enum FuncStep {
    Count,
    Size,
    Type,
}

#[derive(Debug, PartialEq)]
pub enum Step {
    /// $
    Root,
    /// .XXX
    Object(ObjectStep),
    /// \[XXX]
    Array(ArrayStep),
    /// ..key
    Descendent(String),
    /// .XXX()
    Func(FuncStep),
}

pub struct PathParser<'a> {
    input: &'a [u8],
    pos: usize,
    path: Vec<Step>,
}

impl<'a> PathParser<'a> {
    #[inline]
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            path: vec![],
        }
    }

    #[inline]
    pub fn parse(mut self) -> PathParseResult<PathExpression> {
        // the first non-space character must be `$`
        self.skip(|i| i == b' ');
        if self.pop() != Some(ROOT) {
            return Err(PathParseError::new(PathParseErrorKind::NotStartWithDollar, self.pos));
        }
        self.push_step(Step::Root)?;

        self.skip(|i| i == b' ');
        while !self.exhausted() {
            match self.pop() {
                Some(BEGIN_ARRAY) => self.parse_array_step()?,
                Some(DOT) => match self.peek() {
                    Some(DOT) => self.parse_descendent_step()?,
                    _ => self.parse_object_step()?,
                },
                None => {}
                _ => {
                    return Err(PathParseError::new(
                        PathParseErrorKind::InvalidCharacterAtStepStart,
                        self.pos,
                    ))
                }
            }
            self.eat_whitespaces();
        }

        Ok(PathExpression::new(self.path))
    }

    #[inline]
    fn parse_array_step(&mut self) -> PathParseResult<()> {
        self.eat_whitespaces();

        match self.peek() {
            Some(END_ARRAY) => {
                return Err(PathParseError::new(PathParseErrorKind::EmptyArrayStep, self.pos));
            }
            None => {
                return Err(PathParseError::new(PathParseErrorKind::MissingSquareBracket, self.pos));
            }
            Some(WILDCARD) => {
                self.advance(CTRL_CHAR_LEN);
                self.push_step(Step::Array(ArrayStep::Wildcard))?;
            }
            _ => {
                let mut steps = Vec::new();
                self.parse_array_cell(&mut steps)?;
                debug_assert!(!steps.is_empty());
                if steps.len() == 1 {
                    // SAFETY: steps.len() == 1
                    let step = steps.pop().unwrap();
                    match step {
                        SingleStep::Single(single_index) => match single_index {
                            SingleIndex::Index(index) => self.push_step(Step::Array(ArrayStep::Index(index)))?,
                            SingleIndex::Last(minus) => self.push_step(Step::Array(ArrayStep::Last(minus)))?,
                        },
                        SingleStep::Range(begin, end) => self.push_step(Step::Array(ArrayStep::Range(begin, end)))?,
                    };
                } else {
                    self.push_step(Step::Array(ArrayStep::Multiple(steps)))?;
                }
            }
        }

        // the next non-whitespace should be the closing ]
        self.eat_whitespaces();
        if self.pop() != Some(END_ARRAY) {
            return Err(PathParseError::new(PathParseErrorKind::MissingSquareBracket, self.pos));
        }
        Ok(())
    }

    #[inline]
    fn parse_array_cell(&mut self, steps: &mut Vec<SingleStep>) -> PathParseResult<()> {
        loop {
            let begin = self.parse_last_or_index()?;
            steps
                .try_reserve(std::mem::size_of::<Step>())
                .map_err(|e| PathParseError::new(PathParseErrorKind::TryReserveError(e), self.pos))?;

            self.eat_whitespaces();
            if self.has_keyword(TO) {
                self.advance(TO.len());
                self.eat_whitespaces();

                let end = self.parse_last_or_index()?;
                steps.push(SingleStep::Range(begin, end));
            } else {
                steps.push(SingleStep::Single(begin));
            }

            self.eat_whitespaces();
            if self.peek() == Some(COMMA) {
                self.advance(CTRL_CHAR_LEN);
                self.eat_whitespaces();
            } else {
                break;
            }
        }

        Ok(())
    }

    #[inline]
    fn parse_last_or_index(&mut self) -> PathParseResult<SingleIndex> {
        if self.has_keyword(LAST) {
            self.parse_array_last()
        } else {
            self.parse_array_index()
        }
    }

    #[inline]
    fn parse_array_last(&mut self) -> PathParseResult<SingleIndex> {
        self.advance(LAST.len());

        self.eat_whitespaces();
        match self.peek() {
            Some(MINUS) => {
                self.advance(CTRL_CHAR_LEN);
                self.eat_whitespaces();
                match self.peek() {
                    Some(char) if char.is_ascii_digit() => Ok(SingleIndex::Last(self.parse_index()?)),
                    _ => Err(PathParseError::new(PathParseErrorKind::ArrayStepSyntaxError, self.pos)),
                }
            }
            None => Err(PathParseError::new(PathParseErrorKind::MissingSquareBracket, self.pos)),
            _ => Ok(SingleIndex::Last(0)),
        }
    }

    #[inline]
    fn parse_array_index(&mut self) -> PathParseResult<SingleIndex> {
        match self.peek() {
            Some(char) if char.is_ascii_digit() => {
                let index = self.parse_index()?;
                Ok(SingleIndex::Index(index))
            }
            None => Err(PathParseError::new(PathParseErrorKind::MissingSquareBracket, self.pos)),
            _ => Err(PathParseError::new(PathParseErrorKind::ArrayStepSyntaxError, self.pos)),
        }
    }

    #[inline]
    fn has_keyword(&self, keyword: &[u8]) -> bool {
        let len = keyword.len();
        match self.remain() {
            Some(bytes) => {
                if bytes.len() < len {
                    false
                } else {
                    &bytes[0..len] == keyword
                }
            }
            None => false,
        }
    }

    #[inline]
    fn parse_index(&mut self) -> PathParseResult<usize> {
        let begin = self.pos;
        self.skip(|i| i.is_ascii_digit());
        let digits = &self.input[begin..self.pos];

        let mut res = 0usize;
        for &i in digits {
            res = res * 10 + (i - b'0') as usize;
            if res > i32::MAX as usize {
                return Err(PathParseError::new(PathParseErrorKind::ArrayIndexTooLong, self.pos));
            }
        }

        Ok(res)
    }

    #[inline]
    fn parse_object_step(&mut self) -> PathParseResult<()> {
        self.eat_whitespaces();
        match self.peek() {
            None => Err(PathParseError::new(PathParseErrorKind::InvalidKeyStep, self.pos)),
            Some(WILDCARD) => {
                self.advance(CTRL_CHAR_LEN);
                self.push_step(Step::Object(ObjectStep::Wildcard))
            }
            Some(DOUBLE_QUOTE) => self.parse_quoted_field_name::<false>(),
            _ => self.parse_unquoted_field_name::<false>(),
        }
    }

    #[inline]
    fn parse_quoted_field_name<const DESCENDENT: bool>(&mut self) -> PathParseResult<()> {
        debug_assert!(self.peek() == Some(DOUBLE_QUOTE));
        self.advance(CTRL_CHAR_LEN);
        let begin = self.pos;
        let mut end = None;
        while let Some(c) = self.pop() {
            match c {
                b'\\' => match self.pop() {
                    Some(b'b') | Some(b'f') | Some(b'n') | Some(b'r') | Some(b't') | Some(b'"') | Some(b'/')
                    | Some(b'\\') => {
                        //Skip the next character after a backslash.
                    }
                    _ => {
                        return Err(PathParseError::new(PathParseErrorKind::InvalidEscapeSequence, self.pos));
                    }
                },
                b'"' => {
                    // An unescaped double quote marks the end of the quoted string.
                    end = Some(self.pos - 1);
                    break;
                }
                _ => {}
            }
        }

        if let Some(end) = end {
            let key = self.create_key::<true>(begin, end)?;
            if DESCENDENT {
                self.push_step(Step::Descendent(key))
            } else {
                self.push_step(Step::Object(ObjectStep::Key(key)))
            }
        } else {
            Err(PathParseError::new(PathParseErrorKind::UnclosedQuotedStep, self.pos))
        }
    }

    #[inline]
    fn parse_unquoted_field_name<const DESCENDENT: bool>(&mut self) -> PathParseResult<()> {
        self.eat_whitespaces();
        match self.peek() {
            Some(char) if char.is_ascii_alphabetic() => {
                let begin = self.pos;
                self.skip(|i| i.is_ascii_alphabetic() || i.is_ascii_digit());
                let end = self.pos;

                if DESCENDENT {
                    let key = self.create_key::<false>(begin, end)?;
                    self.push_step(Step::Descendent(key))
                } else {
                    self.eat_whitespaces();
                    match self.peek() {
                        Some(LEFT_BRACKET) => {
                            let field_name = &self.input[begin..end];
                            self.parse_item_method(field_name)
                        }
                        Some(DOT) | Some(BEGIN_ARRAY) | None => {
                            let key = self.create_key::<false>(begin, end)?;
                            self.push_step(Step::Object(ObjectStep::Key(key)))
                        }
                        _ => Err(PathParseError::new(
                            PathParseErrorKind::UnexpectedCharacterAtEnd,
                            self.pos,
                        )),
                    }
                }
            }
            _ => Err(PathParseError::new(PathParseErrorKind::InvalidKeyStep, self.pos)),
        }
    }

    #[inline]
    fn parse_item_method(&mut self, field_name: &[u8]) -> PathParseResult<()> {
        debug_assert!(self.peek() == Some(LEFT_BRACKET));
        self.advance(CTRL_CHAR_LEN);
        self.eat_whitespaces();

        if self.peek() == Some(RIGHT_BRACKET) {
            self.advance(CTRL_CHAR_LEN);
            self.eat_whitespaces();

            if !self.exhausted() {
                return Err(PathParseError::new(
                    PathParseErrorKind::UnexpectedCharacterAtEnd,
                    self.pos,
                ));
            }

            match field_name {
                COUNT => self.push_step(Step::Func(FuncStep::Count)),
                SIZE => self.push_step(Step::Func(FuncStep::Size)),
                TYPE => self.push_step(Step::Func(FuncStep::Type)),
                _ => Err(PathParseError::new(PathParseErrorKind::InvalidFunction, self.pos)),
            }
        } else {
            Err(PathParseError::new(PathParseErrorKind::InvalidFunction, self.pos))
        }
    }

    #[inline]
    fn parse_descendent_step(&mut self) -> PathParseResult<()> {
        debug_assert!(self.peek() == Some(DOT));
        self.advance(CTRL_CHAR_LEN);
        self.eat_whitespaces();
        match self.peek() {
            Some(DOUBLE_QUOTE) => self.parse_quoted_field_name::<true>(),
            None => Err(PathParseError::new(PathParseErrorKind::InvalidKeyStep, self.pos)),
            _ => self.parse_unquoted_field_name::<true>(),
        }
    }

    #[inline]
    fn remain(&self) -> Option<&[u8]> {
        if self.pos < self.input.len() {
            Some(&self.input[self.pos..])
        } else {
            None
        }
    }

    #[inline]
    fn eat_whitespaces(&mut self) {
        let count = self
            .remain()
            .map_or(0, |rem| rem.iter().take_while(|&i| i.is_ascii_whitespace()).count());
        self.advance(count);
    }

    #[inline]
    fn exhausted(&self) -> bool {
        self.pos >= self.input.len()
    }

    #[inline]
    fn pop(&mut self) -> Option<u8> {
        if self.exhausted() {
            return None;
        }
        let val = self.input[self.pos];
        self.pos += 1;
        Some(val)
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    #[inline]
    fn advance(&mut self, step: usize) {
        self.pos += step;
    }

    #[inline]
    fn skip<F: Fn(u8) -> bool>(&mut self, f: F) {
        let count = self.remain().map_or(0, |rem| rem.iter().take_while(|i| f(**i)).count());
        self.advance(count);
    }

    #[inline]
    fn push_step(&mut self, step: Step) -> PathParseResult<()> {
        self.path
            .try_reserve(std::mem::size_of::<Step>())
            .map_err(|e| PathParseError::new(PathParseErrorKind::TryReserveError(e), self.pos))?;
        self.path.push(step);
        Ok(())
    }

    #[inline]
    fn create_key<const CHECK_UTF8: bool>(&self, begin: usize, end: usize) -> PathParseResult<String> {
        let bytes = &self.input[begin..end];
        let mut key = String::new();
        key.try_reserve(bytes.len())
            .map_err(|e| PathParseError::new(PathParseErrorKind::TryReserveError(e), self.pos))?;
        let str = if CHECK_UTF8 {
            std::str::from_utf8(bytes).map_err(|_| PathParseError::new(PathParseErrorKind::InvalidKeyStep, self.pos))?
        } else {
            // SAFETY: bytes must only contains [0..9], [a..z] and [A..Z] when CHECK_UTF8 is false.
            unsafe { std::str::from_utf8_unchecked(bytes) }
        };
        key.push_str(str);
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert(input: &str, expected: Option<&[Step]>, error: Option<(PathParseErrorKind, usize)>) {
        let path = str::parse::<PathExpression>(input);
        match error {
            Some((kind, pos)) => {
                assert!(path.is_err());
                let err = path.err().unwrap();
                assert_eq!(err.kind, kind);
                assert_eq!(err.pos, pos);
            }
            None => {
                assert!(path.is_ok());
                let path = path.unwrap();
                let fields = path.steps();
                assert_eq!(fields, expected.unwrap());
            }
        }
    }

    fn assert_path_parse_error(input: &str, kind: PathParseErrorKind, pos: usize) {
        assert(input, None, Some((kind, pos)))
    }

    fn assert_path_parse(input: &str, expected: &[Step]) {
        assert(input, Some(expected), None)
    }

    #[test]
    fn test_path_parse() {
        let input = "$";
        let expected = vec![Step::Root];
        assert_path_parse(input, &expected);

        let input = "$  ";
        let expected = vec![Step::Root];
        assert_path_parse(input, &expected);

        let input = "  $";
        let expected = vec![Step::Root];
        assert_path_parse(input, &expected);

        let input = "$.key";
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = "$.  key";
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = "$.key  ";
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = "$.  key  ";
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = "    $.key";
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = r#"$."key""#;
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = r#"$.  "key""#;
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = r#"$."key"  "#;
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = r#"$.  "key"  "#;
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("key".to_string()))];
        assert_path_parse(input, &expected);

        let input = r#"$."测试""#;
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key("测试".to_string()))];
        assert_path_parse(input, &expected);

        let input = r#"$."测\t试""#;
        let expected = vec![Step::Root, Step::Object(ObjectStep::Key(r#"测\t试"#.to_string()))];
        assert_path_parse(input, &expected);

        let input = "$..key";
        let expected = vec![Step::Root, Step::Descendent("key".to_string())];
        assert_path_parse(input, &expected);

        let input = "$.*";
        let expected = vec![Step::Root, Step::Object(ObjectStep::Wildcard)];
        assert_path_parse(input, &expected);

        let input = "$[1]";
        let expected = vec![Step::Root, Step::Array(ArrayStep::Index(1))];
        assert_path_parse(input, &expected);

        let input = "$[last]";
        let expected = vec![Step::Root, Step::Array(ArrayStep::Last(0))];
        assert_path_parse(input, &expected);

        let input = "$[last - 4]";
        let expected = vec![Step::Root, Step::Array(ArrayStep::Last(4))];
        assert_path_parse(input, &expected);

        let input = "$[1 to 5]";
        let expected = vec![
            Step::Root,
            Step::Array(ArrayStep::Range(SingleIndex::Index(1), SingleIndex::Index(5))),
        ];
        assert_path_parse(input, &expected);

        let input = "$[1 to last]";
        let expected = vec![
            Step::Root,
            Step::Array(ArrayStep::Range(SingleIndex::Index(1), SingleIndex::Last(0))),
        ];
        assert_path_parse(input, &expected);

        let input = "$[1 to last - 4]";
        let expected = vec![
            Step::Root,
            Step::Array(ArrayStep::Range(SingleIndex::Index(1), SingleIndex::Last(4))),
        ];
        assert_path_parse(input, &expected);

        let input = "$[1, last, last - 2, 3 to 10, last - 4 to 2]";
        let expected = vec![
            Step::Root,
            Step::Array(ArrayStep::Multiple(vec![
                SingleStep::Single(SingleIndex::Index(1)),
                SingleStep::Single(SingleIndex::Last(0)),
                SingleStep::Single(SingleIndex::Last(2)),
                SingleStep::Range(SingleIndex::Index(3), SingleIndex::Index(10)),
                SingleStep::Range(SingleIndex::Last(4), SingleIndex::Index(2)),
            ])),
        ];
        assert_path_parse(input, &expected);

        let input = "$[*]";
        let expected = vec![Step::Root, Step::Array(ArrayStep::Wildcard)];
        assert_path_parse(input, &expected);

        let input = "$.size()";
        let expected = vec![Step::Root, Step::Func(FuncStep::Size)];
        assert_path_parse(input, &expected);

        let input = "$.type()";
        let expected = vec![Step::Root, Step::Func(FuncStep::Type)];
        assert_path_parse(input, &expected);

        let input = "$.count()";
        let expected = vec![Step::Root, Step::Func(FuncStep::Count)];
        assert_path_parse(input, &expected);

        let input = "$.key[1]";
        let expected = vec![
            Step::Root,
            Step::Object(ObjectStep::Key("key".to_string())),
            Step::Array(ArrayStep::Index(1)),
        ];
        assert_path_parse(input, &expected);

        let input = r#"$."key"[last]"#;
        let expected = vec![
            Step::Root,
            Step::Object(ObjectStep::Key("key".to_string())),
            Step::Array(ArrayStep::Last(0)),
        ];
        assert_path_parse(input, &expected);

        let input = r#"$."key"[*]"#;
        let expected = vec![
            Step::Root,
            Step::Object(ObjectStep::Key("key".to_string())),
            Step::Array(ArrayStep::Wildcard),
        ];
        assert_path_parse(input, &expected);

        let input = r#"$."key"[*].type()"#;
        let expected = vec![
            Step::Root,
            Step::Object(ObjectStep::Key("key".to_string())),
            Step::Array(ArrayStep::Wildcard),
            Step::Func(FuncStep::Type),
        ];
        assert_path_parse(input, &expected);

        let input = r#"$.key..name[*].type()"#;
        let expected = vec![
            Step::Root,
            Step::Object(ObjectStep::Key("key".to_string())),
            Step::Descendent("name".to_string()),
            Step::Array(ArrayStep::Wildcard),
            Step::Func(FuncStep::Type),
        ];
        assert_path_parse(input, &expected);

        let input = r#"$."key"[*]..name.size()"#;
        let expected = vec![
            Step::Root,
            Step::Object(ObjectStep::Key("key".to_string())),
            Step::Array(ArrayStep::Wildcard),
            Step::Descendent("name".to_string()),
            Step::Func(FuncStep::Size),
        ];
        assert_path_parse(input, &expected);
    }

    #[test]
    fn test_path_parse_error() {
        let input = "@.key";
        assert_path_parse_error(input, PathParseErrorKind::NotStartWithDollar, 1);

        let input = "\t$.key";
        assert_path_parse_error(input, PathParseErrorKind::NotStartWithDollar, 1);

        let input = "$.key123&.key2";
        assert_path_parse_error(input, PathParseErrorKind::UnexpectedCharacterAtEnd, 8);

        let input = "$.key[123";
        assert_path_parse_error(input, PathParseErrorKind::MissingSquareBracket, 9);

        let input = "$.key[abc]";
        assert_path_parse_error(input, PathParseErrorKind::ArrayStepSyntaxError, 6);

        let input = "$.key[]";
        assert_path_parse_error(input, PathParseErrorKind::EmptyArrayStep, 6);

        let input = "$.";
        assert_path_parse_error(input, PathParseErrorKind::InvalidKeyStep, 2);

        let input = "$..";
        assert_path_parse_error(input, PathParseErrorKind::InvalidKeyStep, 3);

        let input = "$.key[";
        assert_path_parse_error(input, PathParseErrorKind::MissingSquareBracket, 6);

        let input = "$.key[12312313131321321231]";
        assert_path_parse_error(input, PathParseErrorKind::ArrayIndexTooLong, 26);

        let input = r#"$."nam\ae""#;
        assert_path_parse_error(input, PathParseErrorKind::InvalidEscapeSequence, 8);

        let input = r#"$."nam"#;
        assert_path_parse_error(input, PathParseErrorKind::UnclosedQuotedStep, 6);

        let input = "$.1key";
        assert_path_parse_error(input, PathParseErrorKind::InvalidKeyStep, 2);

        let input = "$.abs()";
        assert_path_parse_error(input, PathParseErrorKind::InvalidFunction, 7);

        let input = "$.size().key";
        assert_path_parse_error(input, PathParseErrorKind::UnexpectedCharacterAtEnd, 8);

        let input = "$a.size()";
        assert_path_parse_error(input, PathParseErrorKind::InvalidCharacterAtStepStart, 2);

        let input = "$\t.size()";
        assert_path_parse_error(input, PathParseErrorKind::InvalidCharacterAtStepStart, 2);
    }
}
