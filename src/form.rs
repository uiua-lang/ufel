use std::{
    borrow::Cow,
    fmt,
    ops::{Deref, Index, IndexMut, Not},
};

use tinyvec::{tiny_vec, TinyVec};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Form {
    vert: usize,
    hori: usize,
    dims: FormDims,
}

pub type FormDims = TinyVec<[usize; 3]>;

impl Form {
    pub fn new(vert: usize, hori: usize, dims: FormDims) -> Self {
        let form = Self { vert, hori, dims };
        form.validate();
        form
    }
    pub fn scalar() -> Self {
        Self {
            vert: 0,
            hori: 0,
            dims: TinyVec::new(),
        }
    }
    pub fn empty_list() -> Self {
        Self {
            vert: 1,
            hori: 1,
            dims: tiny_vec![0],
        }
    }
    pub fn elems(&self) -> usize {
        self.dims.iter().product()
    }
    pub fn row_count(&self, ori: Ori) -> usize {
        match ori {
            Ori::Hori => self.hori_axis_rows().filter_map(|r| r.first()).product(),
            Ori::Vert => self
                .hori_axis_rows()
                .next()
                .map(|r| r.iter().product())
                .unwrap_or(1),
        }
    }
    pub fn row_len(&self, ori: Ori) -> usize {
        match ori {
            Ori::Hori => self
                .hori_axis_rows()
                .flat_map(|r| r.iter().skip(1))
                .product(),
            Ori::Vert => self.hori_axis_rows().skip(1).flatten().product(),
        }
    }
    pub fn shape(&self, ori: Ori) -> Shape {
        match ori {
            Ori::Hori => Shape(self.hori_axis_rows().next().unwrap_or(&[]).into()),
            Ori::Vert => Shape(
                self.hori_axis_rows()
                    .flat_map(|r| r.first())
                    .copied()
                    .collect::<Vec<_>>()
                    .into(),
            ),
        }
    }
    pub fn is_scalar(&self) -> bool {
        self.vert == 0 || self.hori == 0
    }
    pub fn is_list(&self) -> bool {
        self.normal_rank() == Some(1)
    }
    pub fn rank(&self, ori: Ori) -> usize {
        match ori {
            Ori::Hori => self.hori_rank(),
            Ori::Vert => self.vert_rank(),
        }
    }
    pub fn vert_rank(&self) -> usize {
        self.vert
    }
    pub fn hori_rank(&self) -> usize {
        self.hori
    }
    /// The total number of axes
    pub fn dims_rank(&self) -> usize {
        self.vert * self.hori
    }
    pub fn dims(&self) -> &[usize] {
        &self.dims
    }
    pub fn is_normal(&self) -> bool {
        self.vert <= 1 || self.hori == 0
    }
    pub fn as_normal(&self) -> Option<&[usize]> {
        if self.is_scalar() {
            Some(&[])
        } else {
            self.hori_axis_rows().next().filter(|_| self.is_normal())
        }
    }
    pub fn normal_rank(&self) -> Option<usize> {
        self.is_normal().then(|| self.hori_rank())
    }
    pub fn row(&self, ori: Ori) -> Self {
        let (vert, hori, dims) = match ori {
            Ori::Hori => {
                let vert = self.vert;
                let hori = self.hori.saturating_sub(1);
                let mut dims = TinyVec::with_capacity(vert * hori);
                for i in 0..vert {
                    for j in 0..hori {
                        dims.push(self.dims[i * self.hori + (j + 1)]);
                    }
                }
                (vert, hori, dims)
            }
            Ori::Vert => {
                let vert = self.vert.saturating_sub(1);
                let hori = self.hori;
                let mut dims = TinyVec::with_capacity(vert * hori);
                for i in 0..vert {
                    for j in 0..hori {
                        dims.push(self.dims[(i + 1) * self.hori + j]);
                    }
                }
                (vert, hori, dims)
            }
        };
        let form = Self { vert, hori, dims };
        form.validate();
        form
    }
    pub fn hori_axis_rows(&self) -> impl Iterator<Item = &[usize]> {
        self.dims.chunks_exact(self.hori.max(1))
    }
    pub fn is_prefix_of(&self, other: &Self) -> bool {
        if !(self.vert <= other.vert && self.hori <= other.hori) {
            return false;
        }
        for i in 0..self.vert {
            for j in 0..self.hori {
                if self[i][j] != other[i][j] {
                    return false;
                }
            }
        }
        true
    }
    pub fn prefixes_match(&self, other: &Self) -> bool {
        self.is_prefix_of(other) || other.is_prefix_of(self)
    }
    pub fn fix(&mut self, ori: Ori) {
        let new_vert = (self.vert + (ori == Ori::Vert) as usize).max(1);
        let new_hori = (self.hori + (ori == Ori::Hori) as usize).max(1);
        let mut dims = TinyVec::with_capacity(new_vert * new_hori);
        if self.is_scalar() {
            dims.push(1);
        } else {
            match ori {
                Ori::Hori => {
                    for i in 0..self.vert {
                        dims.push(1);
                        for j in 0..self.hori {
                            dims.push(self[i][j]);
                        }
                    }
                }
                Ori::Vert => {
                    for i in 0..self.vert {
                        for _ in 0..self.hori {
                            dims.push(1);
                        }
                        for j in 0..self.hori {
                            dims.push(self[i][j]);
                        }
                    }
                }
            }
        }
        self.dims = dims;
        self.vert = new_vert;
        self.hori = new_hori;
        self.validate();
    }
    pub fn deform(&mut self, ori: Ori) {
        match ori {
            Ori::Hori => {
                self.hori *= self.vert;
                self.vert = 1;
            }
            Ori::Vert => {
                self.vert *= self.hori;
                self.hori = 1;
            }
        }
        self.validate();
    }
    pub fn rerank(&mut self, _rank: usize) {
        todo!()
    }
    #[track_caller]
    pub(crate) fn validate(&self) {
        #[cfg(debug_assertions)]
        assert_eq!(
            self.dims_rank(),
            self.dims.len(),
            "Form is {}x{} but has {} elements",
            self.vert,
            self.hori,
            self.dims.len()
        );
    }
}

