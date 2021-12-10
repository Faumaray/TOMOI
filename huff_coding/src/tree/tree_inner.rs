use crate::{
    prelude::*,
    utils::size_of_bits,
    bitvec::prelude::{bitvec, BitVec, Msb0},
};
use super::branch_heap::HuffBranchHeap;

use std::{
    fmt,
    mem,
    collections::{hash_map::RandomState, HashMap},
    hash::BuildHasher,
};



/// Struct representing a Huffman Tree with an alphabet of
/// type implementing [`HuffLetter`][letter]
/// 
/// A `HuffTree` can be initialized in two ways:
/// * from a struct implementing the [`Weights<L>`][weights] trait ([`from_weights`](#method.from_weights)), 
/// where `L` must implement the [`HuffLetter`][letter] trait  
/// * from a binary representation ([`try_from_bin`](#method.try_from_bin)): 
/// [`BitVec<Msb0, u8>`][bitvec::prelude::BitVec], where in order to even get it,
/// `L` must implement the [`HuffLetterAsBytes`][letter_bytes] trait 
/// 
/// Codes stored by the tree can be retrieved using the [`codes`](#method.codes) method
/// 
/// # How it works
/// ---
/// When initialized with the [`HuffTree::from_weights`](#method.from_weights) method it
/// follows the steps of the [Huffman Coding algorithm][huff_wiki] (duh):
/// 1. Creates standalone branches for every letter found in the given weights and
/// pushes them onto a branch heap
/// 2. Finds two branches with the lowest weights
/// 3. Makes them children to a branch with a [`None`][None] letter and
/// the children's summed up weight
/// 4. Removes the two found branches from the heap and adds the newly created
/// branch into it
/// 5. Repeats steps 2 to 4 until there's only one branch left
/// 6. Sets the only branch left as root
/// 7. Recurses into the tree to set every branch's code
///  * Every branch gets its parent's code with its own position in the parent branch (left - 0, right - 1)
/// 
/// Initializing from bits goes as follows:
/// 1. Go through the `HuffTree` encoded in binary ([big endian][end_wiki]) bit by bit
/// 2. Every 1 means a joint branch
/// 3. Every 0 means a letter branch followed by [`size_of::<L> * 8`][mem::size_of] bits representing
/// the stored letter
/// 
/// 
/// # Examples
/// ---
/// Initialization from [`ByteWeights`][byte_weights]
/// ```
/// use huff_coding::{
///     bitvec::prelude::*,
///     prelude::{HuffTree, ByteWeights},
/// };
/// use std::collections::HashMap;
/// 
/// let tree = HuffTree::from_weights(
///     ByteWeights::from_bytes(b"abbccc")
/// );
/// let codes = tree.read_codes();
/// 
/// assert_eq!(
///     codes.get(&b'c').unwrap(), 
///     &bitvec![Msb0, u8; 0]
/// );
/// assert_eq!(
///     codes.get(&b'b').unwrap(),
///     &bitvec![Msb0, u8; 1, 1]
/// );
/// assert_eq!(
///     codes.get(&b'a').unwrap(),
///     &bitvec![Msb0, u8; 1, 0]
/// );
/// ```
/// Initialization from [`HashMap<L, usize>`][HashMap]:
/// ```
/// use huff_coding::{
///     bitvec::prelude::*,
///     prelude::{HuffTree, Weights},
/// };
/// use std::collections::HashMap;
/// 
/// let mut weights = HashMap::new();
/// weights.insert(String::from("pudzian"), 1);
/// weights.insert(String::from("krol"), 2);
/// weights.insert(String::from("szef"), 3);
/// 
/// let tree = HuffTree::from_weights(weights);
/// let codes = tree.read_codes();
/// 
/// assert_eq!(
///     codes.get("szef").unwrap(), 
///     &bitvec![Msb0, u8; 0]
/// );
/// assert_eq!(
///     codes.get("krol").unwrap(),
///     &bitvec![Msb0, u8; 1, 1]
/// );
/// assert_eq!(
///     codes.get("pudzian").unwrap(),
///     &bitvec![Msb0, u8; 1, 0]
/// );
/// ```
/// Representing and reading the tree from bits:
/// ```
/// use huff_coding::prelude::{HuffTree, ByteWeights};
/// 
/// let tree = HuffTree::from_weights(
///     ByteWeights::from_bytes(b"abbccc")
/// );
/// 
/// let tree_bin = tree.as_bin();
/// // the tree's root's left child is a letter branch, which are encoded by a 0 
/// assert_eq!(*tree_bin.get(1).unwrap(), false);
/// 
/// let new_tree = HuffTree::try_from_bin(tree_bin).unwrap();
/// // the newly created tree is identical, except in weights
/// assert_eq!(
///     tree.read_codes(),
///     new_tree.read_codes()
/// );
/// assert_ne!(
///     tree
///         .root()
///         .leaf()
///         .weight(), 
///     new_tree
///         .root()
///         .leaf()
///         .weight()
/// );
/// // every weight in a HuffTree read from binary is set to 0 
/// assert_eq!(
///     new_tree
///         .root()
///         .leaf()
///         .weight(),
///      0
/// );
/// ```
/// 
/// # Panics
/// ---
/// When trying to create a `HuffTree<L>` from a type implementing 
/// [`Weights<L>`][weights] with len == 0:
/// ```should_panic
/// use huff_coding::prelude::{HuffTree, Weights};
/// use std::collections::HashMap;
/// 
/// let weights = HashMap::<char, usize>::new();
/// 
/// // panics here at 'provided empty weights'
/// let tree = HuffTree::from_weights(weights);
/// ```
/// 
/// # Errors
/// ---
/// When trying to create a `HuffTree<L>` from binary where the original's
/// letter type is different than the one specified to be read:
/// ```should_panic
/// use huff_coding::prelude::{HuffTree, ByteWeights};
/// 
/// let tree = HuffTree::from_weights(
///     ByteWeights::from_bytes(b"abbccc")
/// );
/// let tree_bin = tree.as_bin();
/// let new_tree = HuffTree::<u128>::try_from_bin(tree_bin)
///     .expect("this will return a FromBinError");
/// ```
/// or when providing a too small/big BitVec to create a HuffTree<L>:
/// ```should_panic
/// use huff_coding::{
///     bitvec::prelude::*,
///     prelude::{HuffTree, ByteWeights},
/// };
/// 
/// let tree = HuffTree::<u128>::try_from_bin(bitvec![Msb0, u8; 0, 1])
///     .expect("this will return a FromBinError (provided BitVec is to small)");
/// ```
/// 
/// [branch]:crate::tree::branch::HuffBranch
/// [letter]:crate::tree::letter::HuffLetter
/// [letter_bytes]:crate::tree::letter::HuffLetterAsBytes
/// [weights]:crate::weights::Weights
/// [byte_weights]:crate::weights::ByteWeights
/// [huff_wiki]:https://en.wikipedia.org/wiki/Huffman_coding
/// [end_wiki]:https://en.wikipedia.org/wiki/Endianness
#[derive(Debug, Clone)]
pub struct HuffTree<L: HuffLetter>{
    root: HuffBranch<L>,
}

