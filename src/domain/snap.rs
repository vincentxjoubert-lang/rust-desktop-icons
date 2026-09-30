pub type Rect = [i32; 4];

fn near(pairs: [(i32, i32); 2], d: i32) -> i32 {
    pairs.iter().find(|(v, t)| (t - v).abs() < d).map_or(0, |(v, t)| t - v)
}

pub fn moving(r: Rect, area: Rect, d: i32) -> Rect {
    let dx = near([(r[0], area[0]), (r[2], area[2])], d);
    let dy = near([(r[1], area[1]), (r[3], area[3])], d);
    [r[0] + dx, r[1] + dy, r[2] + dx, r[3] + dy]
}

pub fn sizing(r: Rect, area: Rect, d: i32, sides: [bool; 4]) -> Rect {
    std::array::from_fn(|i| if sides[i] && (area[i] - r[i]).abs() < d { area[i] } else { r[i] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapping() {
        let area = [0, 0, 1920, 1040];
        assert_eq!(moving([5, 300, 365, 540], area, 16), [0, 300, 360, 540]);
        assert_eq!(moving([-7, 1030, 353, 1050], area, 16), [0, 1020, 360, 1040]);
        assert_eq!(moving([1565, 10, 1925, 250], area, 16), [1560, 0, 1920, 240]);
        assert_eq!(moving([100, 100, 460, 340], area, 16), [100, 100, 460, 340]);
        assert_eq!(sizing([4, 4, 1910, 1045], area, 16, [false, true, true, false]), [4, 0, 1920, 1045]);
        assert_eq!(sizing([4, 4, 1910, 1045], area, 16, [true; 4]), area);
    }
}
