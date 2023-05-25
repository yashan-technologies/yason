use std::borrow::Cow;
use std::collections::BTreeMap;

use super::error::{ErrorCode, JsonParseError, Result};

use super::{
    tokenizer::{Token, Tokenizer},
    value::Value,
};

pub(crate) struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    #[inline]
    pub const fn new(s: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::<'_>::new(s),
        }
    }

    #[inline]
    fn step(&mut self) -> Result<Token<'a>> {
        let token = self.tokenizer.next();

        if let Some(t) = token {
            t
        } else {
            Err(self.error(ErrorCode::EofWhileParsingValue))
        }
    }

    #[inline]
    pub fn parse_str(&mut self) -> Result<Value<'a>> {
        let value = self.parse()?;

        self.end()?;

        Ok(value)
    }

    #[inline]
    fn parse(&mut self) -> Result<Value<'a>> {
        let token = self.step()?;

        self.parse_from(token)
    }

    #[inline]
    fn parse_array(&mut self) -> Result<Value<'a>> {
        let mut array = Vec::new();

        // deal with [] or [v
        match self.step()? {
            Token::BracketOff => {
                return Ok(Value::Array(array));
            }
            token => {
                array.try_reserve(1)?;
                array.push(self.parse_from(token)?);
            }
        }

        // deal with ,v,v,v]
        loop {
            match self.step()? {
                Token::BracketOff => {
                    break;
                }
                Token::Comma => {
                    array.try_reserve(1)?;
                    array.push(self.parse()?);
                }
                _ => {
                    return Err(self.error(ErrorCode::ExpectedListCommaOrEnd));
                }
            }
        }

        Ok(Value::Array(array))
    }

    #[inline]
    fn parse_object(&mut self) -> Result<Value<'a>> {
        let mut object = BTreeMap::new();

        // deal with {} or {"key":value
        match self.step()? {
            Token::BraceOff => {
                return Ok(Value::Object(object));
            }
            Token::String(s) => {
                self.parse_object_pair(&mut object, Some(s))?;
            }
            _ => {
                return Err(self.error(ErrorCode::KeyMustBeAString));
            }
        }

        // dealt with ,"key":value,"key":value}
        loop {
            match self.step()? {
                Token::BraceOff => {
                    break;
                }
                Token::Comma => {
                    self.parse_object_pair(&mut object, None)?;
                }
                _ => {
                    return Err(self.error(ErrorCode::ExpectedObjectCommaOrEnd));
                }
            }
        }

        Ok(Value::Object(object))
    }

    #[inline]
    fn parse_object_pair(
        &mut self,
        object: &mut BTreeMap<Cow<'a, str>, Value<'a>>,
        first_key: Option<Cow<'a, str>>,
    ) -> Result<()> {
        // k
        let k: Cow<str>;
        if let Some(s) = first_key {
            k = s;
        } else if let Token::String(s) = self.step()? {
            k = s;
        } else {
            return Err(self.error(ErrorCode::KeyMustBeAString));
        }

        // :
        if Token::Colon != self.step()? {
            return Err(self.error(ErrorCode::ExpectedColon));
        }

        // v
        let token = self.step()?;
        let v = self.parse_from(token)?;

        // TODO: OOM panic
        object.insert(k, v);

        Ok(())
    }

    #[inline]
    fn parse_from(&mut self, token: Token<'a>) -> Result<Value<'a>> {
        match token {
            Token::Null => Ok(Value::Null),
            Token::String(s) => Ok(Value::String(s)),
            Token::Number(n) => Ok(Value::Number(n)),
            Token::Boolean(b) => Ok(Value::Bool(b)),
            Token::BracketOn => self.parse_array(),
            Token::BraceOn => self.parse_object(),
            _ => Err(self.error(ErrorCode::ExpectedSomeValue)),
        }
    }

    #[inline]
    fn end(&mut self) -> Result<()> {
        self.tokenizer.end()
    }

    #[cold]
    fn error(&self, reason: ErrorCode) -> JsonParseError {
        self.tokenizer.error(reason)
    }
}
