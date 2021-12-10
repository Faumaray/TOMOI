use super::{
    prelude::{
        HuffTree,
        HuffLetter,
        HuffLetterAsBytes,
        build_weights_map,
    },
    utils::calc_padding_bits,
    bitvec::prelude::BitVec,
};
use self::errors::{
    CompressError,
    CompressedDataFromBytesError,
};

use std::{
    convert::TryInto,
    marker::PhantomData,
};



/// Data representing a slice of letters (types implementing [`HuffLetter`][letter]) 
/// compressed into bytes by the [`compress`][compress] or [`compress_with_tree`][compress_with_tree] 
/// function.
/// 
/// It stores:
/// * `L` -> generic type of the compressed letters
/// * [`comp_bytes`](#method.comp_bytes) -> representing the compressed slice
/// * [`padding_bits`](#method.padding_bits) -> the number of bits used for padding in the comp_bytes
/// * [`huff_tree`](#method.huff_tree) -> the [`HuffTree`][tree] used to compress the slice
/// 
/// If the letter type also implements [`HuffLetterAsBytes`][letter_bytes], the compressed
/// data can be easily represented as bytes (see the [`to_bytes`](#method.to_bytes) method's 
/// docs for more information).
/// 
/// [tree]:crate::tree::HuffTree
/// [letter]:crate::tree::letter::HuffLetter
/// [letter_bytes]:crate::tree::letter::HuffLetterAsBytes
#[derive(Debug, Clone)]
pub struct CompressData<L: HuffLetter>{
    comp_bytes: Vec<u8>,
    padding_bits: u8,
    huff_tree: HuffTree<L>,
    _typebind: PhantomData<L>
}

impl<L: HuffLetter> CompressData<L>{
    /// Initialize a new instance of `CompressData` with the provided
    /// compressed bytes, padding bits and [`HuffTree`][crate::tree::HuffTree].
    /// 
    /// # Panics
    /// When providing an empty `comp_bytes` or
    /// when providing `padding_bits` larger than 7.
    pub fn new(comp_bytes: Vec<u8>, padding_bits: u8, huff_tree: HuffTree<L>) -> Self{
        if comp_bytes.is_empty(){
            panic!("provided comp_bytes are empty")
        }
        if padding_bits > 7{
            panic!("padding bits cannot be larger than 7")
        }
        Self{
            comp_bytes,
            padding_bits,
            huff_tree,
            _typebind: PhantomData::default(),
        }
    }

    /// Return a reference to the stored slice compressed into bytes
    pub fn comp_bytes(&self) -> &[u8]{
        &self.comp_bytes
    }

    /// Return the number of bits used for padding in the compressed slice
    pub fn padding_bits(&self) -> u8{
        self.padding_bits
    }

    /// Return a reference to the [`HuffTree`][crate::tree::HuffTree] used to compress the slice
    pub fn huff_tree(&self) -> &HuffTree<L>{
        &self.huff_tree
    }

    /// Consume `self` returning the ownership of `comp_bytes`, `padding_bits` and `huff_tree`
    pub fn into_inner(self) -> (Vec<u8>, u8, HuffTree<L>){
        (self.comp_bytes, self.padding_bits, self.huff_tree)
    }
}

