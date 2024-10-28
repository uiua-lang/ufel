use std::ptr;

use ecow::{eco_vec, EcoVec};

use crate::{cowslice::CowSlice, Array, Element, Form, FormDims, Monadic, Ori, Ufel, UfelResult};

impl Array {
    pub fn range(self, rt: &Ufel) -> UfelResult<Self> {
        if !self.form.is_normal() {
            return Err(rt.error(format!(
                "{:?} array must be normal, but its form is {:?}",
                Monadic::Range,
                self.form
            )));
        }
        let shape = self.form.shape(rt.ori());
        if let Some(d) = self.data.iter().find(|&d| d.fract() != 0.0 || *d < 0.0) {
            return Err(rt.error(format!(
                "{:?} array must be naturals, but one element is {d}",
                Monadic::Range
            )));
        }
        let max: f64 = self.data.iter().product();
        if max > 1e8 {
            return Err(rt.error(format!("Array of {} elements would be too large", max)));
        }
        Ok(match *shape.as_ref() {
            [] => (0..max as usize).map(|i| i as f64).collect(),
            [n] => {
                let dims: Vec<usize> = self.data.iter().map(|d| *d as usize).collect();
                let form = Form::from_iter(dims.iter().copied().chain([n]));
                let mut data = eco_vec![0.0; form.elems()];
                let slice = data.make_mut();
                let mut index = vec![0; n];
                for i in 0..max as usize {
                    flat_to_dims(&dims, i, &mut index);
                    for (x, &y) in slice[i * n..(i + 1) * n].iter_mut().zip(&index) {
                        *x = y as f64;
                    }
                }
                Array::new(form, data.into())
            }
            [_n, _m] => return Err(rt.error("Full form range is not yet implemented")),
            _ => {
                return Err(rt.error(format!(
                    "{:?} array must be at most rank 2, but its form is {:?}",
                    Monadic::Range,
                    self.form
                )))
            }
        })
    }
}

