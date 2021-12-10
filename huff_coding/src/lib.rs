//! An implementation of the [Huffman coding algorithm][huff_wiki], enabling
//! one to create a Huffman tree with any alphabet they choose.
//! 
//! It mainly revolves around the [`HuffTree`][tree] struct, which provides a way to
//! generate Huffman [prefix codes][huff_wiki_codes] for any collection of types implementing
//! the [`HuffLetter`][letter] trait, where for every letter there is a corresponding weight
//! (To ensure this, the [`Weights`][weights] trait must be implemented on the provided collection).
//! If the provided letters also implement the [`HuffLetterAsBytes`][letter_bytes] trait, 
//! the tree can be easily read or returned in binary form.
//! 
//! # Examples
//! 
//! ```
//! use huff_coding::{
//!     prelude::*,
//!     bitvec::prelude::*,
//! };
//! 
//! // every primitive type (except floats) implements HuffLetter
//! let bytes = [0xff, 0xff, 0xff, 0xaa, 0xaa, 0xcc];
//! let chars = ['a', 'a', 'a', 'b', 'b', 'c'];
//! let ints = [-32, 123, -32, -32, 75, 123];
//! 
//! // ------ building weights structs ------
//! // building weights with the ByteWeights struct 
//! let byte_weights = ByteWeights::from_bytes(&bytes);
//! // building weights in the form of a HashMap
//! let char_weights = build_weights_map(&chars);
//! let int_weights = build_weights_map(&ints);
//! 
//! // ------ initializing HuffTrees ------
//! let tree_bytes = HuffTree::from_weights(byte_weights);
//! let tree_chars = HuffTree::from_weights(char_weights);
//! let tree_ints = HuffTree::from_weights(int_weights);
//! 
//! // ------ reading codes from a tree ------
//! let char_codes = tree_chars.read_codes();
//! 
//! assert_eq!(
//!     char_codes.get(&'a').unwrap(),
//!     &bitvec![Msb0, u8; 0]
//! );
//! assert_eq!(
//!     char_codes.get(&'b').unwrap(),
//!     &bitvec![Msb0, u8; 1, 1]
//! );
//! assert_eq!(
//!     char_codes.get(&'c').unwrap(),
//!     &bitvec![Msb0, u8; 1, 0]
//! );
//! 
//! // ------ HuffTree in binary ------
//! // every integer implements HuffLetterAsBytes
//! let tree_bytes_bin = tree_bytes.as_bin(); 
//! assert_eq!(tree_bytes_bin.to_string(), "[10111111, 11101100, 11000101, 01010]");
//! 
//! // reading a HuffTree from a binary representation
//! let tree_bytes_from_bin = HuffTree::<u8>::try_from_bin(tree_bytes_bin).unwrap();
//! assert_eq!(tree_bytes.read_codes(), tree_bytes_from_bin.read_codes());
//! ```  
//! 
//! Included are also example [compression][compress]/[decompression][decompress] functions using my implementation
//! of this algorithm.
//! ```
//! use huff_coding::prelude::*;
//! 
//! let bytes = b"abbccc";
//! 
//! let comp_data = compress(bytes);
//! let decomp_bytes = decompress(&comp_data);
//! 
//! assert_eq!(bytes.to_vec(), decomp_bytes);
//! ```
//! 
//! Every binary representation in the crate is made thanks to the [`bitvec`][bitvec] crate which
//! I've re-exported for convenience.
//! 
//! [tree]:tree::HuffTree
//! [letter]:tree::letter::HuffLetter
//! [letter_bytes]:tree::letter::HuffLetterAsBytes
//! [weights]:weights::Weights
//! [compress]:crate::comp::compress
//! [decompress]:crate::comp::decompress
//! [huff_wiki]:https://en.wikipedia.org/wiki/Huffman_coding
//! [huff_wiki_codes]:https://en.wikipedia.org/wiki/Prefix_code

// when i have time:
// TODO: add some abstraction over returned BitVecs (Deref 'n stuff)
// TODO: look at serde

/// Structs and traits used to represent and construct Huffman trees.
pub mod tree;
/// Trait signifying that the struct stores the weights of a certain type (letter), so that
/// for any stored letter there is a corresponding `usize`(weight).
pub mod weights;
/// Example compression/decompression functions using the [`HuffTree`][crate::tree::HuffTree] struct.
pub mod comp;
/// `huff_coding` prelude.
///
/// This collects the general public API into a single spot for inclusion, as
/// `use huff_coding::prelude::*;`, without polluting the root namespace of the crate.
pub mod prelude;

mod utils;


// `bitvec` re-export
pub use bitvec;