impl<L: HuffLetterAsBytes> CompressData<L>{
    /// Try to construct `CompressData<L>` from the given byte representation.
    /// 
    /// Use [`to_bytes`](#method.to_bytes) to get the byte representation of the `CompressData`.
    /// 
    /// # Example
    /// ---
    /// ```
    /// use huff_coding::prelude::{
    ///     CompressData,
    ///     compress,
    ///     decompress,
    /// };
    /// 
    /// let bytes = b"abbccc";
    /// 
    /// let comp_data = compress(bytes);
    /// 
    /// assert_eq!(
    ///     bytes.to_vec(),
    ///     decompress(
    ///         &CompressData::<u8>::try_from_bytes(
    ///             &comp_data.to_bytes()
    ///         ).unwrap()
    ///     ),
    /// ) 
    /// ```
    /// # Errors
    /// ---
    /// 1. When the provided slice is too short to read padding, tree length, tree
    /// and data
    /// 2. When the stored tree length is lower than 2 (a [`HuffTree`][tree] with padding 
    /// can't be encoded in less than 2 bytes)
    /// 3. When the [`HuffTree`][tree] stored in the bytes is invalid or has a different letter type
    /// than specified
    /// 
    /// [tree]:crate::tree::HuffTree
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, CompressedDataFromBytesError>{
        /// Returns DecompressError with the given message 
        /// if the index is out of bounds of bytes
        macro_rules! bytes_try_get {
            [$index:expr; $message:expr] => {
                if let Some(subslice) = bytes.get($index){
                    Ok(subslice)
                }
                else{
                    Err(CompressedDataFromBytesError::new($message))
                }
            };
        }

        // get padding data
        let padding_bits = bytes_try_get![0; "slice is empty"]?;
        let tree_padding_bits =  padding_bits >> 4;
        let data_padding_bits = padding_bits & 0b0000_1111;

        // read 4 bytes of tree length
        let tree_len = u32::from_be_bytes(
            bytes_try_get![1..5; "slice too short to read tree length"]?
            .try_into()
            .unwrap()
        ) as usize;
        if tree_len < 2{
            panic!("stored tree length must be at least 2");
        } 

        // read the tree
        let tree_from_bin_result = 
            HuffTree::<L>::try_from_bin({
                let mut b = BitVec::from_vec(
                    bytes_try_get![5..5 + tree_len; "slice too short to read tree"]?
                    .to_vec()
                );
                for _ in 0..tree_padding_bits{b.pop();}
                b
            });
        let tree = 
            if let Ok(tree) = tree_from_bin_result{
                tree
            }
            else{
                return Err(
                    CompressedDataFromBytesError::new(
                        "invalid tree in slice"
                    )
                )
            };

        Ok(CompressData::new(
            bytes_try_get![5 + tree_len..; "slice does not contain compressed data"]?.to_vec(), 
            data_padding_bits,
            tree
        ))
    }

