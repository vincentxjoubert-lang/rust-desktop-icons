pub fn premul(c: u32, a: u8) -> u32 {
    let k = |v: u32| v * a as u32 / 255;
    (a as u32) << 24 | k(c & 0xFF) << 16 | k(c >> 8 & 0xFF) << 8 | k(c >> 16 & 0xFF)
}

pub fn fade(p: u32, k: u32) -> u32 {
    (0..4).fold(0, |acc, i| acc | ((p >> (8 * i) & 0xFF) * k / 255) << (8 * i))
}

pub fn over(dst: u32, src: u32) -> u32 {
    src + fade(dst, 255 - (src >> 24))
}

pub fn lerp(a: u32, b: u32, k: u32) -> u32 {
    fade(a, 255 - k) + fade(b, k)
}

pub fn flat(c: u32) -> (u32, u32) {
    (c, c)
}

fn coverage((px, py): (f32, f32), (x0, y0, x1, y1): (f32, f32, f32, f32), r: f32) -> f32 {
    let (hw, hh) = ((x1 - x0) / 2., (y1 - y0) / 2.);
    let (qx, qy) = ((px - x0 - hw).abs() - hw + r, (py - y0 - hh).abs() - hh + r);
    let d = qx.max(0.).hypot(qy.max(0.)) + qx.max(qy).min(0.) - r;
    (0.5 - d).clamp(0., 1.)
}

pub struct Canvas<'a> {
    pub px: &'a mut [u32],
    pub w: i32,
    pub h: i32,
}

impl Canvas<'_> {
    pub fn rrect(&mut self, rect: (i32, i32, i32, i32), r: f32, (top, bottom): (u32, u32), rows: (i32, i32), stroke: bool) {
        let f = (rect.0 as f32, rect.1 as f32, rect.2 as f32, rect.3 as f32);
        let inner = (f.0 + 1., f.1 + 1., f.2 - 1., f.3 - 1.);
        for y in rows.0.max(rect.1).max(0)..rows.1.min(rect.3).min(self.h) {
            let color = lerp(top, bottom, ((y - rect.1) * 255 / (rect.3 - rect.1).max(1)) as u32);
            for x in rect.0.max(0)..rect.2.min(self.w) {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let c = coverage(p, f, r) - if stroke { coverage(p, inner, (r - 1.).max(0.)) } else { 0. };
                if c > 0. {
                    let i = (y * self.w + x) as usize;
                    self.px[i] = over(self.px[i], fade(color, (c * 255.) as u32));
                }
            }
        }
    }

    pub fn layer(&mut self, src: &[u32], (dx, dy): (i32, i32), tint: Option<u32>, rows: (i32, i32)) {
        for y in rows.0.max(0).max(dy)..rows.1.min(self.h).min(self.h + dy) {
            for x in dx.max(0)..self.w.min(self.w + dx) {
                let s = src[((y - dy) * self.w + x - dx) as usize];
                let s = tint.map_or(s, |t| fade(t, s & 0xFF));
                if s != 0 {
                    let i = (y * self.w + x) as usize;
                    self.px[i] = over(self.px[i], s);
                }
            }
        }
    }
}

pub fn opaque_if_flat(px: &mut [u32], w: i32, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
    let idx = |x: i32, y: i32| (y * w + x) as usize;
    let cells = || (y0.max(0)..y1).flat_map(move |y| (x0.max(0)..x1.min(w)).map(move |x| idx(x, y)));
    if cells().filter(|&i| i < px.len()).all(|i| px[i] >> 24 == 0) {
        let len = px.len();
        for i in cells().filter(|&i| i < len) {
            if px[i] != 0 {
                px[i] |= 0xFF00_0000;
            }
        }
    }
}

fn pass(src: &[u32], dst: &mut [u32], (lines, len, line_step, step): (usize, usize, usize, usize), r: usize) {
    let d = 2 * r as u32 + 1;
    for l in 0..lines {
        let at = |i: usize| l * line_step + i * step;
        let mut sum: u32 = (0..=r.min(len - 1)).map(|i| src[at(i)]).sum();
        for i in 0..len {
            dst[at(i)] = sum / d;
            if i + r + 1 < len {
                sum += src[at(i + r + 1)];
            }
            if i >= r {
                sum -= src[at(i - r)];
            }
        }
    }
}

