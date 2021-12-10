pub use self::byte_weights::ByteWeights;

use super::tree::letter::HuffLetter;

use std::{
    collections::{
        HashMap,
        hash_map::RandomState,
    },
    hash::{
        Hash, 
        BuildHasher
    },
};

/// Trait signifying that the struct stores the weights of a certain type (letter), so that
/// for any stored letter there is a corresponding `usize`(weight).
/// 
/// Implemented by default for [`HashMap<L, usize>`][std::collections::HashMap] and
/// for [`ByteWeights`][byte_weights::ByteWeights]
/// 
/// Needed implementations:
/// * Traits:
///  * [`Eq`][Eq]
///  * [`Clone`][Clone]
///  * [`IntoIterator<Item = (L, usize)>`][IntoIterator]
/// * Methods:
///  * `fn get(&self, letter: &L) -> Option<&usize>`
///  * `fn get_mut(&mut self, letter: &L) -> Option<&mut usize>`
///  * `fn len(&self) -> usize`
///  * `fn is_empty(&self) -> bool`
/// 
/// In order to build with a [`HuffTree`][crate::tree::HuffTree] `L` must implement [`HuffLetter`][crate::tree::letter::HuffLetter]
pub trait Weights<L>: Eq + Clone + IntoIterator<Item = (L, usize)>{
    fn get(&self, letter: &L) -> Option<&usize>;
    fn get_mut(&mut self, letter: &L) -> Option<&mut usize>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<L: Eq + Clone + Hash> Weights<L> for HashMap<L, usize>{
    fn get(&self, letter: &L) -> Option<&usize>{
        self.get(letter)
    }
    fn get_mut(&mut self, letter: &L) -> Option<&mut usize>{
        self.get_mut(letter)
    }
    fn len(&self) -> usize{
        self.len()
    }
    fn is_empty(&self) -> bool{
        self.is_empty()
    }
}

/// Count every letter in the provided slice Returning a [`HashMap`][std::collections::HashMap]
/// of letters to their counts (weights)
/// 
/// # Example
/// ---
/// ```
/// use huff_coding::weights::build_weights_map;
/// 
/// let weights = build_weights_map(&[12, -543, 12, 66, 66, 66]);
/// 
/// assert_eq!(weights.get(&-543), Some(&1));
/// assert_eq!(weights.get(&12), Some(&2));
/// assert_eq!(weights.get(&66), Some(&3));
/// ```
/// The resulting [`HashMap`][std::collections::HashMap] 
/// can be used to build a [`HuffTree`][crate::tree::HuffTree]:
/// ```
/// use huff_coding::prelude::{
///     HuffTree,
///     build_weights_map,
/// };
/// 
/// let weights = build_weights_map(&['a', 'a', 'a', 'b', 'b', 'c']);
/// 
/// let tree = HuffTree::from_weights(weights);
/// ```
pub fn build_weights_map<L: HuffLetter>(letters: &[L]) -> HashMap<L, usize>{
    build_weights_map_with_hasher(letters, RandomState::default())
}

/// Count every letter in the provided slice Returning a [`HashMap`][std::collections::HashMap]
/// of letters to their counts (weights), with the provided hash builder.
/// 
/// # Example
/// ---
/// ```
/// use huff_coding::weights::build_weights_map;
/// 
/// let weights = build_weights_map(&[8, 6, 8, 12, 12, 12]);
/// 
/// assert_eq!(weights.get(&6), Some(&1));
/// assert_eq!(weights.get(&8), Some(&2));
/// assert_eq!(weights.get(&12), Some(&3));
/// ```
/// The resulting [`HashMap`][std::collections::HashMap] 
/// can be used to build a [`HuffTree`][crate::tree::HuffTree]:
/// ```
/// use huff_coding::prelude::{
///     HuffTree,
///     build_weights_map_with_hasher,
/// };
/// use std::collections::hash_map::RandomState;
/// 
/// let weights = build_weights_map_with_hasher(
///     &['d', 'd', 'd', 'e', 'e', 'f'],
///     RandomState::default()
/// );
/// 
/// let tree = HuffTree::from_weights(weights);
/// ```
pub fn build_weights_map_with_hasher<L: HuffLetter, S: BuildHasher>(letters: &[L], hash_builder: S) -> HashMap<L, usize, S>{
    let mut map = HashMap::with_hasher(hash_builder);
    for l in letters{
        let entry = map.entry(l.clone()).or_insert(0);
        *entry += 1;
    }
    map
}

/// Struct storing the number of occurences of each byte in
/// a provided byte slice.
pub mod byte_weights{
    use crate::utils::ration_vec;
    use super::Weights;