    /// Convert the `CompressData` into a byte representation.
    /// 
    /// Use [`try_from_bytes`](#method.try_from_bytes) to convert it back into `CompressData`.
    /// 
    /// # Encoding scheme
    /// ---
    /// The returned bytes store, in order:
    /// 1. A byte containing the number of bits used for padding:
    ///  * first 4 bits store the [HuffTree's][tree] padding bits
    ///  * the remaining bits store the compressed data's padding bits
    /// 2. 4 byte number representing the length (in bytes) of the stored [`HuffTree`][tree]
    /// 3. A [`HuffTree`][tree], used to compress the file, 
    /// represented in binary (see [`HuffTree::try_from_bin`][from_bin])
    /// 4. The actual compressed data
    /// 
    /// # Example
    /// –––
    /// Here's a manual deconstruction of the compressed
    /// data:
    /// ```
    /// use huff_coding::{
    ///     prelude::{
    ///         compress,
    ///         HuffTree,
    ///     },
    ///     bitvec::prelude::*,
    /// };
    /// use std::{
    ///     convert::TryInto,
    ///     collections::HashMap,
    /// };
    /// 
    /// // get compressed data
    /// let compressed_data = compress(b"abbccc");
    /// let compressed_data_bytes = compressed_data.to_bytes();
    /// 
    /// // first byte stores the padding bits, 
    /// // in this case:
    /// // * 3 padding bits used for the tree
    /// // * 7 padding bits used for the data
    /// assert_eq!(
    ///     compressed_data_bytes[0], 
    ///     0x37
    /// );
    /// 
    /// // the next 4 bytes store the tree's length, 
    /// // in this case: 4
    /// assert_eq!(
    ///     u32::from_be_bytes(compressed_data_bytes[1..5].try_into().unwrap()), 
    ///     4
    /// );
    /// 
    /// // next 4 bytes (as read from the length) store the tree,
    /// // in this case it stores the following codes:
    /// // b'a' - 10
    /// // b'b' - 11
    /// // b'c' - 0
    /// let codes = 
    ///     HuffTree::<u8>::try_from_bin({
    ///         // get the next 4 bytes from compressed_data_bytes and remove the specified 3 padding bits
    ///         let mut b = BitVec::from_vec(compressed_data_bytes[5..9].to_vec());
    ///         b.drain(29..);
    ///         b
    ///     })
    ///     .unwrap()
    ///     .read_codes();
    /// 
    /// let mut cmp_codes = HashMap::new();
    /// cmp_codes.insert(b'a', bitvec![Msb0, u8; 1, 0]);
    /// cmp_codes.insert(b'b', bitvec![Msb0, u8; 1, 1]);
    /// cmp_codes.insert(b'c', bitvec![Msb0, u8; 0]);
    /// 
    /// assert_eq!(codes, cmp_codes);
    /// 
    /// // the last bytes (containing the compressed data) are:
    /// assert_eq!(compressed_data_bytes[9], 0b10111100);
    /// assert_eq!(compressed_data_bytes[10], 0b00000000);
    /// ```
    /// now we could easily read the actual data:
    /// 1. 10111100:
    ///  * 10 -> `b'a'`
    ///  * 11 -> `b'b'`
    ///  * 11 -> `b'b'`
    ///  * 0  -> `b'c'`
    ///  * 0  -> `b'c'`
    /// 2. 00000000:
    ///  * 0  -> `b'c'`
    ///  * the remaining 7 bits are used for padding.
    /// 
    /// And thus we succesfully read the bytes `b"abbccc"`!
    /// 
    /// [tree]:crate::tree::HuffTree
    /// [from_bin]:../tree/struct.HuffTree.html#method.try_from_bin
    pub fn to_bytes(&self) -> Vec<u8>{
        // get tree in binary, 
        // calculate its padding bits when converted to bytes
        // calculate its lenght in bytes
        let tree_bin = self.huff_tree().as_bin();
        let tree_bin_padding_bits = calc_padding_bits(tree_bin.len());
        let tree_bytes_len = (tree_bin.len() as u32 + tree_bin_padding_bits as u32) / 8;

        let mut bytes = Vec::new();
        // push an empty byte, later to be filled by the padding bit nums
        bytes.push((tree_bin_padding_bits << 4) + self.padding_bits());
        // push the length  of the tree_bin (4 byte num)
        bytes.extend(
            tree_bytes_len.to_be_bytes().iter()
        );
        // next push the tree in binary
        bytes.append(&mut tree_bin.into_vec());
        
        bytes.extend(self.comp_bytes());
 
        bytes
    }
}


/// Compress the provided slice of letters (types implementing [`HuffLetter`][letter]), using binary
/// codes generated with a [`HuffTree`][tree] struct, into a byte slice (returned with additional data
/// in the form of [`CompressData`][CompressData]).
/// 
/// The letters are counted into a [`Weights`][weights] collection to create a [`HuffTree`][tree] using
/// the [`build_weights_map`][weights_map] function, which can be optimized a lot for certain letter types.
/// Because of this fact, it's generally faster to use the [`compress_with_tree`][compress_with_tree] function,
/// providing a [`HuffTree`][tree] built with our own [`Weights`][weights] collection (an example of such collection
/// is implemented in the crate on `u8` in the form of [`ByteWeights`][byte_weights]).
/// 
/// The returned [`CompressData`][CompressData] can be decompressed into the original letter slice with
/// the [`decompress`][decompress] function.
/// 
/// # How it works
/// ---
/// It just reads every letter's code in the created [`HuffTree`][tree] and inserts them into
/// a [`Vec<u8>`][Vec]. The codes themselves are mostly not a multiple of 8 bits long, so some
/// of them can be used as padding in the last byte. The padding information, as well as
/// the tree used to compress the slice are included in the returned [`CompressData`][CompressData].
/// 
/// # Example
/// ---
/// ```
/// use huff_coding::prelude::{
///     compress,
///     decompress
/// };
/// 
/// let bytes = b"abbccc";
/// let nums = &[-97, -98, -98, -99, -99, -99];
/// let chars = &['a', 'b', 'b', 'c', 'c', 'c'];
/// let strs = &["ay", "bee", "bee", "cee", "cee", "cee"];
/// 
/// let comp_bytes = compress(bytes);
/// let comp_nums = compress(nums);
/// let comp_chars = compress(chars);
/// let comp_strs = compress(strs);
/// 
/// assert_eq!(bytes.to_vec(), decompress(&comp_bytes));
/// assert_eq!(nums.to_vec(), decompress(&comp_nums));
/// assert_eq!(chars.to_vec(), decompress(&comp_chars));
/// assert_eq!(strs.to_vec(), decompress(&comp_strs));
/// ```
/// 
/// [tree]:crate::tree::HuffTree
/// [letter]:crate::tree::letter::HuffLetter
/// [weights]:crate::weights::Weights
/// [weights_map]:crate::weights::build_weights_map
/// [byte_weights]:crate::weights::ByteWeights
pub fn compress<L: HuffLetter>(letters: &[L]) -> CompressData<L>{
    let huff_tree = HuffTree::from_weights(build_weights_map(letters));
    compress_with_tree(letters, huff_tree).unwrap()
}

