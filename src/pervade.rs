use ecow::eco_vec;

use crate::{Array, Ufel, UfelResult};

pub fn pervade<A, B, C>(
    a: Array<A>,
    b: Array<B>,
    _a_depth: usize,
    _b_depth: usize,
    f: impl Fn(A, B) -> C,
    rt: &Ufel,
) -> UfelResult<Array<C>>
where
    A: Clone,
    B: Clone,
    C: Clone + Default,
{
    if a.form == b.form {
        let mut c_data = eco_vec![C::default(); a.form.elems()];
        for ((a, b), c) in a
            .data
            .into_iter()
            .zip(b.data.into_iter())
            .zip(c_data.make_mut())
        {
            *c = f(a, b);
        }
        Ok(Array {
            form: a.form,
            data: c_data.into(),
        })
    } else if a.form.is_prefix_of(&b.form) {
        let mut c_data = eco_vec![C::default(); b.form.elems()];
        let a_elem_count = a.form.elems();
        for (b, c) in (b.data.chunks_exact(a_elem_count))
            .zip(c_data.make_mut().chunks_exact_mut(a_elem_count))
        {
            for ((a, b), c) in a.data.iter().zip(b).zip(c) {
                *c = f(a.clone(), b.clone());
            }
        }
        Ok(Array {
            form: b.form,
            data: c_data.into(),
        })
    } else if b.form.is_prefix_of(&a.form) {
        let mut c_data = eco_vec![C::default(); a.form.elems()];
        let b_elem_count = b.form.elems();
        for (a, c) in (a.data.chunks_exact(b_elem_count))
            .zip(c_data.make_mut().chunks_exact_mut(b_elem_count))
        {
            for ((a, b), c) in a.iter().zip(b.data.iter()).zip(c) {
                *c = f(a.clone(), b.clone());
            }
        }
        Ok(Array {
            form: a.form,
            data: c_data.into(),
        })
    } else {
        Err(rt.error(format!(
            "Forms {:?} and {:?} are not compatible",
            a.form, b.form
        )))
    }
}

// Monadic

pub mod neg {
    pub fn num(a: f64) -> f64 {
        -a
    }
}

pub mod not {
    pub fn num(a: f64) -> f64 {
        1.0 - a
    }
}

pub mod abs {
    pub fn num(a: f64) -> f64 {
        a.abs()
    }
}

pub mod sign {
    pub fn num(a: f64) -> f64 {
        a.signum()
    }
}

// Dyadic

pub mod add {
    pub fn num_num(a: f64, b: f64) -> f64 {
        b + a
    }
}

pub mod sub {
    pub fn num_num(a: f64, b: f64) -> f64 {
        b - a
    }
}

pub mod mul {
    pub fn num_num(a: f64, b: f64) -> f64 {
        b * a
    }
}

pub mod div {
    pub fn num_num(a: f64, b: f64) -> f64 {
        b / a
    }
}

pub mod mod_ {
    pub fn num_num(a: f64, b: f64) -> f64 {
        b.rem_euclid(a)
    }
}

pub mod eq {
    pub fn num_num(a: f64, b: f64) -> f64 {
        (a == b) as u8 as f64
    }
}
pub mod ne {
    pub fn num_num(a: f64, b: f64) -> f64 {
        (a != b) as u8 as f64
    }
}

pub mod lt {
    pub fn num_num(a: f64, b: f64) -> f64 {
        (b < a) as u8 as f64
    }
}

pub mod gt {
    pub fn num_num(a: f64, b: f64) -> f64 {
        (b > a) as u8 as f64
    }
}

pub mod min {
    pub fn num_num(a: f64, b: f64) -> f64 {
        a.min(b)
    }
}

pub mod max {
    pub fn num_num(a: f64, b: f64) -> f64 {
        a.max(b)
    }
}