pub fn gray(px: &mut [u32]) {
    for p in px {
        *p = ((*p & 0xFF) + (*p >> 8 & 0xFF) + (*p >> 16 & 0xFF)) / 3 * 0x0101_0101;
    }
}

pub fn tint(px: &mut [u32], color: u32) {
    let t = premul(color, 255) & 0xFF_FFFF;
    for p in px.iter_mut().filter(|p| **p >> 24 != 0) {
        let a = *p >> 24;
        let lum = ((*p >> 16 & 0xFF) * 77 + (*p >> 8 & 0xFF) * 150 + (*p & 0xFF) * 29) >> 8;
        *p = a << 24 | fade(t, (lum + a) / 2);
    }
}

pub fn blur(src: &[u32], w: i32, h: i32, r: i32) -> Vec<u32> {
    let (w, h, r) = (w.max(1) as usize, h.max(1) as usize, r.max(0) as usize);
    let mut a: Vec<u32> = src.iter().map(|p| p & 0xFF).collect();
    let mut b = vec![0; a.len()];
    for _ in 0..2 {
        pass(&a, &mut b, (h, w, w, 1), r);
        pass(&b, &mut a, (w, h, 1, w), r);
    }
    a
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
        assert!(b.iter().all(|&v| v <= 255));
        assert_eq!(blur(&[0x12_34_56_FF; 9], 3, 3, 0), vec![255; 9]);
    }

    #[test]
    fn cleartype_to_gray() {
        let mut px = [0x00_FF_00_00, 0x00_30_60_90, 0];
        gray(&mut px);
        assert_eq!(px, [0x5555_5555, 0x6060_6060, 0]);
    }

    #[test]
    fn tinting() {
        let mut px = [0xFF_FF_FF_FF, 0x80_00_00_00, 0];
        tint(&mut px, 0x00_00_FF);
        assert_eq!(px, [0xFF_FF_00_00, 0x80_40_00_00, 0]);
    }

    #[test]
    fn blending() {
        assert_eq!(premul(0x00_FF_80_00, 128), 0x80_00_40_80);
        assert_eq!(over(0xFF_00_00_FF, 0xFF_FF_00_00), 0xFF_FF_00_00);
        assert_eq!(over(0xFF_00_00_FF, 0), 0xFF_00_00_FF);
        assert_eq!(lerp(0xFF_00_00_00, 0xFF_FF_FF_FF, 255), 0xFF_FF_FF_FF);
        assert_eq!(lerp(0x80_00_00_00, 0xFF_00_00_00, 0), 0x80_00_00_00);
        assert_eq!(over(0xFF_00_00_FE, 0x80_80_00_00), 0xFF_80_00_7E);
    }

    #[test]
    fn rounded_rect_and_layers() {
        let mut px = vec![0u32; 20 * 20];
        let mut c = Canvas { px: &mut px, w: 20, h: 20 };
        c.rrect((0, 0, 20, 20), 6., flat(0xFF_FF_FF_FF), (0, 20), false);
        assert_eq!(c.px[10 * 20 + 10], 0xFF_FF_FF_FF);
        assert_eq!(c.px[0], 0);
        let mask = vec![0x00_00_00_FFu32; 400];
        let mut px2 = vec![0u32; 400];
        let mut d = Canvas { px: &mut px2, w: 20, h: 20 };
        d.layer(&mask, (1, 1), Some(0xFF_10_20_30), (0, 20));
        assert_eq!((d.px[0], d.px[21]), (0, 0xFF_10_20_30));
        let mut flat = vec![0x00_12_34_56u32, 0];
        opaque_if_flat(&mut flat, 2, (0, 0, 2, 1));
        assert_eq!(flat, [0xFF_12_34_56, 0]);
    }
}
