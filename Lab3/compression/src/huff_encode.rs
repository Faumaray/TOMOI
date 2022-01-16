use std::{
    collections::{hash_map::RandomState, HashMap},
    convert::TryFrom,
    convert::TryInto,
    fmt::Debug,
    hash::BuildHasher,
    hash::Hash,
    io::Error,
    marker::PhantomData,
    mem,
    mem::size_of,
    str::FromStr,
};
pub trait HuffLetter: Clone + Eq + Hash + Debug {}
/// Trait specifying that the given HuffLetter can be converted
/// into bytes *(returns `Box<[u8]>`)* and
/// can be created from bytes (`&[u8]`),
/// so the [`HuffTree`][crate::tree::HuffTree] can be represented in binary.
///
/// Implemented by default for every integer
pub trait HuffLetterAsBytes: HuffLetter {
    fn try_from_be_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>>;
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
primitive_letter_impl! {
    char,
    &str,
    String
}
#[derive(Debug, Clone)]
pub struct CompressData {
    comp_bytes: Vec<u8>,
    padding_bits: u8,
    huff_tree: Tree,
    _typebind: PhantomData<char>,
}

impl CompressData {
    /// Initialize a new instance of `CompressData` with the provided
    /// compressed bytes, padding bits and [`HuffTree`][crate::tree::HuffTree].
    ///
    /// # Panics
    /// When providing an empty `comp_bytes` or
    /// when providing `padding_bits` larger than 7.
    pub fn new(comp_bytes: Vec<u8>, padding_bits: u8, huff_tree: Tree) -> Self {
        if comp_bytes.is_empty() {
            panic!("provided comp_bytes are empty")
        }
        if padding_bits > 7 {
            panic!("padding bits cannot be larger than 7")
        }
        Self {
            comp_bytes,
            padding_bits,
            huff_tree,
            _typebind: PhantomData::default(),
        }
    }
    pub fn into_inner(self) -> (Vec<u8>, u8, Tree) {
        (self.comp_bytes, self.padding_bits, self.huff_tree)
    }
}

pub fn compress_with_tree(letters: String, huff_tree: Tree) -> Result<CompressData, Error> {
    let mut comp_letters = Vec::with_capacity(letters.len());
    let codes = huff_tree.read_codes();
    let mut comp_byte = 0b0000_0000;
    let mut bit_ptr = 7;
    for letter in letters.chars() {
        // return Err if there's no code
        let code = if let Some(code) = codes.get(&letter) {
            Ok(code)
        } else {
            Err(Error::new(
                std::io::ErrorKind::NotFound,
                format!("letter not found in codes {}", letter.clone()),
            ))
        }?;
        for bit in code {
            // set bit on current byte
            comp_byte |= (*bit as u8) << bit_ptr;
            // if filled comp_byte
            if bit_ptr == 0 {
                comp_letters.push(comp_byte);
                comp_byte = 0b0000_0000;
                bit_ptr = 7;
            } else {
                bit_ptr -= 1
            };
        }
    }
    // calculate the compressed_letters' padding bits
    let padding_bits = if bit_ptr == 7 { 0 } else { bit_ptr + 1 };
    if padding_bits != 0 {
        comp_letters.push(comp_byte);
    }

    Ok(CompressData::new(comp_letters, padding_bits, huff_tree))
}
fn set_codes_in_child_branches(parent: &mut Node, parent_code: Option<BitVec<u8, Msb0>>) {
    if parent.has_children() {
        let set_code = |child: &mut Node, pos| {
            // append pos_in_parent to parent_code and set the newly created code on child
            let mut child_code = BitVec::with_capacity(1);
            if let Some(parent_code) = parent_code {
                child_code = parent_code;
            }
            child_code.push(pos != 0);
            child.set_code(child_code.clone());

            // recurse into the child's children
            set_codes_in_child_branches(child, Some(child_code));
        };

        set_code.clone()(parent.left_child_mut().unwrap(), 0);
        set_code(parent.right_child_mut().unwrap(), 1);
    }
}
pub fn calc_padding_bits(bit_count: usize) -> u8 {
    let n = (8 - bit_count % 8) as u8;
    match n {
        8 => 0,
        _ => n,
    }
}
use bitvec::{prelude::*, view::AsBits};
#[derive(Debug, Clone)]
pub struct Tree {
    root: Box<Node>,
}
pub struct ChildrenIter<'a> {
    parent: &'a Node,
    child_pos: u8,
}

impl<'a> Iterator for ChildrenIter<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.child_pos {
            0 => {
                self.child_pos += 1;
                self.parent.left_child()
            }
            1 => {
                self.child_pos += 1;
                self.parent.right_child()
            }
            _ => None,
        }
    }
}