impl Index<usize> for Form {
    type Output = [usize];
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.vert,
            "Index {index} out of bounds of {} form rows",
            self.vert
        );
        &self.dims[index * self.hori..(index + 1) * self.hori]
    }
}

impl IndexMut<usize> for Form {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(
            index < self.vert,
            "Index {index} out of bounds of {} form rows",
            self.vert
        );
        &mut self.dims[index * self.hori..(index + 1) * self.hori]
    }
}

impl From<usize> for Form {
    fn from(n: usize) -> Self {
        Form::from([n])
    }
}

impl<const N: usize> From<[usize; N]> for Form {
    fn from(dims: [usize; N]) -> Self {
        Self {
            vert: 1,
            hori: N,
            dims: dims.into_iter().collect(),
        }
    }
}

impl From<&[usize]> for Form {
    fn from(dims: &[usize]) -> Self {
        Self {
            vert: 1,
            hori: dims.len(),
            dims: dims.iter().copied().collect(),
        }
    }
}

impl<const M: usize, const N: usize> From<[[usize; N]; M]> for Form {
    fn from(dims: [[usize; N]; M]) -> Self {
        Self {
            vert: M,
            hori: N,
            dims: dims.into_iter().flatten().collect(),
        }
    }
}

impl From<FormDims> for Form {
    fn from(dims: FormDims) -> Self {
        Self {
            vert: 1,
            hori: dims.len(),
            dims,
        }
    }
}

impl FromIterator<usize> for Form {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let dims: FormDims = iter.into_iter().collect();
        Self::from(dims)
    }
}

impl fmt::Debug for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        if let Some(dims) = self.as_normal() {
            for (i, dim) in dims.iter().enumerate() {
                if i > 0 {
                    write!(f, "×")?;
                }
                write!(f, "{}", dim)?;
            }
        } else {
            for i in 0..self.vert {
                if i > 0 {
                    write!(f, " ")?;
                }
                if self.hori == 0 {
                    write!(f, "×")?;
                }
                for j in 0..self.hori {
                    if j > 0 || self.hori == 1 {
                        write!(f, "×")?;
                    }
                    write!(f, "{}", self[i][j])?;
                }
            }
        }
        write!(f, "]")
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Shape<'a>(Cow<'a, [usize]>);

impl<'a> fmt::Debug for Shape<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, dim) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "×")?;
            }
            write!(f, "{dim}")?;
        }
        write!(f, "]")
    }
}

impl<'a> Deref for Shape<'a> {
    type Target = [usize];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ori {
    #[default]
    Hori,
    Vert,
}

impl Ori {
    pub fn str(&self) -> &'static str {
        match self {
            Ori::Hori => "horizontal",
            Ori::Vert => "vertical",
        }
    }
}

impl Not for Ori {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Ori::Hori => Self::Vert,
            Ori::Vert => Self::Hori,
        }
    }
}
