//! Encoding and decoding support for YASON in Rust.
//!
//! ## Optional features
//!
//! ### `serde`
//!
//! When this optional dependency is enabled, `YasonBuf` implements the `serde::Serialize` and
//! `serde::Deserialize` traits.
//!
//! ## Yason binary format
//!
//! ```BNF
//! yason ::= type value
//!
//! type ::=
//!     object-type |
//!     array-type |
//!     scalar-type |
//!
//! object-type ::= 1
//! array-type ::= 2
//! scalar-type ::=
//!     3 |     // string
//!     4 |     // number
//!     5 |     // bool
//!     6 |     // null
//!     7 |     // 8-bit signed integer
//!     8 |     // 16-bit signed integer
//!     9 |     // 32-bit signed integer
//!     10 |    // 64-bit signed integer
//!     11 |    // 8-bit unsigned integer
//!     12 |    // 16-bit unsigned integer
//!     13 |    // 32-bit unsigned integer
//!     14 |    // 64-bit unsigned integer
//!     15 |    // 32-bit floating point
//!     16 |    // 64-bit floating point
//!     17 |    // binary data
//!     18 |    // timestamp
//!     19 |    // date
//!     20 |    // short date
//!     21 |    // time
//!     22 |    // interval year-month
//!     23 |    // interval day-time
//!
//! value ::=
//!     object |
//!     array |
//!     scalar |
//!
//! scalar ::=
//!     string |            // string
//!     number |            // number
//!     bool |              // bool
//!     int8 |              // 8-bit signed integer
//!     int16 |             // 16-bit signed integer
//!     int32 |             // 32-bit signed integer
//!     int64 |             // 64-bit signed integer
//!     uint8 |             // 8-bit unsigned integer
//!     uint16 |            // 16-bit unsigned integer
//!     uint32 |            // 32-bit unsigned integer
//!     uint64 |            // 64-bit unsigned integer
//!     float32 |           // 32-bit floating point
//!     float64 |           // 64-bit floating point
//!     binary |            // binary data
//!     timestamp |         // timestamp
//!     date |              // date
//!     short-date |        // short date
//!     time |              // time
//!     interval-ym |       // interval year-month
//!     interval-dt         // interval day-time
//!
//! string ::= data-length uint8*
//! number ::= uint8 uint8* // first uint8 indicates size of number
//! bool ::= 0 | 1
//! binary ::= data-length uint8*
//! timestamp ::= int64
//! date ::= int64
//! short-date ::= int32
//! time ::= int64
//! interval-ym ::= int32
//! interval-dt ::= int64
//!
//! data-length ::= uint8*  // If the high bit of a byte is 1,
//!                         // the length field is continued in the next byte,
//!                         // otherwise it is the last byte of the length field.
//!                         // So we need 1 byte to represent lengths up to 127,
//!                         // 2 bytes to represent lengths up to 16383, and so on...
//!                         // Use 4 bytes at most.
//!
//! // key-offset is ordered by key length and lexicographical order
//! object ::= size element-count key-offset* key-value*
//!
//! array ::= size element-count value-entry* outlined-value*
//!
//! size ::= int32  // size indicates total size of object or array
//! element-count ::= uint16 // number of members in object or array
//!
//! key-offset ::= uint32
//! key-value ::= key type value
//! key ::= key-length uint8*
//! key-length ::= uint16    // key length must be less than 64KB
//!
//! value-entry ::= type offset-or-inlined-value
//!
//! // This field holds either the offset to where the value is stored,
//! // or the value itself if it is small enough to be inlined (that is 4 bytes).
//! offset-or-inlined-value ::= uint32
//!
//! outlined-value ::= type value
//! ```
//!
//! ## Usage
//!
//! ### `Scalar`
//!
//! To encode a `Scalar`, use [`Scalar`]:
//!
//! ```rust
//! use yason::Scalar;
//!
//! let yason = Scalar::string("string").unwrap();
//!
//! let mut bytes = Vec::with_capacity(16);
//! let yason = Scalar::string_with_vec("string", &mut bytes).unwrap();
//! ```
//!
//! ### `Object` / `Array`
//!
//! To encode an `Object`, use [`ObjectBuilder`] or [`ObjectRefBuilder`]:
//!
//! ```rust
//! use yason::{DataType, ObjectBuilder, ObjectRefBuilder};
//!
//! let mut builder = ObjectBuilder::try_new(2, false).unwrap();
//! builder.push_string("key1", "value").unwrap();
//! builder.push_bool("key2", true);
//! let yason = builder.finish().unwrap();
//! assert_eq!(yason.data_type().unwrap(), DataType::Object);
//!
//! let mut bytes = Vec::with_capacity(16);
//! let mut builder = ObjectRefBuilder::try_new(&mut bytes, 1, false).unwrap();
//! builder.push_string("key", "value").unwrap();
//! let yason = builder.finish().unwrap();
//! assert_eq!(yason.data_type().unwrap(), DataType::Object);
//! ```
//! To encode an `Array`, use [`ArrayBuilder`] or [`ArrayRefBuilder`]:
//!
//! ```rust
//! use yason::{DataType, ArrayBuilder, ArrayRefBuilder};
//!
//! let mut builder = ArrayBuilder::try_new(2).unwrap();
//! builder.push_string("string").unwrap();
//! builder.push_bool(true);
//! let yason = builder.finish().unwrap();
//! assert_eq!(yason.data_type().unwrap(), DataType::Array);
//!
//! let mut bytes = Vec::with_capacity(16);
//! let mut builder = ArrayRefBuilder::try_new(&mut bytes, 1).unwrap();
//! builder.push_string("string").unwrap();
//! let yason = builder.finish().unwrap();
//! assert_eq!(yason.data_type().unwrap(), DataType::Array);
//! ```
//!
//! ### Nested `Object` / `Array`
//! To encode an `Object` or `Array` that contains nested `Object` or `Array`:
//!
//! ```rust
//! use yason::{DataType, ObjectBuilder};
//! let mut obj_builder = ObjectBuilder::try_new(1, true).unwrap();
//! let mut array_builder = obj_builder.push_array("key", 1).unwrap();
//! array_builder.push_bool(true).unwrap();
//!
//! array_builder.finish().unwrap();
//! let yason = obj_builder.finish().unwrap();
//! assert_eq!(yason.data_type().unwrap(), DataType::Object);
//! ```
//!

#![cfg_attr(docsrs, feature(doc_cfg))]

mod binary;
mod builder;
mod data_type;
mod format;
mod json;
mod path;
mod util;
mod vec;
mod yason;

#[cfg(feature = "serde")]
mod serde;

pub use self::{
    builder::{ArrayBuilder, ArrayRefBuilder, BuildError, NumberError, ObjectBuilder, ObjectRefBuilder, Scalar},
    data_type::{DataType, InvalidDataType},
    format::FormatError,
    path::{PathExpression, PathParseError, QueriedValue},
    yason::{Array, ArrayIter, KeyIter, Object, ObjectIter, Value, ValueIter, Yason, YasonBuf, YasonError},
};
pub use decimal_rs::Decimal as Number;