impl<T: Element> Array<T> {
    pub fn first(self, rt: &Ufel) -> UfelResult<Self> {
        Ok(match rt.ori() {
            Ori::Hori => {
                if self.form.row_count(Ori::Hori) == 0 {
                    return Err(rt.error("Cannot get first row of an empty array"));
                }
                let row_len = self.form.row_len(Ori::Hori);
                let form = self.form.row(Ori::Hori);
                let data = self.data.slice(0..row_len);
                Array::new(form, data)
            }
            Ori::Vert => {
                if self.form.row_count(Ori::Vert) == 0 {
                    return Err(rt.error("Cannot get first row of an empty array"));
                }
                let stride = self.form.row_len(Ori::Hori);
                let row_len = self.form.row_len(Ori::Vert);
                let mut data = EcoVec::with_capacity(row_len);
                for i in 0..row_len {
                    data.push(self.data[i * stride].clone());
                }
                let form = self.form.row(Ori::Vert);
                Array::new(form, data.into())
            }
        })
    }
    pub fn reverse(mut self, ori: Ori) -> Self {
        let row_count = self.form.row_count(ori);
        let row_len = self.form.row_len(ori);
        dbg!(&row_count, &row_len, &self.form);
        let data = self.data.as_mut_slice();
        match ori {
            Ori::Hori => {
                for i in 0..row_count / 2 {
                    let left = i * row_len;
                    let right = (row_count - i - 1) * row_len;
                    let left = &mut data[left] as *mut T;
                    let right = &mut data[right] as *mut T;
                    unsafe {
                        ptr::swap_nonoverlapping(left, right, row_len);
                    }
                }
            }
            Ori::Vert => {
                for i in 0..row_count / 2 {
                    let left = i * row_len;
                    let right = (row_count - i - 1) * row_len;
                    let left = &mut data[left] as *mut T;
                    let right = &mut data[right] as *mut T;
                    unsafe {
                        ptr::swap_nonoverlapping(left, right, row_len);
                    }
                }
            }
        }
        self
    }
    pub fn transpose(self, rt: &Ufel) -> UfelResult<Self> {
        let mut axes: Vec<usize> = (0..self.form.dims_rank()).collect();
        let stride = self.form.hori_rank();
        match rt.ori() {
            Ori::Hori => {
                for chunk in axes.chunks_exact_mut(stride) {
                    chunk.rotate_left(1);
                }
            }
            Ori::Vert => axes.rotate_left(stride),
        };
        self.move_axes(&axes, rt)
    }
    pub fn swap(mut self, rt: &Ufel) -> UfelResult<Self> {
        let mut new_form = self.form.clone();
        new_form.swap();
        if new_form.is_normal() {
            self.form = new_form;
            return Ok(self);
        }
        let mut dest: Vec<usize> = Vec::with_capacity(self.form.dims_rank());
        for j in 0..self.form.hori_rank() {
            for i in 0..self.form.vert_rank() {
                dest.push(i * self.form.hori_rank() + j);
            }
        }
        self.move_axes(&dest, rt)
    }
    pub(crate) fn move_axes(self, indices: &[usize], rt: &Ufel) -> UfelResult<Self> {
        fn derive_orient_data(
            indices: &[usize],
            axes: &[usize],
            env: &Ufel,
        ) -> UfelResult<(Vec<usize>, FormDims, usize)> {
            let rank = axes.len();

            let mut indices = indices.to_vec();

            // Add missing axes
            let duplicate_count = indices
                .iter()
                .enumerate()
                .filter(|&(i, a)| indices[..i].contains(a))
                .count();
            let max_index = indices.iter().max().copied().unwrap_or(0);
            let min_allowed_rank = max_index + duplicate_count + 1;
            if rank < min_allowed_rank {
                return Err(env.error(format!(
                    "Indices imply a rank of at least {min_allowed_rank}, \
                    but the array is rank {rank}"
                )));
            }
            let new_rank = rank - duplicate_count;
            for i in 0..new_rank {
                if !indices.contains(&i) {
                    indices.push(i);
                }
            }

            // New shape
            let mut new_axes = FormDims::with_capacity(new_rank);
            for i in 0..new_rank {
                new_axes.push(
                    (indices.iter().enumerate())
                        .filter(|&(_, &j)| j == i)
                        .map(|(j, _)| axes[j])
                        .min()
                        .unwrap(),
                );
            }

            // Trailing dimensions
            let trailing_dims = indices
                .iter()
                .enumerate()
                .rev()
                .take_while(|&(i, a)| !indices[..i].contains(a))
                .zip((0..new_rank).rev())
                .take_while(|&((_, &a), b)| a == b)
                .count();

            Ok((indices, new_axes, trailing_dims))
        }

        let (indices, new_dims, trailing_dims) = derive_orient_data(indices, self.form.dims(), rt)?;

        let new_dims_elems: usize = new_dims.iter().product();
        if new_dims_elems == 0 {
            return Ok(Array::new(
                Form::new(self.form.vert_rank(), self.form.hori_rank(), new_dims),
                CowSlice::new(),
            ));
        } else if trailing_dims == self.form.dims_rank() {
            return Ok(self.clone());
        }

        let mut data = self.data.clone();
        data.truncate(new_dims_elems);
        let considered_orig_dims =
            FormDims::from(&self.form.dims()[..self.form.dims_rank() - trailing_dims]);
        let considered_new_dims = FormDims::from(&new_dims[..new_dims.len() - trailing_dims]);
        let trailing_row_len: usize = self.form.dims()[considered_orig_dims.len()..]
            .iter()
            .product();
        let mut orig_index = vec![0; considered_orig_dims.len()];
        let mut new_index = vec![0; considered_new_dims.len()];
        for (i, row) in data
            .as_mut_slice()
            .chunks_exact_mut(trailing_row_len)
            .enumerate()
        {
            flat_to_dims(&considered_new_dims, i, &mut new_index);
            for (j, oi) in orig_index.iter_mut().enumerate() {
                *oi = new_index[indices[j]];
            }
            let j = dims_to_flat(&considered_orig_dims, &orig_index).unwrap();
            row.clone_from_slice(&self.data[j * trailing_row_len..][..trailing_row_len]);
        }

        Ok(Array::new(
            Form::new(self.form.vert_rank(), self.form.hori_rank(), new_dims),
            data,
        ))
    }
}

fn flat_to_dims(axes: &[usize], flat: usize, index: &mut Vec<usize>) {
    index.clear();
    let mut flat = flat;
    for &dim in axes.iter().rev() {
        index.push(flat % dim);
        flat /= dim;
    }
    index.reverse();
}
fn dims_to_flat(axes: &[usize], index: &[usize]) -> Option<usize> {
    let mut flat = 0;
    for (&dim, &i) in axes.iter().zip(index) {
        if i >= dim {
            return None;
        }
        flat = flat * dim + i;
    }
    Some(flat)
}
