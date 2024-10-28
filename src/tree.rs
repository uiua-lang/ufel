use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    mem::{discriminant, swap, take},
    ops::Deref,
    slice::{self, SliceIndex},
};

use ecow::{eco_vec, EcoVec};

use crate::{Array, DyMod, Dyadic, Mod, Monadic};

node!(
    Run(nodes(EcoVec<Node>)),
    Push(val(Array)),
    Array(len(usize), inner(Box<Node>), span(usize)),
    Mon(prim(Monadic), span(usize)),
    Dy(prim(Dyadic), span(usize)),
    Mod(prim(Mod), f(Box<SigNode>), span(usize)),
    DyMod(prim(DyMod), f(Box<SigNode>), g(Box<SigNode>), span(usize)),
);

/// A node with a signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SigNode {
    /// The node
    pub node: Node,
    /// The signature
    pub sig: Signature,
}

impl SigNode {
    /// Create a new signature node
    pub fn new(node: impl Into<Node>, sig: impl Into<Signature>) -> Self {
        Self {
            node: node.into(),
            sig: sig.into(),
        }
    }
}

impl From<SigNode> for Node {
    fn from(sn: SigNode) -> Self {
        sn.node
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::empty()
    }
}

impl Node {
    /// Create an empty node
    pub const fn empty() -> Self {
        Self::Run(EcoVec::new())
    }
    /// Create a push node from a value
    pub fn new_push(val: impl Into<Array>) -> Self {
        Self::Push(val.into())
    }
    /// Get a slice of the nodes in this node
    pub fn as_slice(&self) -> &[Node] {
        if let Node::Run(nodes) = self {
            nodes
        } else {
            slice::from_ref(self)
        }
    }
    /// Get a mutable slice of the nodes in this node
    ///
    /// Transforms the node into a [`Node::Run`] if it is not already a [`Node::Run`]
    pub fn as_mut_slice(&mut self) -> &mut [Node] {
        match self {
            Node::Run(nodes) => nodes.make_mut(),
            other => {
                let first = take(other);
                let Node::Run(nodes) = other else {
                    unreachable!()
                };
                nodes.push(first);
                nodes.make_mut()
            }
        }
    }
    /// Slice the node to get a subnode
    pub fn slice<R>(&self, range: R) -> Self
    where
        R: SliceIndex<[Node], Output = [Node]>,
    {
        Self::from_iter(self.as_slice()[range].iter().cloned())
    }
    /// Get a mutable vector of the nodes in this node
    ///
    /// Transforms the node into a [`Node::Run`] if it is not already a [`Node::Run`]
    pub fn as_vec(&mut self) -> &mut EcoVec<Node> {
        match self {
            Node::Run(nodes) => nodes,
            other => {
                let first = take(other);
                let Node::Run(nodes) = other else {
                    unreachable!()
                };
                nodes.push(first);
                nodes
            }
        }
    }
    /// Turn the node into a vector
    pub fn into_vec(self) -> EcoVec<Node> {
        if let Node::Run(nodes) = self {
            nodes
        } else {
            eco_vec![self]
        }
    }
    /// Truncate the node to a certain length
    pub fn truncate(&mut self, len: usize) {
        if let Node::Run(nodes) = self {
            nodes.truncate(len);
            if nodes.len() == 1 {
                *self = take(nodes).remove(0);
            }
        } else if len == 0 {
            *self = Node::default();
        }
    }
    /// Split the node at the given index
    #[track_caller]
    pub fn split_off(&mut self, index: usize) -> Self {
        if let Node::Run(nodes) = self {
            let removed = EcoVec::from(&nodes[index..]);
            nodes.truncate(index);
            Node::Run(removed)
        } else if index == 0 {
            take(self)
        } else if index == 1 {
            Node::empty()
        } else {
            panic!(
                "Index {index} out of bounds of node with length {}",
                self.len()
            );
        }
    }
    /// Mutably iterate over the nodes of this node
    ///
    /// Transforms the node into a [`Node::Run`] if it is not already a [`Node::Run`]
    pub fn iter_mut(&mut self) -> slice::IterMut<Self> {
        self.as_mut_slice().iter_mut()
    }
    /// Push a node onto the end of the node
    ///
    /// Transforms the node into a [`Node::Run`] if it is not already a [`Node::Run`]
    pub fn push(&mut self, mut node: Node) {
        if let Node::Run(nodes) = self {
            if nodes.is_empty() {
                *self = node;
            } else {
                match node {
                    Node::Run(other) => nodes.extend(other),
                    node => nodes.push(node),
                }
            }
        } else if let Node::Run(nodes) = &node {
            if !nodes.is_empty() {
                swap(self, &mut node);
                self.as_vec().insert(0, node);
            }
        } else {
            self.as_vec().push(node);
        }
    }
    /// Push a node onto the beginning of the node
    ///
    /// Transforms the node into a [`Node::Run`] if it is not already a [`Node::Run`]
    pub fn prepend(&mut self, mut node: Node) {
        if let Node::Run(nodes) = self {
            if nodes.is_empty() {
                *self = node;
            } else {
                match node {
                    Node::Run(mut other) => {
                        swap(nodes, &mut other);
                        nodes.extend(other)
                    }
                    node => nodes.insert(0, node),
                }
            }
        } else if let Node::Run(nodes) = &node {
            if !nodes.is_empty() {
                swap(self, &mut node);
                self.as_vec().push(node);
            }
        } else {
            self.as_vec().insert(0, node);
        }
    }
    /// Pop a node from the end of this node
    pub fn pop(&mut self) -> Option<Node> {
        match self {
            Node::Run(nodes) => {
                let res = nodes.pop();
                if nodes.len() == 1 {
                    *self = take(nodes).remove(0);
                }
                res
            }
            node => Some(take(node)),
        }
    }
}