impl<L: HuffLetter> HuffTree<L>{
    /// Initialize the `HuffTree` with a struct implementing the [`Weights<L>`][weights] trait,
    /// where `L` implements [`HuffLetter`][letter]
    /// 
    /// In order to get the tree represented in binary([`Bitvec<Msb0, u8>`][bitvec::prelude::BitVec]) you must ensure 
    /// that `L` also implements [`HuffLetterAsBytes`][letter_bytes]
    /// 
    /// # Examples
    /// ---
    /// Initialization from [`ByteWeights`][byte_weights]
    /// ```
    /// use huff_coding::{
    ///     bitvec::prelude::*,
    ///     prelude::{HuffTree, ByteWeights},
    /// };
    /// use std::collections::HashMap;
    /// 
    /// let tree = HuffTree::from_weights(
    ///     ByteWeights::from_bytes(b"deefff")
    /// );
    /// let codes = tree.read_codes();
    /// 
    /// assert_eq!(
    ///     codes.get(&b'f').unwrap(),
    ///     &bitvec![Msb0, u8; 0]
    /// );
    /// assert_eq!(
    ///     codes.get(&b'e').unwrap(),
    ///     &bitvec![Msb0, u8; 1, 1]
    /// );
    /// assert_eq!(
    ///     codes.get(&b'd').unwrap(),
    ///     &bitvec![Msb0, u8; 1, 0]
    /// );
    /// ```
    /// Initialization from [`HashMap<L, usize>`][std::collections::HashMap]:
    /// ```
    /// use huff_coding::{
    ///     bitvec::prelude::*,
    ///     prelude::{HuffTree, Weights},
    /// };
    /// use std::collections::HashMap;
    /// 
    /// let mut weights = HashMap::new();
    /// weights.insert('ą', 1);
    /// weights.insert('þ', 2);
    /// weights.insert('😎', 3);
    /// 
    /// let tree = HuffTree::from_weights(weights);
    /// let codes = tree.read_codes();
    /// 
    /// assert_eq!(
    ///     codes.get(&'😎').unwrap(),
    ///     &bitvec![Msb0, u8; 0]
    /// );
    /// assert_eq!(
    ///     codes.get(&'þ').unwrap(),
    ///     &bitvec![Msb0, u8; 1, 1]
    /// );
    /// assert_eq!(
    ///     codes.get(&'ą').unwrap(),
    ///     &bitvec![Msb0, u8; 1, 0]
    /// );
    /// ```
    /// 
    /// # Panics
    /// ---
    /// When trying to create a `HuffTree<L>` from a type implementing 
    /// [`Weights<L>`][weights] with len == 0:
    /// ```should_panic
    /// use huff_coding::prelude::{HuffTree, Weights};
    /// use std::collections::HashMap;
    /// 
    /// let weights = HashMap::<char, usize>::new();
    /// 
    /// // panics here at 'provided empty weights'
    /// let tree = HuffTree::from_weights(weights);
    /// ```
    /// 
    /// [letter]:crate::tree::letter::HuffLetter
    /// [letter_bytes]:crate::tree::letter::HuffLetterAsBytes
    /// [weights]:crate::weights::Weights
    /// [byte_weights]:crate::weights::ByteWeights
    pub fn from_weights<W: Weights<L>>(weights: W) -> Self{
        // panic when provided with empty weights
        if weights.is_empty(){
            panic!("provided empty weights")
        }

        let mut branch_heap = HuffBranchHeap::from_weights(weights);

        while branch_heap.len() > 1{
            // get the min pair, removing it from the heap
            let min = branch_heap.pop_min();
            let next_min = branch_heap.pop_min();

            // initialize a joint branch and push it onto the heap
            let branch = HuffBranch::new(
                HuffLeaf::new(
                    None,
                    min.leaf().weight() + next_min.leaf().weight()
                ),
                Some((min, next_min))
            );
            branch_heap.push(branch);
        }

        // last branch in branch_heap is root
        let mut root = branch_heap.pop_min();

        // set codes for all branches recursively if has children
        // else just set the root's code to 0
        if root.has_children(){
            HuffTree::set_codes_in_child_branches(&mut root, None);
        }
        else{
            root.set_code({let mut c = BitVec::with_capacity(1); c.push(false); c});
        }

        HuffTree{
            root
        }
    }

