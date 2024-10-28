use ecow::{eco_vec, EcoVec};

use crate::{pervade::*, Array, Dyadic, Mod, Ori, SigNode, Ufel, UfelResult};

fn flip<T>(f: impl Fn(T, T) -> T) -> impl Fn(T, T) -> T {
    move |a, b| f(b, a)
}

pub fn reduce(f: SigNode, rt: &mut Ufel) -> UfelResult {
    if f.sig.args != 2 {
        return Err(rt.error(format!(
            "{:?}d function must have 2 arguments, \
            but its signature is {:?}",
            Mod::Reduce,
            f.sig
        )));
    }
    let a = rt.pop(1)?;
    let flipped_dy = (f.node)
        .as_flipped_dy()
        .ok_or_else(|| rt.error(format!("Function cannot be {:?}d", Mod::Reduce)))?;
    let ori = rt.ori();
    let res = match flipped_dy {
        (Dyadic::Add, _) => reduce_pervasive(a, 0.0, add::num_num, ori),
        (Dyadic::Sub, false) => reduce_pervasive(a, 0.0, sub::num_num, ori),
        (Dyadic::Sub, true) => reduce_pervasive(a, 0.0, flip(sub::num_num), ori),
        (Dyadic::Mul, _) => reduce_pervasive(a, 1.0, mul::num_num, ori),
        (Dyadic::Div, false) => reduce_pervasive(a, 1.0, div::num_num, ori),
        (Dyadic::Div, true) => reduce_pervasive(a, 1.0, flip(div::num_num), ori),
        (Dyadic::Mod, false) => reduce_pervasive(a, 0.0, mod_::num_num, ori),
        (Dyadic::Mod, true) => reduce_pervasive(a, 0.0, flip(mod_::num_num), ori),
        (Dyadic::Eq, _) => reduce_pervasive(a, 0.0, eq::num_num, ori),
        (Dyadic::Lt, _) => reduce_pervasive(a, 0.0, lt::num_num, ori),
        (Dyadic::Gt, _) => reduce_pervasive(a, 0.0, gt::num_num, ori),
        (Dyadic::Min, _) => reduce_pervasive(a, 0.0, min::num_num, ori),
        (Dyadic::Max, _) => reduce_pervasive(a, 1.0, max::num_num, ori),
    };
    rt.push(res);
    Ok(())
}

fn reduce_pervasive<T: Clone>(
    mut a: Array<T>,
    identity: T,
    f: impl Fn(T, T) -> T,
    ori: Ori,
) -> Array<T> {
    if a.form.is_scalar() {
        return a;
    }
    if a.form.dims().contains(&0) {
        let form = a.form.row(ori);
        let data = eco_vec![identity; form.elems()];
        return Array::new(form, data.into());
    }
    if a.form.rank(ori) == 1 && a.form.rank(!ori) == 1 {
        let elem = a.data.iter().cloned().reduce(f).unwrap_or(identity);
        return Array::scalar(elem);
    }
    let row_count = a.form.row_count(ori);
    let row_len = a.form.row_len(ori);
    let row_form = a.form.row(ori);
    if row_count == 0 {
        let data = eco_vec![identity; row_form.elems()];
        return Array::new(row_form, data.into());
    }
    match ori {
        Ori::Hori => {
            let (acc, rest) = a.data.as_mut_slice().split_at_mut(row_len);
            for chunk in rest.chunks_exact(row_len) {
                for (acc, elem) in acc.iter_mut().zip(chunk) {
                    *acc = f(acc.clone(), elem.clone());
                }
            }
            a.data.truncate(row_len);
            Array::new(row_form, a.data)
        }
        Ori::Vert => {
            let mut acc: EcoVec<T> = a
                .data
                .chunks_exact(row_count)
                .map(|chunk| chunk[0].clone())
                .collect();
            let slice = acc.make_mut();
            for i in 1..row_count {
                for (acc, elem) in slice.iter_mut().zip(a.data.chunks_exact(row_count)) {
                    *acc = f(acc.clone(), elem[i].clone());
                }
            }
            Array::new(row_form, acc.into())
        }
    }
}
