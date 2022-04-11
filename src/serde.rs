//! Impl the `serde::Serialize` and `serde::Deserialize` traits.

use crate::YasonBuf;
use std::fmt::Formatter;

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl serde::Serialize for YasonBuf {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut buf = String::new();
        if serializer.is_human_readable() {
            self.format_to(false, &mut buf).map_err(serde::ser::Error::custom)?;
            buf.serialize(serializer)
        } else {
            serializer.serialize_bytes(self.as_bytes())
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> serde::Deserialize<'de> for YasonBuf {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct YasonBufVisitor;

        impl<'de> serde::de::Visitor<'de> for YasonBufVisitor {
            type Value = YasonBuf;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "a yason buf")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<YasonBuf, E>
            where
                E: serde::de::Error,
            {
                YasonBuf::parse(v).map_err(serde::de::Error::custom)
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<YasonBuf, E>
            where
                E: serde::de::Error,
            {
                let mut buf = Vec::new();
                buf.try_reserve(v.len()).map_err(serde::de::Error::custom)?;
                buf.extend_from_slice(v);
                let res = unsafe { YasonBuf::new_unchecked(buf) };
                Ok(res)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(YasonBufVisitor)
        } else {
            deserializer.deserialize_bytes(YasonBufVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let yason_buf = YasonBuf::parse(r#"[123, true, null, "abc"]"#).unwrap();

        let bin = bincode::serialize(&yason_buf).unwrap();
        let bin_yason_buf: YasonBuf = bincode::deserialize(&bin).unwrap();

        assert_eq!(bin_yason_buf, yason_buf);
    }
}
