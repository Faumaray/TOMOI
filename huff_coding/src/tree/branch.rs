use crate::bitvec::prelude::{BitVec, Msb0};
use super::{
    leaf::HuffLeaf, 
    letter::HuffLetter,
};

use std::cmp::Ordering;

/// Struct representing a branch in the [`HuffTree`][tree] struct. 
/// It contains data stored in a [`HuffLeaf`][leaf] (letter, weight and code) and 
/// optionally two child [`HuffBranch`es][branch] (left and right)
///
/// Examples 
/// ---
/// Initializing a branch "by hand":
/// ```
/// use huff_coding::prelude::{HuffBranch, HuffLeaf};
/// use std::cell::RefCell;
/// 
/// let foo = HuffBranch::new(
///     HuffLeaf::new(
///         None,
///         5
///     ),
///     Some((
///         HuffBranch::new(
///             HuffLeaf::new(
///                 Some(0xbb),
///                 2
///             ),
///             None
///         ),
///         HuffBranch::new(
///             HuffLeaf::new(
///                 Some(0xcc),
///                 3
///             ),
///             None
///         ),
///     ))
/// );
/// ```
/// Iterating over the children of a [`HuffBranch`][branch]:
/// ```
/// use huff_coding::prelude::{HuffBranch, HuffLeaf};
/// use std::cell::RefCell;
/// 
/// let foo = HuffBranch::new(
///     HuffLeaf::new(
///         None,
///         9
///     ),
///     Some((
///         HuffBranch::new(
///             HuffLeaf::new(
///                 Some(-423),
///                 7
///             ),
///             None
///         ),
///         HuffBranch::new(
///             HuffLeaf::new(
///                 Some(8),
///                 2
///             ),
///             None
///         ),
///     ))
/// );
/// 
/// let mut children_iter = foo.children_iter().unwrap();
/// 
/// assert_eq!(
///     children_iter.next().unwrap().leaf().letter(),
///     Some(&-423)
/// );
/// 
/// assert_eq!(
///     children_iter.next().unwrap().leaf().letter(),
///     Some(&8)
/// );
/// 
/// assert_eq!(
///     children_iter.next(),
///     None
/// );
/// 
/// ```
/// Comparing different branches:
/// ```
/// use huff_coding::prelude::{HuffBranch, HuffLeaf};
/// 
/// let foo = HuffBranch::new(
///     HuffLeaf::new(
///         Some('t'),
///         2137
///     ),
///     None
/// );
/// 
///  let bar = HuffBranch::new(
///     HuffLeaf::new(
///         Some('x'),
///         144
///     ),
///     None
/// );
/// 
/// let foobar = HuffBranch::new(
///     HuffLeaf::new(
///         Some('t'),
///         144
///     ),
///     None
/// );
/// 
/// assert!(foo > bar);
/// assert!(bar == foobar);
/// assert!(foobar != foo);
/// ```
/// Setting the children after initialization:
/// ```
/// use huff_coding::prelude::{HuffBranch, HuffLeaf};
/// use std::cell::RefCell;
/// 
/// let mut foo = HuffBranch::new(
///     HuffLeaf::new(
///         Some("yś"),
///         12
///     ),
///     None
/// );
/// 
/// foo.set_children(
///     Some((
///         HuffBranch::new(
///             HuffLeaf::new(
///                 Some("yś"),
///                 2
///             ),
///             None
///         ),
///         HuffBranch::new(
///             HuffLeaf::new(
///                 Some("omamale"),
///                 10
///             ),
///             None
///         )
///     ))
/// );
/// ```
/// 
/// [leaf]:crate::tree::leaf::HuffLeaf
/// [tree]:crate::tree::HuffTree
/// [branch]:HuffBranch
#[derive(Debug, Clone, Eq)]
pub struct HuffBranch<L: HuffLetter>{
    leaf: HuffLeaf<L>,
    left_child: Option<Box<HuffBranch<L>>>,
    right_child: Option<Box<HuffBranch<L>>>,
}

impl<L: HuffLetter> Ord for HuffBranch<L>{
    fn cmp(&self, other: &Self) -> Ordering {
        self.leaf().cmp(other.leaf())
    }
}

impl<L: HuffLetter> PartialOrd for HuffBranch<L>{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<L: HuffLetter> PartialEq for HuffBranch<L>{
    fn eq(&self, other: &Self) -> bool {
        self.leaf() == other.leaf()
    }
}

impl<L: HuffLetter> HuffBranch<L>{
    /// Initialize a new `HuffBranch<L>` with the given leaf and children
    /// 
    /// In the provided tuple:
    /// * 0 means left_child
    /// * 1 means right_child
    pub fn new(leaf: HuffLeaf<L>, children: Option<(HuffBranch<L>, HuffBranch<L>)>) -> Self{
        if let Some(children) = children{
            HuffBranch{
                leaf,
                left_child: Some(Box::new(children.0)),
                right_child: Some(Box::new(children.1)),
            }
        }
        else{
            HuffBranch{
                leaf,
                left_child: None,
                right_child: None,
            }
        }
    }

