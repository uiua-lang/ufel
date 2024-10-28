use std::fmt;

use crate::{DyMod, Dyadic, Mod, Monadic, Sp, Span};

#[derive(Debug, Clone)]
pub enum Item {
    Words(Vec<Sp<Word>>),
}

#[derive(Clone)]
pub enum Word {
    Number(f64),
    Func(Func),
    Array(Array),
    Mon(Monadic),
    Dy(Dyadic),
    Mod(Modified),
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Word::Number(n) => n.fmt(f),
            Word::Func(fu) => fu.fmt(f),
            Word::Array(a) => a.fmt(f),
            Word::Mon(m) => m.fmt(f),
            Word::Dy(d) => d.fmt(f),
            Word::Mod(m) => m.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Func {
    pub open: Span,
    pub lines: Vec<Vec<Sp<Word>>>,
    pub close: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Array {
    pub open: Span,
    pub lines: Vec<Vec<Sp<Word>>>,
    pub close: Span,
}

#[derive(Debug, Clone)]
pub struct Modified {
    pub modifier: Sp<Modifier>,
    pub args: Vec<Sp<Word>>,
    pub pack: bool,
}

#[derive(Clone)]
pub enum Modifier {
    Mon(Mod),
    Dy(DyMod),
}

impl Modifier {
    pub fn arg_count(&self) -> usize {
        match self {
            Self::Mon(_) => 1,
            Self::Dy(_) => 2,
        }
    }
}

impl fmt::Debug for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Mon(m) => m.fmt(f),
            Modifier::Dy(d) => d.fmt(f),
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Mon(m) => m.fmt(f),
            Modifier::Dy(d) => d.fmt(f),
        }
    }
}