impl From<&[Node]> for Node {
    fn from(nodes: &[Node]) -> Self {
        Node::from_iter(nodes.iter().cloned())
    }
}

impl FromIterator<Node> for Node {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let Some(mut node) = iter.next() else {
            return Node::default();
        };
        for n in iter {
            node.push(n);
        }
        node
    }
}

impl Extend<Node> for Node {
    fn extend<T: IntoIterator<Item = Node>>(&mut self, iter: T) {
        for node in iter {
            self.push(node);
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Run(eco_vec) => {
                let mut tuple = f.debug_tuple("");
                for node in eco_vec {
                    tuple.field(node);
                }
                tuple.finish()
            }
            Node::Push(array) => write!(f, "push {array:?}"),
            Node::Array(_, inner, _) => {
                write!(f, "[")?;
                inner.fmt(f)?;
                write!(f, "]")
            }
            Node::Mon(prim, _) => write!(f, "{prim}"),
            Node::Dy(prim, _) => write!(f, "{prim}"),
            Node::Mod(prim, ff, _) => f.debug_tuple(&prim.to_string()).field(&ff.node).finish(),
            Node::DyMod(prim, ff, g, _) => f
                .debug_tuple(&prim.to_string())
                .field(&ff.node)
                .field(&g.node)
                .finish(),
        }
    }
}

