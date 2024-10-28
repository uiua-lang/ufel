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
        let max_width = elem_strs.iter().map(|s| s.len()).max().unwrap_or(0);
        let last_dim = *self.form.dims().last().unwrap();
        let width = max_width * last_dim + last_dim.saturating_sub(1) + 5;
        let mut height = 1;
        for (i, row) in self.form.hori_axis_rows().rev().enumerate() {
            let mut dims = row.iter().rev();
            if i == 0 {
                dims.next();
            }
            for (j, &dim) in dims.enumerate() {
                height *= dim;
                height += dim.saturating_sub(1) * ((i, j) != (0, 0)) as usize;
            }
        }
        height += 2;
        let mut grid = vec![' '; width * height];
        let mut curr = vec![0; self.form.dims_rank() - 1];
        let mut strs = elem_strs.into_iter();
        let mut rows = grid.chunks_exact_mut(width);
        rows.next();
        while let Some(row) = rows.next() {
            let mut j = 2;
            for (k, s) in strs.by_ref().take(last_dim).enumerate() {
                if k > 0 {
                    j += 1;
                }
                j += max_width - s.len();
                for c in s.chars() {
                    row[j] = c;
                    j += 1;
                }
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
            grid[i * width] = '╷'
        }
        grid[width * height - 2] = '╯';
        grid.pop();
        grid.into_iter().collect()
    }
}
