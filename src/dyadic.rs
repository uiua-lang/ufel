use crate::{Array, Element, Form, FormDims, Ori, Ufel, UfelResult};

impl<T: Element> Array<T> {
    pub fn chunk(mut self, size: Array, rt: &Ufel) -> UfelResult<Self> {
        if !size.form.is_normal() {
            return Err(rt.error(format!(
                "Chunk size must be normal, but its form is {:?}",
                size.form
            )));
        }
        if size.form.hori_rank() > 1 {
            return Err(rt.error(format!(
                "Chunk size must be a scalar or list, but its form is {:?}",
                size.form
            )));
        }
        let size = size.data.as_slice();
        for &size in size {
            if size.fract() != 0.0 {
                return Err(rt.error(format!(
                    "Chunk size must be all integers, \
                    but one element is {size}"
                )));
            }
        }
        let shape = self.form.shape(rt.ori());
        if size.len() > shape.len() {
            return Err(rt.error(format!(
                "Chunk size has too many axes for {} shape {:?}",
                rt.ori().str(),
                shape
            )));
        }
        let mut new_dims = FormDims::with_capacity(self.form.dims_rank() + size.len());
        let mut dests = Vec::with_capacity(self.form.dims_rank() + size.len());
        match rt.ori() {
            Ori::Hori => {
                for i in 0..self.form.hori_rank() {
                    let dim = self.form.dims()[i];
                    if let Some(&sz) = size.get(i) {
                        let abs_sz = (sz as isize).unsigned_abs();
                        if abs_sz == 0 || dim % abs_sz != 0 {
                            return Err(rt.error(format!(
                                "Chunk size {sz} does not evenly divide axis {i} size {dim}"
                            )));
                        }
                        let sz = if sz >= 0.0 { abs_sz } else { dim / abs_sz };
                        new_dims.push(dim / sz);
                        dests.push(i);
                        new_dims.push(sz);
                        dests.push(i + shape.len());
                    } else {
                        new_dims.push(dim);
                        dests.push(i);
                        new_dims.push(1);
                        dests.push(i + shape.len());
                    }
                }
                for i in 1..self.form.vert_rank() {
                    for j in 0..self.form.hori_rank() {
                        new_dims.push(self.form.dims()[i * self.form.hori_rank() + j]);
                    }
                }
                self.form = Form::new(self.form.vert_rank() + 1, self.form.hori_rank(), new_dims);
                self.validate_form();
                self.move_axes(&dests, rt)
            }
            Ori::Vert => todo!(),
        }
    }
}
