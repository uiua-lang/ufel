use std::{
    fmt,
    ops::{Index, IndexMut},
};

use tinyvec::{tiny_vec, TinyVec};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Form {
    rows: usize,
    cols: usize,
    dims: TinyVec<[usize; 3]>,
}

impl Form {
    pub fn scalar() -> Self {
        Self {
            rows: 0,
            cols: 0,
            dims: TinyVec::new(),
        }
    }
    pub fn empty_list() -> Self {
        Self {
            rows: 1,
            cols: 1,
            dims: tiny_vec![0],
        }
    }
    pub fn elems(&self) -> usize {
        self.dims.iter().product()
    }
    pub fn row_count(&self) -> usize {
        self.axis_rows().filter_map(|r| r.first()).product()
    }
    pub fn col_count(&self) -> usize {
        self.axis_rows()
            .next()
            .map(|r| r.iter().product())
            .unwrap_or(1)
    }
    pub fn row_len(&self) -> usize {
        self.axis_rows().flat_map(|r| r.iter().skip(1)).product()
    }
    pub fn col_len(&self) -> usize {
        self.axis_rows().skip(1).flatten().product()
    }
    pub fn is_scalar(&self) -> bool {
        self.rows == 0 && self.cols == 0
    }
    pub fn is_list(&self) -> bool {
        self.normal_rank() == Some(1)
    }
    pub fn row_rank(&self) -> usize {
        self.rows
    }
    pub fn col_rank(&self) -> usize {
        self.cols
    }
    pub fn axis_rank(&self) -> usize {
        self.rows * self.cols
    }
    pub fn is_normal(&self) -> bool {
        self.rows <= 1
    }
    pub fn as_normal(&self) -> Option<&[usize]> {
        self.axis_rows().next().filter(|_| self.is_normal())
    }
    pub fn normal_rank(&self) -> Option<usize> {
        self.is_normal().then(|| self.col_rank())
    }
    pub fn row(&self) -> Self {
        let rows = self.rows;
        let cols = self.cols.saturating_sub(1);
        let mut dims = TinyVec::with_capacity(rows * cols);
        for i in 0..rows {
            for j in 1..cols {
                dims.push(self.dims[i * self.cols + j]);
            }
        }
        let form = Self { rows, cols, dims };
        form.validate();
        form
    }
    pub fn col(&self) -> Self {
        let rows = self.rows.saturating_sub(1);
        let cols = self.cols;
        let mut dims = TinyVec::with_capacity(rows * cols);
        for i in 1..rows {
            for j in 0..cols {
                dims.push(self.dims[i * self.cols + j]);
            }
        }
        let form = Self { rows, cols, dims };
        form.validate();
        form
    }
    pub fn axis_rows(&self) -> impl Iterator<Item = &[usize]> {
        self.dims.chunks_exact(self.cols.max(1))
    }
    pub fn is_prefix_of(&self, other: &Self) -> bool {
        if !(self.rows <= other.rows && self.cols <= other.cols) {
            return false;
        }
        for i in 0..self.rows {
            for j in 0..self.cols {
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
    pub fn fix(&mut self) {
        if self.is_normal() {
            self.dims.insert(0, 1);
        } else {
            let mut dims = TinyVec::with_capacity(self.rows * (self.cols + 1));
            for i in 0..self.rows {
                dims.push(1);
                for j in 0..self.cols {
                    dims.push(self[i][j]);
                }
            }
            self.dims = dims;
        }
        self.cols += 1;
        self.rows = self.rows.max(1);
        self.validate();
    }
    pub(crate) fn validate(&self) {
        #[cfg(debug_assertions)]
        assert_eq!(
            self.elems(),
            self.dims.len(),
            "Form is {}x{} but has {} elements",
            self.rows,
            self.cols,
            self.dims.len()
        );
    }
}

impl Index<usize> for Form {
    type Output = [usize];
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.rows,
            "Index {index} out of bounds of {} form rows",
            self.rows
        );
        &self.dims[index * self.cols..(index + 1) * self.cols]
    }
}

impl IndexMut<usize> for Form {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(
            index < self.rows,
            "Index {index} out of bounds of {} form rows",
            self.rows
        );
        &mut self.dims[index * self.cols..(index + 1) * self.cols]
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
            rows: 1,
            cols: N,
            dims: dims.into_iter().collect(),
        }
    }
}

impl From<&[usize]> for Form {
    fn from(dims: &[usize]) -> Self {
        Self {
            rows: 1,
            cols: dims.len(),
            dims: dims.iter().copied().collect(),
        }
    }
}

impl<const M: usize, const N: usize> From<[[usize; N]; M]> for Form {
    fn from(dims: [[usize; N]; M]) -> Self {
        Self {
            rows: M,
            cols: N,
            dims: dims.into_iter().flatten().collect(),
        }
    }
}

impl fmt::Debug for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..self.rows {
            if i > 0 {
                write!(f, " ")?;
            }
            if self.cols == 0 {
                write!(f, "_")?;
            }
            for j in 0..self.cols {
                if j > 0 || self.cols == 1 {
                    write!(f, "_")?;
                }
                write!(f, "{}", self[i][j])?;
            }
        }
        write!(f, "]")
    }
}
