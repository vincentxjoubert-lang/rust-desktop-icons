fn rgb(c: u32) -> [i32; 3] {
    [(c & 0xFF) as i32, (c >> 8 & 0xFF) as i32, (c >> 16 & 0xFF) as i32]
}

pub fn shade(c: u32, amount: i32) -> u32 {
    rgb(c).iter().enumerate().fold(0, |acc, (i, v)| acc | ((v + amount).clamp(0, 255) as u32) << (8 * i))
}

pub fn contrast(c: u32) -> u32 {
    let [r, g, b] = rgb(c);
    if r * 299 + g * 587 + b * 114 > 140_000 { 0x20_2020 } else { 0xF5_F5F5 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors() {
        assert_eq!(shade(0x10_20F0, 20), 0x24_34FF);
        assert_eq!(shade(0x05_0505, -10), 0);
        assert_eq!(contrast(0xFF_FFFF), 0x20_2020);
        assert_eq!(contrast(0), 0xF5_F5F5);
    }
}