    use std::{
        ops::{Add, AddAssign},
        thread,
    };

    /// Struct storing the number of occurences of each byte in
    /// a provided byte slice.
    /// 
    /// A [`HuffTree`][crate::tree::HuffTree] can be initialized with it,
    /// as `ByteWeights` implements the [`Weights`][crate::weights::Weights] trait.
    /// 
    /// # Examples
    /// ---
    /// Initialization and interfacing:
    /// ```
    /// use huff_coding::prelude::ByteWeights;
    /// 
    /// let byte_weights = ByteWeights::from_bytes(b"fffff");
    /// assert_eq!(*byte_weights.get(&b'f').unwrap(), 5);
    /// assert_eq!(byte_weights.len(), 1);
    /// ```
    /// Iteration:
    /// ```
    /// use huff_coding::prelude::ByteWeights;
    /// 
    /// let byte_weights = ByteWeights::from_bytes(&[0, 1, 1, 2, 2, 2]);
    /// for (byte, weight) in byte_weights{
    ///     assert_eq!(byte as usize, weight - 1);
    /// }
    /// ```
    /// Adding two `ByteWeights`:
    /// ```
    /// use huff_coding::prelude::ByteWeights;
    /// 
    /// let mut byte_weights = ByteWeights::from_bytes(b"aabbb");
    /// let other = ByteWeights::from_bytes(b"aaabbc");
    /// 
    /// byte_weights += other;
    /// 
    /// assert_eq!(*byte_weights.get(&b'a').unwrap(), 5);
    /// assert_eq!(*byte_weights.get(&b'b').unwrap(), 5);
    /// assert_eq!(*byte_weights.get(&b'c').unwrap(), 1);
    /// ```
    #[derive(Clone, Copy, Eq)]
    pub struct ByteWeights{
        weights: [usize; 256],
        len: usize,
    }

    impl Weights<u8> for ByteWeights{
        fn get(&self, byte: &u8) -> Option<&usize>{
            self.get(byte)
        }

        fn get_mut(&mut self, byte: &u8) -> Option<&mut usize>{
            self.get_mut(byte)
        }

        fn len(&self) -> usize{
            self.len()
        }

        fn is_empty(&self) -> bool{
            self.is_empty()
        }
    }

    impl IntoIterator for ByteWeights{
        type Item = (u8, usize);
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter{
            IntoIter{weights: self, current_index: 0}
        }   
    }

    impl <'a> IntoIterator for &'a ByteWeights{
        type Item = (u8, usize);
        type IntoIter = Iter<'a>;

        fn into_iter(self) -> Iter<'a>{
            Iter{weights: &self, current_index: 0}
        }   
    }

    impl PartialEq for ByteWeights{
        fn eq(&self, other: &Self) -> bool {
            self.weights == other.weights
        }
    }

    impl Add for ByteWeights{
        type Output = Self;

        fn add(mut self, other: Self) -> Self {
            self.add_byte_weights(&other);
            self
        }
    }

    impl AddAssign for ByteWeights{
        fn add_assign(&mut self, other: Self){
            self.add_byte_weights(&other);
        }
    }

    impl Default for ByteWeights{
        fn default() -> Self{
            Self::new()
        }
    }

    impl ByteWeights{
        /// Initialize new empty `ByteWeights`
        pub fn new() -> Self{
            Self{
                weights: [0;256],
                len: 0,
            }
        }

        /// Initialize new `ByteWeights` from the given [`&[u8]`][u8]
        /// 
        /// This algorithm is inherently O(n), therefore for
        /// larger collections [`threaded_from_bytes`](#method.threaded_from_bytes) is faster.
        /// 
        /// # Example
        /// ---
        /// ```
        /// use huff_coding::prelude::ByteWeights;
        /// 
        /// let byte_weights = ByteWeights::from_bytes(b"aaaaa");
        /// assert_eq!(*byte_weights.get(&b'a').unwrap(), 5);
        /// ```
        pub fn from_bytes(bytes: &[u8]) -> Self{
            // count bytes into an array
            let mut weights: [usize; 256] = [0;256];
            let mut len = 0;

            for byte in bytes{
                if weights[*byte as usize] == 0{len += 1;}
                weights[*byte as usize] += 1;
            }
 
            ByteWeights{
                weights,
                len,
            }
        }