impl Node {
    pub fn sig_node(self) -> SigNode {
        let sig = self.sig();
        SigNode::new(self, sig)
    }
    pub fn sig(&self) -> Signature {
        #[derive(Default)]
        struct Checker {
            min_height: i32,
            height: i32,
        }
        impl Checker {
            fn handle_args_outputs(&mut self, args: usize, outputs: usize) {
                self.height -= args as i32;
                self.min_height = self.min_height.min(self.height);
                self.height += outputs as i32;
            }
            fn sig(&self) -> Signature {
                Signature {
                    args: (-self.min_height) as usize,
                    outputs: (self.height - self.min_height) as usize,
                }
            }
            fn node(&mut self, node: &Node) {
                match node {
                    Node::Run(nodes) => nodes.iter().for_each(|node| self.node(node)),
                    Node::Array(len, inner, _) => {
                        self.node(inner);
                        self.handle_args_outputs(*len, 1);
                    }
                    Node::Push(_) => self.handle_args_outputs(0, 1),
                    Node::Mon(_, _) => self.handle_args_outputs(1, 1),
                    Node::Dy(_, _) => self.handle_args_outputs(2, 1),
                    Node::Mod(m, f, _) => match m {
                        Mod::Dip => self.handle_args_outputs(f.sig.args + 1, f.sig.outputs + 1),
                        Mod::Reduce | Mod::Scan => self.handle_args_outputs(1, 1),
                        Mod::Slf => {
                            self.handle_args_outputs(1, 2);
                            self.handle_args_outputs(f.sig.args, f.sig.outputs)
                        }
                        Mod::Flip => {
                            self.handle_args_outputs(2, 2);
                            self.handle_args_outputs(f.sig.args, f.sig.outputs)
                        }
                        Mod::On | Mod::By => {
                            self.handle_args_outputs(f.sig.args.max(1), f.sig.outputs + 1)
                        }
                        Mod::Both => self.handle_args_outputs(f.sig.args * 2, f.sig.outputs * 2),
                    },
                    Node::DyMod(m, f, g, _) => match m {
                        DyMod::Fork => self.handle_args_outputs(
                            f.sig.args.max(g.sig.args),
                            f.sig.outputs + g.sig.outputs,
                        ),
                        DyMod::Bracket => self.handle_args_outputs(
                            f.sig.args + g.sig.args,
                            f.sig.outputs + g.sig.outputs,
                        ),
                    },
                }
            }
        }
        let mut checker = Checker::default();
        checker.node(self);
        checker.sig()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Signature {
    /// The number of arguments the function pops off the stack
    pub args: usize,
    /// The number of values the function pushes onto the stack
    pub outputs: usize,
}

impl From<(usize, usize)> for Signature {
    fn from((args, outputs): (usize, usize)) -> Self {
        Self::new(args, outputs)
    }
}

impl From<Signature> for (usize, usize) {
    fn from(sig: Signature) -> Self {
        (sig.args, sig.outputs)
    }
}

impl Signature {
    /// Create a new signature with the given number of arguments and outputs
    pub const fn new(args: usize, outputs: usize) -> Self {
        Self { args, outputs }
    }
}

impl PartialEq<(usize, usize)> for Signature {
    fn eq(&self, other: &(usize, usize)) -> bool {
        self.args == other.0 && self.outputs == other.1
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|{}.{}", self.args, self.outputs)
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.outputs == 1 {
            write!(f, "|{}", self.args)
        } else {
            write!(f, "{self:?}")
        }
    }
}

impl Deref for Node {
    type Target = [Node];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a> IntoIterator for &'a Node {
    type Item = &'a Node;
    type IntoIter = slice::Iter<'a, Node>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a> IntoIterator for &'a mut Node {
    type Item = &'a mut Node;
    type IntoIter = slice::IterMut<'a, Node>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl IntoIterator for Node {
    type Item = Node;
    type IntoIter = ecow::vec::IntoIter<Node>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

macro_rules! node {
    ($(
        $(#[$attr:meta])*
        $((#[$rep_attr:meta] rep),)?
        $name:ident
        $(($($tup_name:ident($tup_type:ty)),* $(,)?))?
        $({$($field_name:ident : $field_type:ty),* $(,)?})?
    ),* $(,)?) => {
        /// A Uiua execution tree node
        ///
        /// A node is a tree structure of instructions. It can be used as both a single unit as well as a list.
        #[derive(Clone)]
        #[repr(u8)]
        #[allow(missing_docs)]
        pub enum Node {
            $(
                $(#[$attr])*
                $name $(($($tup_type),*))? $({$($field_name : $field_type),*})?,
            )*
        }

        macro_rules! field_span {
            (span, $sp:ident) => {
                return Some($sp)
            };
            ($sp:ident, $sp2:ident) => {};
        }

        impl Node {
            /// Get the span index of this instruction
            #[allow(unreachable_code, unused)]
            pub fn span(&self) -> Option<usize> {
                (|| match self {
                    $(
                        Self::$name $(($($tup_name),*))? $({$($field_name),*})? => {
                            $($(field_span!($tup_name, $tup_name);)*)*
                            $($(field_span!($field_name, $field_name);)*)*
                            return None;
                        },
                    )*
                })().copied()
            }
            /// Get a mutable reference to the span index of this instruction
            #[allow(unreachable_code, unused)]
            pub fn span_mut(&mut self) -> Option<&mut usize> {
                match self {
                    $(
                        Self::$name $(($($tup_name),*))? $({$($field_name),*})? => {
                            $($(field_span!($tup_name, $tup_name);)*)*
                            $($(field_span!($field_name, $field_name);)*)*
                            return None;
                        },
                    )*
                }
            }
        }

        impl PartialEq for Node {
            #[allow(unused_variables)]
            fn eq(&self, other: &Self) -> bool {
                let mut hasher = DefaultHasher::new();
                self.hash(&mut hasher);
                let hash = hasher.finish();
                let mut other_hasher = DefaultHasher::new();
                other.hash(&mut other_hasher);
                let other_hash = other_hasher.finish();
                hash == other_hash
            }
        }

        impl Eq for Node {}

        impl Hash for Node {
            #[allow(unused_variables)]
            fn hash<H: Hasher>(&self, state: &mut H) {
                macro_rules! hash_field {
                    (span) => {};
                    ($nm:ident) => {Hash::hash($nm, state)};
                }
                match self {
                    $(
                        Self::$name $(($($tup_name),*))? $({$($field_name),*})? => {
                            discriminant(self).hash(state);
                            $($(hash_field!($field_name);)*)?
                            $($(hash_field!($tup_name);)*)?
                        }
                    )*
                }
            }
        }
    };
}
use node;