    /// Return a reference to the tree's root branch
    pub fn root(&self) -> &HuffBranch<L>{
        &self.root
    }

    /// Return a mutable reference to the tree's root branch
    pub fn root_mut(&mut self) -> &mut HuffBranch<L>{
        &mut self.root
    }

    /// Go down the tree reading every letter's code and returning
    /// a [`HashMap<L, BitVec<Msb0, u8>>`][HashMap]
    /// 
    /// # Example
    /// ---
    /// ```
    /// use huff_coding::{
    ///     bitvec::prelude::*,
    ///     prelude::{HuffTree, ByteWeights},
    /// };
    /// use std::collections::HashMap;
    /// 
    /// let tree = HuffTree::from_weights(
    ///     ByteWeights::from_bytes(b"ghhiii")
    /// );
    /// let codes = tree.read_codes();
    /// 
    /// let mut cmp_codes = HashMap::new();
    /// cmp_codes.insert(b'i', bitvec![Msb0, u8; 0]);
    /// cmp_codes.insert(b'h', bitvec![Msb0, u8; 1, 1]);
    /// cmp_codes.insert(b'g', bitvec![Msb0, u8; 1, 0]);
    /// 
    /// assert_eq!(codes, cmp_codes);
    /// ```
    pub fn read_codes(&self) -> HashMap<L, BitVec<Msb0, u8>>{
        self.read_codes_with_hasher(RandomState::default())
    }

