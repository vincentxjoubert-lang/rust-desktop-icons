pub const SIZES: [i32; 8] = [24, 32, 40, 48, 64, 80, 96, 128];
pub const DEFAULT: i32 = 48;

pub fn nearest(px: i32) -> i32 {
    *SIZES.iter().min_by_key(|s| (*s - px).abs()).unwrap_or(&DEFAULT)
}

pub fn step(px: i32, up: bool) -> i32 {
    let i = SIZES.iter().position(|&s| s == nearest(px)).unwrap_or(3);
    SIZES[if up { (i + 1).min(SIZES.len() - 1) } else { i.saturating_sub(1) }]
}

pub fn cell(px: i32) -> i32 {
    px + 56
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_sizes() {
        assert_eq!(nearest(50), 48);
        assert_eq!(nearest(0), 24);
        assert_eq!(step(48, true), 64);
        assert_eq!(step(48, false), 40);
        assert_eq!(step(128, true), 128);
        assert_eq!(step(24, false), 24);
        assert_eq!(cell(DEFAULT), 104);
    }
}
