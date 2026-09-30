pub const SPEEDS: [u32; 5] = [0, 120, 220, 350, 500];
pub const DEFAULT_MS: u32 = 350;
pub const REST: f32 = 0.22;

pub fn advance(p: f32, target: f32, dt_ms: f32, ms: u32) -> f32 {
    if ms == 0 {
        return target;
    }
    let s = dt_ms / ms as f32;
    if target > p { (p + s).min(target) } else { (p - s).max(target) }
}

pub fn ease(p: f32) -> f32 {
    p * p * (3. - 2. * p)
}

pub fn ease_out(p: f32) -> f32 {
    1. - (1. - p).powi(3)
}

fn curve(p: f32, opening: bool) -> f32 {
    if opening { ease_out(p) } else { ease(p) }
}

pub fn retarget(p: f32, from: bool, to: bool) -> f32 {
    let e = curve(p, from);
    let (mut lo, mut hi) = (0f32, 1f32);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.;
        if curve(mid, to) < e { lo = mid } else { hi = mid }
    }
    (lo + hi) / 2.
}

pub fn height(title: i32, full: i32, p: f32, opening: bool) -> i32 {
    title + ((full - title).max(0) as f32 * curve(p, opening)).round() as i32
}

pub fn opacity(glow: f32) -> u8 {
    ((REST + (1. - REST) * ease(glow)) * 255.).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling() {
        assert_eq!(advance(0., 1., 110., 220), 0.5);
        assert_eq!(advance(0.9, 1., 220., 220), 1.);
        assert_eq!(advance(0.1, 0., 220., 220), 0.);
        assert_eq!(advance(0.3, 1., 1., 0), 1.);
        assert_eq!((height(34, 400, 0., false), height(34, 400, 1., false), height(34, 400, 0.5, false)), (34, 400, 217));
        assert!(height(34, 400, 0.1, false) - 34 < 40);
    }

    #[test]
    fn opening_starts_fast_and_reversal_is_continuous() {
        assert!(height(34, 400, 0.1, true) - 34 > 90);
        assert_eq!((height(34, 400, 0., true), height(34, 400, 1., true)), (34, 400));
        for p in [0.1, 0.3, 0.5, 0.8] {
            let q = retarget(p, true, false);
            assert!((curve(p, true) - curve(q, false)).abs() < 1e-4, "p={p}");
            assert!((retarget(q, false, true) - p).abs() < 1e-3);
        }
    }

    #[test]
    fn chameleon_fade() {
        assert_eq!(opacity(1.), 255);
        assert_eq!(opacity(0.), 56);
    }
}
