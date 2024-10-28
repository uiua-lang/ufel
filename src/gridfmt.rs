use crate::Array;

impl Array {
    pub fn show(&self) -> String {
        if self.form.is_scalar() {
            return self.data[0].to_string();
        }
        let elem_strs: Vec<String> = self.data.iter().map(|x| x.to_string()).collect();
        if self.form.dims_rank() == 1 {
            let mut s = String::from('[');
            for (i, elem) in elem_strs.iter().enumerate() {
                if i > 0 {
                    s.push(' ');
                }
                s.push_str(elem);
            }
            s.push(']');
            return s;
        }
        let last_dim = *self.form.dims().last().unwrap();
        let mut max_widths = vec![0; last_dim];
        for (i, s) in elem_strs.iter().enumerate() {
            max_widths[i % last_dim] = max_widths[i % last_dim].max(s.len());
        }
        let mut width = max_widths.iter().sum::<usize>() + last_dim.saturating_sub(1) + 5;
        let mut overflow = false;
        if let Some((w, _)) = terminal_size::terminal_size() {
            let w = w.0 as usize;
            overflow = w < width;
            width = width.min(w);
        }
        let mut height = 1;
        let mut j = 0;
        for (i, row) in self.form.hori_axis_rows().rev().enumerate() {
            let mut dims = row.iter().rev();
            if i == 0 {
                dims.next();
            }
            for &dim in dims {
                height *= dim;
                height += dim.saturating_sub(1) * (j != 0) as usize;
                j += 1;
            }
        }
        height = height.max(self.form.dims_rank() - 1);
        height += 2;
        let mut grid = vec![' '; width * height];
        let mut curr = vec![0; self.form.dims_rank() - 1];
        let mut strs = elem_strs.into_iter();
        let mut rows = grid.chunks_exact_mut(width);
        rows.next();
        while let Some(row) = rows.next() {
            let mut j = 2;
            for (k, (s, w)) in strs.by_ref().take(last_dim).zip(&max_widths).enumerate() {
                if k > 0 {
                    j += 1;
                }
                j += *w - s.len();
                if j >= width - 3 {
                    break;
                }
                for c in s.chars() {
                    row[j] = c;
                    j += 1;
                    if j >= width - 3 {
                        break;
                    }
                }
            }
            if overflow {
                row[width - 3] = '…';
            }
            for (curr, &dim) in curr.iter_mut().zip(self.form.dims()).rev() {
                *curr += 1;
                if *curr < dim {
                    break;
                }
                *curr = 0;
                rows.next();
            }
        }
        for row in grid.chunks_exact_mut(width) {
            *row.last_mut().unwrap() = '\n';
        }
        grid[0] = '╭';
        grid[1] = '─';
        for i in 1..self.form.dims_rank() {
            grid[i * width] = '╷';
        }
        grid[width * height - 2] = '╯';
        grid.pop();
        grid.into_iter().collect()
    }
}
