fn pass(src: &[u8], dst: &mut [u8], (lines, len, line_step, step): (usize, usize, usize, usize), r: usize) {
    let d = 2 * r as u32 + 1;
    for l in 0..lines {
        let at = |i: usize| l * line_step + i * step;
        let mut sum: u32 = (0..=r.min(len - 1)).map(|i| src[at(i)] as u32).sum();
        for i in 0..len {
            dst[at(i)] = (sum / d) as u8;
            if i + r + 1 < len {
                sum += src[at(i + r + 1)] as u32;
            }
            if i >= r {
                sum -= src[at(i - r)] as u32;
            }
        }
    }
}

fn blur_all(src: &[u32], w: usize, h: usize, r: usize) -> Vec<u8> {
    let mut a: Vec<u8> = src.iter().map(|p| *p as u8).collect();
    let mut b = vec![0; a.len()];
    for _ in 0..2 {
        pass(&a, &mut b, (h, w, w, 1), r);
        pass(&b, &mut a, (w, h, 1, w), r);
    }
    a
}

pub fn blur(src: &[u32], w: i32, h: i32, r: i32) -> Vec<u8> {
    let (w, h, r) = (w.max(1) as usize, h.max(1) as usize, r.max(0) as usize);
    let used = |y: &usize| src[y * w..(y + 1) * w].iter().any(|p| p & 0xFF != 0);
    let mut out = vec![0; w * h];
    let (Some(first), Some(last)) = ((0..h).find(used), (0..h).rev().find(used)) else { return out };
    let (y0, y1) = (first.saturating_sub(2 * r), (last + 1 + 2 * r).min(h));
    out[y0 * w..y1 * w].copy_from_slice(&blur_all(&src[y0 * w..y1 * w], w, y1 - y0, r));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_blur() {
        let mut px = vec![0u32; 49];
        px[24] = 0xFF_FF_FF_FF;
        let b = blur(&px, 7, 7, 1);
        assert!(b[24] < 255 && b[24] > 0 && b[23] > 0 && b[0] == 0);
        assert_eq!(blur(&[0x12_34_56_FF; 9], 3, 3, 0), vec![255; 9]);
    }

    #[test]
    fn banded_blur_matches_full() {
        let (w, h) = (9, 30);
        let mut px = vec![0u32; w * h];
        px[12 * w + 4] = 0xFF;
        px[14 * w + 1] = 0x80;
        assert_eq!(blur(&px, w as i32, h as i32, 2), blur_all(&px, w, h, 2));
        assert_eq!(blur(&vec![0; w * h], w as i32, h as i32, 2), vec![0; w * h]);
    }
}