    /// Go down the tree reading every letter's code and returning
    /// a [`HashMap<L, BitVec<Msb0, u8>, S>][HashMap]` where `S` 
    /// is the provided hash builder (implementing [`BuildHasher`][BuildHasher])
    /// 
    /// # Example
    /// ---
    /// ```
    /// use huff_coding::{
    ///     bitvec::prelude::*,
    ///     prelude::{HuffTree, ByteWeights},
    /// };
    /// use std::collections::{
    ///     HashMap,
    ///     hash_map::RandomState,
    /// };
    /// 
    /// let tree = HuffTree::from_weights(
    ///     ByteWeights::from_bytes(b"ghhiii")
    /// );
    /// let codes = tree.read_codes_with_hasher(RandomState::default());
    /// 
    /// let mut cmp_codes = HashMap::new();
    /// cmp_codes.insert(b'i', bitvec![Msb0, u8; 0]);
    /// cmp_codes.insert(b'h', bitvec![Msb0, u8; 1, 1]);
    /// cmp_codes.insert(b'g', bitvec![Msb0, u8; 1, 0]);
    /// 
    /// assert_eq!(codes, cmp_codes);
    /// ```
    pub fn read_codes_with_hasher<S: BuildHasher>(&self, hash_builder: S) -> HashMap<L, BitVec<Msb0, u8>, S>{
        /// Recursively insert letters to codes into the given HashMap<L, BitVec<Msb0, u8>>
        fn set_codes<L: HuffLetter, S: BuildHasher>(codes: &mut HashMap<L, BitVec<Msb0, u8>, S>, root: &HuffBranch<L>, pos_in_parent: bool){
            if let Some(children_iter) = root.children_iter(){
                for (pos, child) in children_iter.enumerate(){
                    let branch = child;
                    let leaf = branch.leaf();
                    if let Some(letter) = leaf.letter(){
                        codes.insert(letter.clone(), leaf.code().unwrap().clone());
                    }
                    else{
                        set_codes(codes, child, pos != 0);
                    }
                }
            }  
            else{
                codes.insert(root.leaf().letter().unwrap().clone(), bitvec![Msb0, u8; pos_in_parent as u8]);
            }
        }
        
        let mut codes = HashMap::with_hasher(hash_builder);
        let root = self.root();
        if root.has_children(){
            set_codes(&mut codes, root.left_child().unwrap(), false);
            set_codes(&mut codes, root.right_child().unwrap(), true);
            codes
        }
        else{
            codes.insert(root.leaf().letter().unwrap().clone(), bitvec![Msb0, u8; 0]);
            codes
        }
    }

    /// Recursively set the codes in every encountered branch
    fn set_codes_in_child_branches(parent: &mut HuffBranch<L>, parent_code: Option<BitVec<Msb0, u8>>){
        if parent.has_children(){
            let set_code = |child: &mut HuffBranch<L>, pos|{
                // append pos_in_parent to parent_code and set the newly created code on child
                let mut child_code = BitVec::with_capacity(1);
                if let Some(parent_code) = parent_code{
                    child_code = parent_code;
                }   
                child_code.push(pos != 0);
                child.set_code(child_code.clone());
    
                // recurse into the child's children
                HuffTree::set_codes_in_child_branches(child, Some(child_code));
            };
            
            set_code.clone()(parent.left_child_mut().unwrap(), 0);
            set_code(parent.right_child_mut().unwrap(), 1);
        }
    }
}

