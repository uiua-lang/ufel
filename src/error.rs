use std::{error::Error, fmt, mem::take};

use ecow::{EcoString, EcoVec};

use crate::{HumanSp, HumanSpan, ParseError};

#[derive(Debug, Clone)]
pub struct UfelError {
    pub kind: UfelErrorKind,
    pub multi: EcoVec<Self>,
}

impl UfelError {
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<E>(errors: impl IntoIterator<Item = E>) -> Option<Self>
    where
        E: Into<Self>,
    {
        let mut errors = errors.into_iter().map(Into::into);
        let mut error = errors.next()?;
        error.multi.extend(errors);
        Some(error)
    }
}

pub type UfelResult<T = ()> = Result<T, UfelError>;

#[derive(Debug, Clone)]
pub enum UfelErrorKind {
    Parse(HumanSp<ParseError>),
    Compile(LocatedError),
    Run(LocatedError),
}

impl fmt::Display for UfelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            UfelErrorKind::Parse(e) => write!(f, "Parse error at {e}"),
            UfelErrorKind::Compile(e) => write!(f, "Compile error at {e}"),
            UfelErrorKind::Run(e) => write!(f, "Runtime error at {e}"),
        }
    }
}

impl Error for UfelError {}

impl From<HumanSp<ParseError>> for UfelError {
    fn from(e: HumanSp<ParseError>) -> Self {
        Self {
            kind: UfelErrorKind::Parse(e),
            multi: EcoVec::new(),
        }
    }
}

impl From<UfelErrorKind> for UfelError {
    fn from(kind: UfelErrorKind) -> Self {
        Self {
            kind,
            multi: EcoVec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedError {
    pub span: HumanSpan,
    pub message: EcoString,
    pub line: EcoString,
}

impl Error for LocatedError {}

impl fmt::Display for LocatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.span, self.message)
    }
}

impl IntoIterator for UfelError {
    type Item = Self;
    type IntoIter = ecow::vec::IntoIter<Self>;
    fn into_iter(mut self) -> Self::IntoIter {
        let mut multi = take(&mut self.multi);
        multi.insert(0, self);
        multi.into_iter()
    }
}
