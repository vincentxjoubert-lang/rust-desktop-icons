mod blur;
mod pixels;

pub use blur::blur;
pub use pixels::{gray, opaque_if_flat, straight, tint};

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
        let (x0, x1) = (rect.0.max(0), rect.2.min(self.w));
        for y in rows.0.max(rect.1).max(0)..rows.1.min(rect.3).min(self.h) {
            let color = lerp(top, bottom, ((y - rect.1) * 255 / (rect.3 - rect.1).max(1)) as u32);
            let cy = y as f32 + 0.5;
            let straight = cy >= f.1 + r + 1. && cy <= f.3 - r - 1.;
            let a = (rect.0 + 2).clamp(x0, x1.max(x0));
            let b = (rect.2 - 2).clamp(a, x1.max(a));
            let row = (y * self.w) as usize;
            let (left, right) = if straight { (x0..a, b..x1) } else { (x0..x1, x1..x1) };
            for x in left.chain(right) {
                let p = (x as f32 + 0.5, cy);
                let c = coverage(p, f, r) - if stroke { coverage(p, inner, (r - 1.).max(0.)) } else { 0. };
                if c > 0. {
                    let i = row + x as usize;
                    self.px[i] = over(self.px[i], fade(color, (c * 255.) as u32));
                }
            }
            if straight && !stroke {
                for x in a..b {
                    let i = row + x as usize;
                    self.px[i] = over(self.px[i], color);
                }
            }
        }
    }

    pub fn layer<P: Copy + Into<u32>>(&mut self, src: &[P], (dx, dy): (i32, i32), tint: Option<u32>, rows: (i32, i32)) {
        for y in rows.0.max(0).max(dy)..rows.1.min(self.h).min(self.h + dy) {
            for x in dx.max(0)..self.w.min(self.w + dx) {
                let s: u32 = src[((y - dy) * self.w + x - dx) as usize].into();
                let s = match tint {
                    Some(_) if s & 0xFF == 0 => continue,
                    Some(t) => fade(t, s & 0xFF),
                    None => s,
                };
                if s != 0 {
                    let i = (y * self.w + x) as usize;
                    self.px[i] = over(self.px[i], s);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn fast_rrect_matches_reference() {
        let reference = |rect: (i32, i32, i32, i32), r: f32, stroke: bool| {
            let mut px = vec![0u32; 40 * 40];
            let f = (rect.0 as f32, rect.1 as f32, rect.2 as f32, rect.3 as f32);
            let inner = (f.0 + 1., f.1 + 1., f.2 - 1., f.3 - 1.);
            for y in rect.1.max(0)..rect.3.min(40) {
                let color = lerp(0xFF_20_40_60, 0xFF_20_40_60, ((y - rect.1) * 255 / (rect.3 - rect.1).max(1)) as u32);
                for x in rect.0.max(0)..rect.2.min(40) {
                    let p = (x as f32 + 0.5, y as f32 + 0.5);
                    let c = coverage(p, f, r) - if stroke { coverage(p, inner, (r - 1.).max(0.)) } else { 0. };
                    if c > 0. {
                        px[(y * 40 + x) as usize] = over(0, fade(color, (c * 255.) as u32));
                    }
                }
            }
            px
        };
        for (rect, r) in [((0, 0, 40, 40), 6.), ((3, 5, 30, 38), 8.), ((-4, -2, 44, 20), 0.), ((10, 10, 13, 13), 2.)] {
            for stroke in [false, true] {
                let mut px = vec![0u32; 40 * 40];
                Canvas { px: &mut px, w: 40, h: 40 }.rrect(rect, r, flat(0xFF_20_40_60), (0, 40), stroke);
                assert_eq!(px, reference(rect, r, stroke), "{rect:?} r={r} stroke={stroke}");
            }
        }
    }
}
