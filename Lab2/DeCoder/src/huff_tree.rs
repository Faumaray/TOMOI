pub mod huff_tree {
    use std::{
        collections::{hash_map::RandomState, HashMap},
        hash::BuildHasher,
        io::Error,
        marker::PhantomData,
        mem,
    };
    pub fn size_of_bits<T>() -> usize {
        std::mem::size_of::<T>() * 8
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
    fn set_codes_in_child_branches(parent: &mut Node, parent_code: Option<BitVec<Msb0, u8>>) {
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
    use bitvec::prelude::*;
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
        pub fn root(&self) -> &Node {
            &self.root
        }
        pub fn try_from_bin(bin: BitVec<Msb0, u8>) -> Result<Self, Error> {
            /// Recursively reads branches and their children from the given bits
            /// When finding a 1 -> recurses to get children,
            /// and when a 0 -> ends recursion returning a letter branch
            fn read_branches_from_bits(
                bits: &mut bitvec::slice::IterMut<Msb0, u8>,
            ) -> Result<Node, Error> {
                // check whether the bit can be popped at all, if not return Err
                // remove first bit, if its 1 -> joint branch
                if if let Some(bit) = bits.next() {
                    *bit
                } else {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Provided BitVec is too big for an encoded HuffTree",
                    ));
                } {
                    // create joint branch, recurse to get its children
                    let branch = Node::new(
                        None,
                        0,
                        Some((
                            read_branches_from_bits(bits)?,
                            read_branches_from_bits(bits)?,
                        )),
                    );
                    Ok(branch)
                }
                // if it's 0 -> letter branch
                else {
                    // read the letter bits and convert them to bytes
                    let mut letter_bytes = Vec::<u8>::with_capacity(mem::size_of::<char>());
                    let mut byte = 0b0000_0000;
                    let mut bit_ptr = 7;

                    // get an iterator over the letter bits, if not enough bits left return err
                    let letter_bits = bits.take(size_of_bits::<char>());
                    if letter_bits.len() != size_of_bits::<char>() {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Provided BitVec is too big for an encoded HuffTree",
                        ));
                    };
                    for bit in letter_bits {
                        byte |= (*bit as u8) << bit_ptr;
                        if bit_ptr == 0 {
                            letter_bytes.push(byte);
                            byte = 0b0000_0000;
                            bit_ptr = 7;
                        } else {
                            bit_ptr -= 1
                        };
                    }
                    // create letter branch (no children)
                    println!("{:?}", letter_bytes);
                    let branch = Node::new(
                        // create letter from letter_bytes
                        Some(
                            String::from_utf8(letter_bytes)
                                .unwrap()
                                .chars()
                                .collect::<Vec<char>>()[0],
                        ),
                        0,
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
            if bin_iter_mut.next().is_some() {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Provided BitVec is too big for an encoded HuffTree",
                ));
            }

            // set codes for all branches recursively if has children
            // else just set the root's code to 0
            if root.has_children() {
                set_codes_in_child_branches(&mut root, None);
            } else {
                root.set_code(bitvec![Msb0, u8; 0]);
            }

            Ok(Tree {
                root: Box::new(root),
            })
        }
        pub fn read_codes_with_hasher<S: BuildHasher>(
            &self,
            hash_builder: S,
        ) -> HashMap<char, BitVec<Msb0, u8>, S> {
            /// Recursively insert letters to codes into the given HashMap<L, BitVec<Msb0, u8>>
            fn set_codes<S: BuildHasher>(
                codes: &mut HashMap<char, BitVec<Msb0, u8>, S>,
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
                        bitvec![Msb0, u8; pos_in_parent as u8],
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
                codes.insert(root.letter().unwrap().clone(), bitvec![Msb0, u8; 0]);
                codes
            }
        }
        pub fn read_codes(&self) -> HashMap<char, BitVec<Msb0, u8>> {
            self.read_codes_with_hasher(RandomState::default())
        }
        pub fn as_bin(&self) -> BitVec<Msb0, u8> {
            /// Recursively push bits to the given BitVec<Msb0, u8>
            /// depending on the branches you encounter:
            /// * 0 being a letter branch (followed by a letter encoded in binary)
            /// * 1 being a joint branch
            fn set_tree_as_bin(tree_bin: &mut BitVec<Msb0, u8>, root: &Node) {
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
                    let mut tmp = [0u8; 4];
                    // convert the letter to bytes and push the bytes' bits into the tree_bin
                    for byte in root.letter.unwrap().encode_utf8(&mut tmp).as_bytes().iter() {
                        for bit_ptr in 0..8 {
                            tree_bin.push((byte >> (7 - bit_ptr)) & 1 == 1)
                        }
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
        code: Option<BitVec<Msb0, u8>>,
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
        pub fn new(letter: Option<char>, weight: usize, children: Option<(Node, Node)>) -> Self {
            if let Some(children) = children {
                Node {
                    letter: letter,
                    freq: weight,
                    code: None,
                    left: Some(Box::new(children.0)),
                    right: Some(Box::new(children.1)),
                }
            } else {
                Node {
                    letter: letter,
                    freq: weight,
                    code: None,
                    left: None,
                    right: None,
                }
            }
        }
        pub fn code(&self) -> Option<&BitVec<Msb0, u8>> {
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
        pub fn set_code(&mut self, code: BitVec<Msb0, u8>) {
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
}
