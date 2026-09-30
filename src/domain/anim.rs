pub const SPEEDS: [u32; 5] = [0, 120, 220, 350, 500];
pub const DEFAULT_MS: u32 = 220;
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

pub fn height(title: i32, full: i32, p: f32) -> i32 {
    title + ((full - title).max(0) as f32 * ease(p)).round() as i32
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
        assert_eq!((height(34, 400, 0.), height(34, 400, 1.), height(34, 400, 0.5)), (34, 400, 217));
        assert!(height(34, 400, 0.1) - 34 < 40);
    }

    #[test]
    fn chameleon_fade() {
        assert_eq!(opacity(1.), 255);
        assert_eq!(opacity(0.), 56);
    }
}