/// Compress the provided slice of letters (types implementing [`HuffLetter`][letter]), using binary
/// codes generated with the provided [`HuffTree`][tree] struct, into a byte slice (returned with additional 
/// data in the form of [`CompressData`][CompressData]).
/// 
/// The returned [`CompressData`][CompressData] can be decompressed into the original letter slice with
/// the [`decompress`][decompress] function.
/// 
/// Be wary that the same [`CompressData`][CompressData] built from different [`HuffTree`'s][tree]
/// may not be exactly the same, but will decompress into the same thing 
/// 
/// # How it works
/// ---
/// It just reads every letter's code in the provided [`HuffTree`][tree] and inserts them into
/// a [`Vec<u8>`][Vec]. The codes themselves are mostly not a multiple of 8 bits long, so some
/// of them can be used as padding in the last byte. The padding information, as well as
/// the tree used to compress the slice are included in the returned [`CompressData`][CompressData].
/// 
/// # Example
/// ---
/// ```
/// use huff_coding::prelude::{
///     compress_with_tree,
///     decompress,
///     HuffTree,
///     ByteWeights,
/// };
/// 
/// let bytes = b"abbccc";
/// 
/// let tree = HuffTree::from_weights(
///     ByteWeights::from_bytes(bytes)
/// );
/// let comp_bytes = compress_with_tree(bytes, tree).unwrap();
/// 
/// assert_eq!(bytes.to_vec(), decompress(&comp_bytes));
/// ```
/// 
/// # Errors
/// ---
/// When the provided tree does not contain a code 
/// for a letter in the provided slice:
/// ```should_panic
/// use huff_coding::prelude::{
///     compress_with_tree,
///     HuffTree,
///     ByteWeights,
/// };
/// 
/// let bytes = b"abbccc";
/// let other_bytes = b"abb";
/// 
/// let tree = HuffTree::from_weights(
///     ByteWeights::from_bytes(other_bytes)
/// );
/// 
/// let comp_bytes = compress_with_tree(bytes, tree)
///     .expect("this will panic, letter b'c' not found in codes");
/// ```
/// 
/// [tree]:crate::tree::HuffTree
/// [letter]:crate::tree::letter::HuffLetter
pub fn compress_with_tree<L: HuffLetter>(letters: &[L], huff_tree: HuffTree<L>) -> Result<CompressData<L>, CompressError<L>>{
    let mut comp_letters = Vec::with_capacity(letters.len());
    let codes = huff_tree.read_codes();
    let mut comp_byte = 0b0000_0000;
    let mut bit_ptr = 7;
    for letter in letters{
        // return Err if there's no code
        let code = 
            if let Some(code) = codes.get(letter){Ok(code)}
            else{
                Err(CompressError::new(
                    "letter not found in codes", 
                    letter.clone()))
            }?;
        for bit in code{
            // set bit on current byte
            comp_byte |= (*bit as u8) << bit_ptr;
            // if filled comp_byte
            if bit_ptr == 0{
                comp_letters.push(comp_byte);
                comp_byte = 0b0000_0000;
                bit_ptr = 7;
            }
            else{bit_ptr -= 1};
        }
    }
    // calculate the compressed_letters' padding bits
    let padding_bits = if bit_ptr == 7{0} else{bit_ptr + 1};
    if padding_bits != 0{comp_letters.push(comp_byte);}


    Ok(CompressData::new(comp_letters, padding_bits, huff_tree))
}

