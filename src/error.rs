#[derive(Debug, Clone)]
pub struct UfelError {
    pub kind: UfelErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UfelErrorKind {}