    /// Return a reference to the [`HuffLeaf`][crate::tree::leaf::HuffLeaf] containing the branch's
    /// letter, weight and code.
    pub fn leaf(&self) -> &HuffLeaf<L>{
        &self.leaf
    }

    /// Return an iterator over the branch's children (`&HuffBranch<L>`])
    /// or [`None`][None] if it has no children
    /// 
    /// # Example
    /// ---
    /// ```
    /// use huff_coding::prelude::{HuffBranch, HuffLeaf};
    /// 
    /// let branch = HuffBranch::new(
    ///     HuffLeaf::new(None, 7),
    ///     Some((
    ///         HuffBranch::new(
    ///             HuffLeaf::new(Some(5), 3), 
    ///             None,
    ///         ),
    ///         HuffBranch::new(
    ///             HuffLeaf::new(Some(42), 4), 
    ///             None,
    ///         )
    ///     ))
    /// );
    /// 
    /// let mut children_iter = branch.children_iter().unwrap();
    /// assert_eq!(
    ///     children_iter.next().unwrap()
    ///         .leaf()
    ///         .letter(),
    ///     Some(&5)
    /// );
    /// assert_eq!(
    ///     children_iter.next().unwrap()
    ///         .leaf()
    ///         .letter(),
    ///     Some(&42)
    /// );
    /// ```
    pub fn children_iter(&self) -> Option<ChildrenIter<L>>{
        if self.has_children(){Some(ChildrenIter::new(self))}
        else{None}
    }

    /// Return a reference to the left child of the branch `HuffBranch<L>`, or 
    /// [`None`][None] if it has no children
    pub fn left_child(&self) -> Option<&HuffBranch<L>>{
        self.left_child.as_deref()
    }

    /// Return a mutable reference to the left child of the branch `HuffBranch<L>`, or 
    /// [`None`][None] if it has no children
    pub fn left_child_mut(&mut self) -> Option<&mut HuffBranch<L>>{
        self.left_child.as_deref_mut()
    }

    /// Return a reference to the right child of the branch `HuffBranch<L>`, or 
    /// [`None`][None] if it has no children
    pub fn right_child(&self) -> Option<&HuffBranch<L>>{
        self.right_child.as_deref()
    }

    /// Return a mutable reference to the right child of the branch `HuffBranch<L>`, or 
    /// [`None`][None] if it has no children
    pub fn right_child_mut(&mut self) -> Option<&mut HuffBranch<L>>{
        self.right_child.as_deref_mut()
    }

    /// Return true if the branch has children
    pub fn has_children(&self) -> bool{
        self.left_child.is_some()
    }

    /// Setter for the branch's children
    /// 
    /// In the provided tuple:
    /// * 0 means left_child
    /// * 1 means right_child
    pub fn set_children(&mut self, children: Option<(HuffBranch<L>, HuffBranch<L>)>){
        if let Some(children) = children{
            self.left_child = Some(Box::new(children.0));
            self.right_child = Some(Box::new(children.1));  
        }
        else{
            self.left_child = None;
            self.right_child = None;
        }
    }

    /// Setter for the leaf's code 
    pub fn set_code(&mut self, code: BitVec<Msb0, u8>){
        self.leaf.set_code(code);
    }
}

/// An iterator over a [`HuffBranch`'s][HuffBranch] children
pub struct ChildrenIter<'a, L: HuffLetter>{
    parent: &'a HuffBranch<L>,
    child_pos: u8,
}

impl<'a, L: HuffLetter> Iterator for ChildrenIter<'a, L>{
    type Item = &'a HuffBranch<L>;

    fn next(&mut self) -> Option<Self::Item>{
        match self.child_pos{
            0 =>{
                self.child_pos += 1;
                self.parent.left_child()
            }
            1 =>{
                self.child_pos += 1;
                self.parent.right_child()
            }
            _ => 
                None,
        }
    }
}

impl<'a, L: HuffLetter> ChildrenIter<'a, L>{
    /// Initialize a new ```ChildrenIter``` over 
    /// the children of the provided ```HuffBranch```
    pub fn new(parent: &'a HuffBranch<L>) -> Self{
        ChildrenIter{
            parent,
            child_pos: 0,
        }
    }
}
