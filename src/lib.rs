//! Encoding and decoding support for YASON in Rust.

mod builder;
mod data_type;
mod yason;

pub use self::{
    builder::{BuildError, Builder},
    data_type::{DataType, InvalidDataType},
    yason::{Yason, YasonBuf},
};
pub use decimal_rs::Decimal as Number;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let yason = Builder::string("abc").unwrap();
        assert_eq!(yason.data_type().unwrap(), DataType::String);
    }
}