        /// Initialize new `ByteWeights` from the given [`&[u8]`][u8], but
        /// using the specified number of threads to speed up the
        /// process.
        /// 
        /// # Example
        /// ---
        /// ```
        /// use huff_coding::prelude::ByteWeights;
        /// 
        /// let byte_weights = ByteWeights::threaded_from_bytes(b"aaaaa", 12);
        /// assert_eq!(*byte_weights.get(&b'a').unwrap(), 5)
        /// ```
        pub fn threaded_from_bytes(bytes: &[u8], thread_num: usize) -> Self{
            // divide the bytes into rations per thread
            let byte_rations = ration_vec(bytes, thread_num);

            // create ByteWeights from every ration
            let mut handles = Vec::with_capacity(thread_num);
            for ration in byte_rations{
                let handle = thread::spawn(move || {
                    ByteWeights::from_bytes(&ration)
                });
                handles.push(handle);
            }

            // push all created ByteWeights into a Vec 
            let mut weights_vec: Vec<ByteWeights> = Vec::with_capacity(thread_num);
            for handle in handles{
                weights_vec.push(handle.join().unwrap());
            }

            // add all ByteWeights into one
            let mut weights = weights_vec.pop().unwrap();
            for weights_other in weights_vec{
                weights += weights_other;
            }

            weights
        }

        /// Return a reference to the weight corresponding
        /// to the given byte.
        pub fn get(&self, byte: &u8) -> Option<&usize>{
            let weight = self.weights.get(*byte as usize)?;
            if *weight == 0{
                return None
            }
            Some(weight)
        }

        /// Return a mutable reference to the weight corresponding
        /// to the given byte.
        pub fn get_mut(&mut self, byte: &u8) -> Option<&mut usize>{
            let weight = self.weights.get_mut(*byte as usize)?;
            if *weight == 0{
                return None
            }
            Some(weight)
        }

        /// Return the number of different counted bytes stored in the `ByteWeights`
        pub fn len(&self) -> usize{
            self.len
        }

        /// Return true if len == 0
        pub fn is_empty(&self) -> bool{
            self.len == 0
        }

        /// Returns an iterator over the bytes to their weights `(u8, usize)`
        pub fn iter(&self) -> Iter{
            self.into_iter()
        }

        /// Add another `ByteWeights` to self, like so:
        /// * if a byte is present in self & other, add their weights
        /// * if a byte is present in other, but not in self, add it to self with other's weight
        /// 
        /// # Example
        /// –––
        /// ```
        /// use huff_coding::prelude::ByteWeights;
        /// 
        /// let mut byte_weights = ByteWeights::from_bytes(b"aabbb");
        /// let other = ByteWeights::from_bytes(b"aaabbc");
        /// 
        /// byte_weights.add_byte_weights(&other);
        /// 
        /// assert_eq!(*byte_weights.get(&b'a').unwrap(), 5);
        /// assert_eq!(*byte_weights.get(&b'b').unwrap(), 5);
        /// assert_eq!(*byte_weights.get(&b'c').unwrap(), 1);
        /// ```
        pub fn add_byte_weights(&mut self, other: &ByteWeights){
            for (b, f) in other{
                let self_entry = self.get_mut(&b);
                match self_entry{
                    Some(self_entry) =>{
                        *self_entry += f;
                    }
                    None =>{
                        self.weights[b as usize] = f;
                        self.len += 1;
                    }
                }
            }
        }
    }

    /// Consuming iterator over the contents (`(u8, usize)`) of `ByteWeights`
    pub struct IntoIter{
        weights: ByteWeights,
        current_index: usize,
    }
    
    impl Iterator for IntoIter{
        type Item = (u8, usize);

        fn next(&mut self) -> Option<Self::Item>{
            if self.current_index == 256{
                return None
            }

            while self.weights.get(&(self.current_index as u8)).is_none(){
                if self.current_index == 256{
                    return None
                }
                self.current_index += 1
            }
            let entry = Some((self.current_index as u8, *self.weights.get(&(self.current_index as u8)).unwrap()));
            if self.current_index != 256{self.current_index += 1;}

            entry
        }
    }

    /// Non consuming iterator over the contents (`(u8, usize)`) of `ByteWeights`
    pub struct Iter<'a>{
            weights: &'a ByteWeights,
            current_index: usize,
    }

    impl Iterator for Iter<'_>{
            type Item = (u8, usize);
    
            fn next(&mut self) -> Option<Self::Item>{
                if self.current_index == 256{
                    return None
                }
    
                while self.weights.get(&(self.current_index as u8)).is_none(){
                    if self.current_index == 256{
                        return None
                    }
                    self.current_index += 1
                }
                let entry = Some((self.current_index as u8, *self.weights.get(&(self.current_index as u8)).unwrap()));
                if self.current_index != 256{self.current_index += 1;}
    
                entry
            }
    }
}
