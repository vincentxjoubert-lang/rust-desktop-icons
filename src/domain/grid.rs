pub fn cols(w: i32, cell: i32) -> i32 {
    (w / cell).max(1)
}

fn offset(w: i32, cell: i32) -> i32 {
    ((w - cols(w, cell) * cell) / 2).max(0)
}

pub fn origin(i: usize, w: i32, cell: i32) -> (i32, i32) {
    let (c, i) = (cols(w, cell), i as i32);
    (offset(w, cell) + i % c * cell, i / c * cell)
}

pub fn index_at((x, y): (i32, i32), w: i32, cell: i32, n: usize) -> Option<usize> {
    let (c, x) = (cols(w, cell), x - offset(w, cell));
    if x < 0 || y < 0 || x / cell >= c {
        return None;
    }
    let i = (y / cell * c + x / cell) as usize;
    (i < n).then_some(i)
}

pub fn max_scroll(n: usize, w: i32, h: i32, cell: i32) -> i32 {
    let c = cols(w, cell);
    ((n as i32 + c - 1) / c * cell - h).max(0)
}

pub fn fit(n: usize, w: i32, cell: i32, chrome: i32, (min, max): (i32, i32)) -> i32 {
    let rows = (n as i32 + cols(w, cell) - 1) / cols(w, cell);
    (chrome + rows.max(1) * cell).clamp(min, max.max(min))
}

pub fn split(w: i32, n: usize, x: i32) -> Option<usize> {
    let n = n.max(1) as i32;
    (0..w).contains(&x).then(|| (x * n / w.max(1)) as usize)
}

pub fn in_rect(n: usize, w: i32, cell: i32, (x0, y0, x1, y1): (i32, i32, i32, i32)) -> Vec<usize> {
    let (l, r, t, b) = (x0.min(x1), x0.max(x1), y0.min(y1), y0.max(y1));
    (0..n)
        .filter(|&i| {
            let (x, y) = origin(i, w, cell);
            x < r && x + cell > l && y < b && y + cell > t
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_layout() {
        assert_eq!(cols(250, 80), 3);
        assert_eq!(origin(4, 250, 80), (85, 80));
        assert_eq!(index_at((85, 80), 250, 80, 5), Some(4));
        assert_eq!(index_at((165, 80), 250, 80, 5), None);
        assert_eq!(index_at((2, 10), 250, 80, 5), None);
        assert_eq!(max_scroll(7, 250, 100, 80), 140);
        assert_eq!(max_scroll(1, 250, 100, 80), 0);
    }

    #[test]
    fn auto_height() {
        assert_eq!(fit(5, 250, 80, 50, (80, 1000)), 50 + 2 * 80);
        assert_eq!(fit(0, 250, 80, 50, (80, 1000)), 50 + 80);
        assert_eq!(fit(6, 250, 80, 50, (80, 1000)), 50 + 2 * 80);
        assert_eq!(fit(40, 250, 80, 50, (80, 500)), 500);
        assert_eq!(fit(1, 250, 10, 5, (80, 500)), 80);
    }

    #[test]
    fn band_selection() {
        assert_eq!(in_rect(5, 250, 80, (90, 5, 100, 90)), vec![1, 4]);
        assert_eq!(in_rect(5, 250, 80, (100, 90, 90, 5)), vec![1, 4]);
        assert_eq!(in_rect(5, 250, 80, (0, 0, 4, 4)), Vec::<usize>::new());
        assert_eq!(in_rect(3, 250, 80, (0, 0, 250, 300)), vec![0, 1, 2]);
    }

    #[test]
    fn tab_split() {
        assert_eq!(split(300, 3, 0), Some(0));
        assert_eq!(split(300, 3, 150), Some(1));
        assert_eq!(split(300, 3, 299), Some(2));
        assert_eq!(split(300, 3, 300), None);
        assert_eq!(split(300, 1, 10), Some(0));
    }
}
