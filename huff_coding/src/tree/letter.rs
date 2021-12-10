use std::{
    fmt::Debug,
    hash::Hash,
    mem::size_of,
    convert::TryInto,
};

/// Trait specifying that the given type can be stored in a [`HuffTree`][crate::tree::HuffTree], 
/// which means it implements: [`Clone`][Clone] + [`Eq`][Eq] + [`Hash`][std::hash::Hash]
/// 
/// Implemented by default for every [primitive type](https://doc.rust-lang.org/stable/std/primitive), 
/// except floats and including [String][String]
pub trait HuffLetter: Clone + Eq + Hash + Debug{}
/// Trait specifying that the given HuffLetter can be converted
/// into bytes *(returns `Box<[u8]>`)* and
/// can be created from bytes (`&[u8]`),
/// so the [`HuffTree`][crate::tree::HuffTree] can be represented in binary.
/// 
/// Implemented by default for every integer
pub trait HuffLetterAsBytes: HuffLetter{
    fn try_from_be_bytes(bytes: &[u8]) ->  Result<Self, Box<dyn std::error::Error>>;
    fn as_be_bytes(&self) -> Box<[u8]>;
}

/// Implements `HuffLetter` for every provided type (without generics) 
macro_rules! primitive_letter_impl{
    {$($type:ty),+} => {
        $(
        impl HuffLetter for $type{}
        )+
    };
}
primitive_letter_impl!{
    char,
    &str, 
    String
}

/// Implements `HuffLetter` and `HuffLetterAsBytes` with a default implementation
/// for provided primitive integer types
macro_rules! integer_letter_impl{
    {$($type:ty),+} => {
        $(
        primitive_letter_impl!{$type}
        impl HuffLetterAsBytes for $type{
            fn try_from_be_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>>{
                let bytes: [u8; size_of::<$type>()] = bytes.try_into()?;
                Ok(Self::from_be_bytes(bytes))
            }
            fn as_be_bytes(&self) -> Box<[u8]>{
                Box::new(self.to_be_bytes())
            }
        }
        )+
    };
}
integer_letter_impl!{
    u8, u16, u32, u64, usize, u128, 
    i8, i16, i32, i64, isize, i128
}
