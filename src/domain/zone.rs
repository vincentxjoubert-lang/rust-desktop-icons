#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zone {
    Client,
    Caption,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Zone {
    pub fn fixed(self) -> Zone {
        use Zone::*;
        match self {
            Caption | Top => Caption,
            _ => Client,
        }
    }

    pub fn width_only(self) -> Zone {
        use Zone::*;
        match self {
            Top => Caption,
            Bottom => Client,
            TopLeft | BottomLeft => Left,
            TopRight | BottomRight => Right,
            z => z,
        }
    }
}

pub fn zone((w, h): (i32, i32), (x, y): (i32, i32), border: i32, title: i32, rolled: bool) -> Zone {
    use Zone::*;
    let (l, r) = (x < border, x >= w - border);
    if rolled {
        return if l {
            Left
        } else if r {
            Right
        } else {
            Caption
        };
    }
    match (l, r, y < border, y >= h - border) {
        (true, _, true, _) => TopLeft,
        (_, true, true, _) => TopRight,
        (true, _, _, true) => BottomLeft,
        (_, true, _, true) => BottomRight,
        (true, ..) => Left,
        (_, true, ..) => Right,
        (_, _, true, _) => Top,
        (.., true) => Bottom,
        _ if y < title => Caption,
        _ => Client,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zones() {
        let z = |x, y, r| zone((200, 100), (x, y), 6, 28, r);
        assert_eq!(z(0, 0, false), Zone::TopLeft);
        assert_eq!(z(199, 99, false), Zone::BottomRight);
        assert_eq!(z(100, 10, false), Zone::Caption);
        assert_eq!(z(100, 50, false), Zone::Client);
        assert_eq!(z(100, 99, false), Zone::Bottom);
        assert_eq!(z(100, 99, true), Zone::Caption);
        assert_eq!(z(199, 10, true), Zone::Right);
        assert_eq!(z(0, 0, false).width_only(), Zone::Left);
        assert_eq!(z(100, 99, false).width_only(), Zone::Client);
        assert_eq!(z(100, 0, false).width_only(), Zone::Caption);
        assert_eq!(z(0, 50, false).fixed(), Zone::Client);
        assert_eq!(z(100, 10, false).fixed(), Zone::Caption);
        assert_eq!(z(199, 99, false).fixed(), Zone::Client);
    }
}