impl<L: HuffLetterAsBytes> HuffTree<L>{
    /// Try to read the provided [`BitVec<Msb0, u8>`][bitvec::prelude::BitVec] and
    /// construct a `HuffTree<L>` from it.
    /// Every weight in the newly created tree is set to 0 
    /// as they're not stored in the binary representation
    /// 
    /// In order to call this method, `L` must implement [`HuffLetterAsBytes`][letter_bytes]
    /// 
    /// # Decoding scheme
    /// ---
    /// 1. Go bit by bit
    /// 2. Create a [`HuffBranch`][branch] with no letter (a joint branch) when a 1 is found
    /// 3. When a 0 is found, read next [`size_of::<L>() * 8`][mem::size_of] bits and create a
    /// value of type `L` from them, inserting it then into a [`HuffBranch`][branch]
    /// 
    /// # Example
    /// ---
    /// ```
    /// use huff_coding::prelude::{HuffTree, ByteWeights};
    /// 
    /// let tree = HuffTree::from_weights(
    ///     ByteWeights::from_bytes(b"mnnooo")
    /// );
    /// 
    /// let tree_bin = tree.as_bin();
    /// 
    /// let new_tree = HuffTree::try_from_bin(tree_bin).unwrap();
    /// // the newly created tree is identical, except in weights
    /// assert_eq!(
    ///     tree.read_codes(),
    ///     new_tree.read_codes()
    /// );
    /// assert_ne!(
    ///     tree
    ///         .root()
    ///         .leaf()
    ///         .weight(), 
    ///     new_tree
    ///         .root()
    ///         .leaf()
    ///         .weight()
    /// );
    /// // every weight in a HuffTree read from binary is set to 0 
    /// assert_eq!(
    ///     new_tree
    ///         .root()
    ///         .leaf()
    ///         .weight(),
    ///      0
    /// );
    /// ```
    /// 
    /// # Errors
    /// ---
    /// When trying to create a `HuffTree<L>`from binary where the original's
    /// letter type is different than the one specified to be read:
    /// ```should_panic
    /// use huff_coding::prelude::{HuffTree, ByteWeights};
    /// 
    /// let tree = HuffTree::from_weights(
    ///     ByteWeights::from_bytes(b"abbccc")
    /// );
    /// let tree_bin = tree.as_bin();
    /// let new_tree = HuffTree::<u128>::try_from_bin(tree_bin)
    ///     .expect("this will return a FromBinError");
    /// ```
    /// or when providing a too small/big BitVec to create a HuffTree<L>:
    /// ```should_panic
    /// use huff_coding::{
    ///     bitvec::prelude::*,
    ///     prelude::{HuffTree, ByteWeights},
    /// };
    /// 
    /// let tree = HuffTree::<u128>::try_from_bin(bitvec![Msb0, u8; 0, 1])
    ///     .expect("this will return a FromBinError (provided BitVec is to small)");
    /// ```
    /// 
    /// [branch]:crate::tree::branch::HuffBranch
    /// [letter_bytes]:crate::tree::letter::HuffLetterAsBytes
    pub fn try_from_bin(bin: BitVec<Msb0, u8>) -> Result<Self, FromBinError<L>>{
        /// Recursively reads branches and their children from the given bits
        /// When finding a 1 -> recurses to get children,
        /// and when a 0 -> ends recursion returning a letter branch
        fn read_branches_from_bits<L: HuffLetterAsBytes>(bits: &mut bitvec::slice::IterMut<Msb0, u8>) -> 
        Result<HuffBranch<L>, FromBinError<L>>{
            // check whether the bit can be popped at all, if not return Err
            // remove first bit, if its 1 -> joint branch
            if if let Some(bit) = bits.next(){*bit}
                else{
                    return Err(FromBinError::new(
                        "Provided BitVec is too small for an encoded HuffTree"
                    ))
                }{
                // create joint branch, recurse to get its children
                let branch = HuffBranch::new(
                    HuffLeaf::new(None, 0),
                    Some(( 
                        read_branches_from_bits(bits)?, 
                        read_branches_from_bits(bits)?
                    ))
                );
                Ok(branch)
            }
            // if it's 0 -> letter branch
            else{
                // read the letter bits and convert them to bytes
                let mut letter_bytes = Vec::<u8>::with_capacity(mem::size_of::<L>());
                let mut byte = 0b0000_0000;
                let mut bit_ptr = 7;

                // get an iterator over the letter bits, if not enough bits left return err
                let letter_bits = bits.take(size_of_bits::<L>());
                if letter_bits.len() != size_of_bits::<L>(){
                    return Err(FromBinError::new(
                        "Provided BitVec is too small for an encoded HuffTree", 
                    ))
                };
                for bit in letter_bits{
                    byte |= (*bit as u8) << bit_ptr;
                    if bit_ptr == 0{
                        letter_bytes.push(byte);
                        byte = 0b0000_0000;
                        bit_ptr = 7;
                    }
                    else{bit_ptr -= 1};
                }
                
                // create letter branch (no children)
                let branch = HuffBranch::new(
                    // create letter from letter_bytes
                    HuffLeaf::new(Some(L::try_from_be_bytes(&letter_bytes).unwrap()), 0),
                    None,
                );
                Ok(branch)
            }
        }
        // declare bin as mutable
        let mut bin = bin;
        // recurse to create root, and set codes for all branches
        let mut bin_iter_mut = bin.iter_mut();
        let mut root = read_branches_from_bits(&mut bin_iter_mut)?;

        // return Err if not all bits used
        if bin_iter_mut.next().is_some(){
            return Err(FromBinError::new(
                "Provided BitVec is too big for an encoded HuffTree", 
            ))
        }

        // set codes for all branches recursively if has children
        // else just set the root's code to 0
        if root.has_children(){
            HuffTree::set_codes_in_child_branches(&mut root, None);
        }
        else{
            root.set_code(bitvec![Msb0, u8; 0]);
        }
        
        Ok(HuffTree{
            root
        })
    }