impl<'a> ChildrenIter<'a> {
    /// Initialize a new ```ChildrenIter``` over
    /// the children of the provided ```HuffBranch```
    pub fn new(parent: &'a Node) -> Self {
        ChildrenIter {
            parent,
            child_pos: 0,
        }
    }
}
impl Tree {
    pub fn read_codes_with_hasher<S: BuildHasher>(
        &self,
        hash_builder: S,
    ) -> HashMap<char, BitVec<u8, Msb0>, S> {
        /// Recursively insert letters to codes into the given HashMap<L, BitVec<Msb0, u8>>
        fn set_codes<S: BuildHasher>(
            codes: &mut HashMap<char, BitVec<u8, Msb0>, S>,
            root: &Node,
            pos_in_parent: bool,
        ) {
            if let Some(children_iter) = root.children_iter() {
                for (pos, child) in children_iter.enumerate() {
                    let branch = child;
                    if let Some(letter) = branch.letter() {
                        codes.insert(letter.clone(), branch.code().unwrap().clone());
                    } else {
                        set_codes(codes, child, pos != 0);
                    }
                }
            } else {
                codes.insert(
                    root.letter().unwrap().clone(),
                    bitvec![u8,Msb0; pos_in_parent as u8;1],
                );
            }
        }

        let mut codes = HashMap::with_hasher(hash_builder);
        let root = &self.root;
        if root.has_children() {
            set_codes(&mut codes, root.left_child().unwrap(), false);
            set_codes(&mut codes, root.right_child().unwrap(), true);
            codes
        } else {
            codes.insert(root.letter().unwrap().clone(), bitvec![u8,Msb0; 0;1]);
            codes
        }
    }
    pub fn read_codes(&self) -> HashMap<char, BitVec<u8, Msb0>> {
        self.read_codes_with_hasher(RandomState::default())
    }
    pub fn as_bin(&self) -> BitVec<u8, Msb0> {
        /// Recursively push bits to the given BitVec<Msb0, u8>
        /// depending on the branches you encounter:
        /// * 0 being a letter branch (followed by a letter encoded in binary)
        /// * 1 being a joint branch
        fn set_tree_as_bin(tree_bin: &mut BitVec<u8, Msb0>, root: &Node) {
            let root = root;
            let children_iter = root.children_iter();

            // has children -> joint branch
            if let Some(children_iter) = children_iter {
                // 1 means joint branch
                tree_bin.push(true);

                // call set_bin on children
                for child in children_iter {
                    set_tree_as_bin(tree_bin, &child);
                }
            }
            // no children -> letter branch
            else {
                // 0 means letter branch
                tree_bin.push(false);
                let mut vc = root.letter().unwrap().to_string().as_bytes().to_vec();
                for _i in 0..(4 - vc.len()) {
                    vc.push(0u8);
                }
                vc.shrink_to_fit();
                let mut tmp = [0u8; 4];
                let c = root.letter().unwrap().encode_utf8(&mut tmp);

                // convert the letter to bytes and push the bytes' bits into the tree_bin
                for byte in BitVec::<u8, Msb0>::from_vec(tmp.to_vec()) {
                    tree_bin.push(byte);
                }
            }
        }

        let mut treebin = BitVec::new();
        set_tree_as_bin(&mut treebin, &self.root);
        treebin
    }
    pub fn assign_codes(&mut self) {
        if self.root.has_children() {
            set_codes_in_child_branches(&mut self.root, None);
        } else {
            self.root.set_code({
                let mut c = BitVec::with_capacity(1);
                c.push(false);
                c
            });
        }
    }
    pub fn build_from_weights(weights: HashMap<char, usize>) -> Self {
        let mut p: Vec<Box<Node>> = weights
            .iter()
            .map(|x| {
                Box::new(Node {
                    letter: Some(*x.0),
                    freq: *x.1,
                    code: None,
                    left: None,
                    right: None,
                })
            })
            .collect();
        while p.len() > 1 {
            p.sort_by(|a, b| (&(b.freq)).cmp(&(a.freq)));
            let a = p.pop().unwrap();
            let b = p.pop().unwrap();
            let mut c = Box::new(Node {
                letter: None,
                freq: a.freq + b.freq,
                code: None,
                left: None,
                right: None,
            });
            c.left = Some(a);
            c.right = Some(b);
            p.push(c);
        }
        Self {
            root: p.pop().unwrap(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Node {
    letter: Option<char>,
    freq: usize,
    code: Option<BitVec<u8, Msb0>>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}
impl Node {
    pub fn children_iter(&self) -> Option<ChildrenIter> {
        if self.has_children() {
            Some(ChildrenIter::new(self))
        } else {
            None
        }
    }
    pub fn code(&self) -> Option<&BitVec<u8, Msb0>> {
        self.code.as_ref()
    }
    pub fn letter(&self) -> Option<char> {
        self.letter
    }
    pub fn left_child(&self) -> Option<&Node> {
        self.left.as_deref()
    }

    pub fn right_child(&self) -> Option<&Node> {
        self.right.as_deref()
    }
    pub fn left_child_mut(&mut self) -> Option<&mut Node> {
        self.left.as_deref_mut()
    }

    pub fn right_child_mut(&mut self) -> Option<&mut Node> {
        self.right.as_deref_mut()
    }
    pub fn has_children(&self) -> bool {
        self.left.is_some()
    }
    pub fn set_code(&mut self, code: BitVec<u8, Msb0>) {
        self.code = Some(code);
    }
}

pub fn build_weights(s: &str) -> HashMap<char, usize> {
    let mut h = HashMap::new();
    for ch in s.chars() {
        let counter = h.entry(ch).or_insert(0);
        *counter += 1;
    }
    h
}
