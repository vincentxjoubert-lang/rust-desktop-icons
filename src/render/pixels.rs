use super::{fade, premul};

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

pub fn gray(px: &mut [u32]) {
    for p in px {
        *p = ((*p & 0xFF) + (*p >> 8 & 0xFF) + (*p >> 16 & 0xFF)) / 3 * 0x0101_0101;
    }
}

pub fn straight(px: &mut [u32]) {
    for p in px.iter_mut() {
        let a = *p >> 24;
        if a != 0 && a != 255 {
            let k = |v: u32| (v * 255 / a).min(255);
            *p = a << 24 | k(*p >> 16 & 0xFF) << 16 | k(*p >> 8 & 0xFF) << 8 | k(*p & 0xFF);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleartype_to_gray() {
        let mut px = [0x00_FF_00_00, 0x00_30_60_90, 0];
        gray(&mut px);
        assert_eq!(px, [0x5555_5555, 0x6060_6060, 0]);
    }

    #[test]
    fn unpremultiply() {
        let mut px = [premul(0x00_80_FF, 128), 0xFF_12_34_56, 0];
        straight(&mut px);
        assert_eq!(px, [0x80_FF_7F_00, 0xFF_12_34_56, 0]);
    }

    #[test]
    fn tinting() {
        let mut px = [0xFF_FF_FF_FF, 0x80_00_00_00, 0];
        tint(&mut px, 0x00_00_FF);
        assert_eq!(px, [0xFF_FF_00_00, 0x80_40_00_00, 0]);
    }
}