    /// Return a binary representation of the `HuffTree<L>` 
    /// ([`BitVec<Msb0, u8>`][bitvec::prelude::BitVec])
    /// 
    /// In order to call this method, `L` must implement [`HuffLetterAsBytes`][letter_bytes]
    /// 
    /// # Encoding scheme
    /// ---
    /// 1. Recurse down the tree
    /// 2. Every joint branch is encoded as a 1
    /// 3. Every letter branch is encoded as a 0
    /// and is followed by the letter itself encoded in binary
    /// 
    /// # Example
    /// ---
    /// ```
    /// use huff_coding::prelude::{HuffTree, ByteWeights};
    /// 
    /// let tree = HuffTree::from_weights(
    ///     ByteWeights::from_bytes(b"abbccc")
    /// );
    /// 
    /// let tree_bin = tree.as_bin();
    /// assert_eq!(tree_bin.to_string(), "[10011000, 11100110, 00010011, 00010]");
    /// ```
    /// 
    /// [letter_bytes]:crate::tree::letter::HuffLetterAsBytes
    pub fn as_bin(&self) -> BitVec<Msb0, u8>{
        /// Recursively push bits to the given BitVec<Msb0, u8>
        /// depending on the branches you encounter:
        /// * 0 being a letter branch (followed by a letter encoded in binary)
        /// * 1 being a joint branch
        fn set_tree_as_bin<L: HuffLetterAsBytes>(tree_bin: &mut BitVec<Msb0, u8>, root: &HuffBranch<L>){
            let root = root;
            let children_iter = root.children_iter();

            // has children -> joint branch
            if let Some(children_iter) = children_iter{
                // 1 means joint branch
                tree_bin.push(true);

                // call set_bin on children
                for child in children_iter{
                    set_tree_as_bin(tree_bin, child);
                }
            }
            // no children -> letter branch
            else{
                // 0 means letter branch
                tree_bin.push(false);

                // convert the letter to bytes and push the bytes' bits into the tree_bin
                for byte in root.leaf().letter().unwrap().as_be_bytes().iter(){
                    for bit_ptr in 0..8{
                        tree_bin.push((byte >> (7 - bit_ptr)) & 1 == 1)
                    }
                }
            }
        }

        let mut treebin = BitVec::new();
        set_tree_as_bin(&mut treebin, self.root());
        treebin
    }
}

/// [Error][std::error::Error] encountered while trying to construct a [`HuffTree`][HuffTree] from bin
/// with the [`HuffTree::try_from_bin`](struct.HuffTree.html#method.try_from_bin) method
#[derive(Debug)]
pub struct FromBinError<L: HuffLetterAsBytes>{
    message: &'static str,
    _typebind: std::marker::PhantomData<L>,
}

impl<L: HuffLetterAsBytes> fmt::Display for FromBinError<L>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}<{}>", self.message, std::any::type_name::<L>())
    }
}

impl<L: HuffLetterAsBytes> std::error::Error for FromBinError<L>{}

impl<L: HuffLetterAsBytes> FromBinError<L>{
    /// Initialize a new `FromBinError` with the given message
    pub fn new(message: &'static str) -> Self{
        Self{
            message,
            _typebind: std::marker::PhantomData,
        }
    }

    /// Return the message
    pub fn message(&self) -> &str{
        self.message
    }
}