/// Decompress the provided [`CompressData<L>`][CompressData] into a [`Vec<L>`][Vec].
/// 
/// # How it works
/// ---
/// 1. Start at the root branch of the tree
/// 2. Go bit by bit through the provided [`CompressData`'s][CompressData] comp_bytes
/// 3. Every time a 0 is found, go to the left branch, and 
/// every 1 means going to the right branch
/// 4. When it finally a letter branch is found, it push the letter into
/// the vec, and return to the root branch.
/// 
/// # Example
/// ---
/// ```
/// use huff_coding::prelude::{
///     compress,
///     decompress
/// };
/// 
/// let bytes = b"deefff";
/// let nums = &[-100, -101, -101, -102, -102, -102];
/// let chars = &['d', 'e', 'e', 'f', 'f', 'f'];
/// let strs = &["dee", "e", "e", "ef", "ef", "ef"];
/// 
/// let comp_bytes = compress(bytes);
/// let comp_nums = compress(nums);
/// let comp_chars = compress(chars);
/// let comp_strs = compress(strs);
/// 
/// assert_eq!(bytes.to_vec(), decompress(&comp_bytes));
/// assert_eq!(nums.to_vec(), decompress(&comp_nums));
/// assert_eq!(chars.to_vec(), decompress(&comp_chars));
/// assert_eq!(strs.to_vec(), decompress(&comp_strs));
/// ```
pub fn decompress<L: HuffLetter>(comp_data: &CompressData<L>) -> Vec<L>{
    let bytes = comp_data.comp_bytes();
    let tree = comp_data.huff_tree();

    let mut decomp_letters = Vec::new();
    let mut current_branch = tree.root();
    macro_rules! read_codes_in_byte {
        ($byte: expr;[$bitrange:expr]) => {
            for bit_ptr in $bitrange{
                if current_branch.has_children(){
                    match ($byte >> (7 - bit_ptr)) & 1 == 1{
                        true =>{
                            current_branch = current_branch.right_child().unwrap();
                        }
                        false =>{
                            current_branch = current_branch.left_child().unwrap();
                        }
                    }
                }
                if !current_branch.has_children(){
                    decomp_letters.push(current_branch.leaf().letter().unwrap().clone());
                    current_branch = tree.root();
                }
            }
        };
    }
    for byte in &bytes[..bytes.len() - 1]{
       read_codes_in_byte!(byte;[0..8]);
    }
    read_codes_in_byte!(bytes[bytes.len() - 1];[0..8 - comp_data.padding_bits()]);

    decomp_letters
}


/// Errors returned in the `comp` module's code.
pub mod errors{
    use super::super::prelude::HuffLetter;

    use std::fmt;



    /// Error encountered while trying to create [`CompressData`][super::CompressData] from bytes.
    #[derive(Debug, Clone)]
    pub struct CompressedDataFromBytesError{
        message: &'static str,
    }

    impl fmt::Display for CompressedDataFromBytesError{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for CompressedDataFromBytesError{}

    impl CompressedDataFromBytesError{
        pub fn new(message: &'static str) -> Self{
            Self{
                message,
            }
        }

        pub fn message(&self) -> &str{
            self.message
        }
    }


    /// Error encountered while compressing, meaning that
    /// a byte hasn't been found in the provided codes.
    /// 
    /// Returned by [`compress_with_tree`][super::compress_with_tree] 
    #[derive(Debug, Clone)]
    pub struct CompressError<L: HuffLetter>{
        message: &'static str,
        missing_letter: L,
    }

    impl<L: HuffLetter> fmt::Display for CompressError<L>{
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} ({:?})", self.message, self.missing_letter)
        }
    }

    impl<L: HuffLetter> std::error::Error for CompressError<L>{}

    impl<L: HuffLetter> CompressError<L>{
        pub fn new(message: &'static str, missing_letter: L) -> Self{
            Self{
                message,
                missing_letter,
            }
        }

        pub fn message(&self) -> &str{
            self.message
        }

        pub fn missing_letter(&self) -> &L{
            &self.missing_letter
        }
    }
}
