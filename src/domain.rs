use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fence {
    pub id: u64,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: u32,
    pub alpha: u8,
    pub rolled: bool,
}

impl Default for Fence {
    fn default() -> Self {
        Self { id: 0, title: String::new(), x: 100, y: 100, w: 360, h: 240, color: 0x302820, alpha: 217, rolled: false }
    }
}

impl Fence {
    pub const MIN: (i32, i32) = (120, 80);

    fn sanitized(mut self) -> Self {
        self.w = self.w.clamp(Self::MIN.0, 10_000);
        self.h = self.h.clamp(Self::MIN.1, 10_000);
        self.x = self.x.clamp(-20_000, 20_000);
        self.y = self.y.clamp(-20_000, 20_000);
        self.alpha = self.alpha.max(64);
        self.color &= 0xFF_FFFF;
        self.title = self.title.chars().filter(|c| !c.is_control()).take(64).collect();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lang: Option<String>,
    pub auto_update: bool,
    pub autostart: bool,
    pub fences: Vec<Fence>,
}

impl Default for Config {
    fn default() -> Self {
        Self { lang: None, auto_update: true, autostart: true, fences: vec![] }
    }
}

impl Config {
    pub fn next_id(&self) -> u64 {
        self.fences.iter().map(|f| f.id).max().map_or(1, |m| m + 1)
    }

    pub fn sanitized(mut self) -> Self {
        let mut seen = HashSet::new();
        self.fences = self.fences.into_iter().filter(|f| f.id > 0 && seen.insert(f.id)).map(Fence::sanitized).collect();
        self
    }
}

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

pub mod grid {
    pub fn cols(w: i32, cell: i32) -> i32 {
        (w / cell).max(1)
    }

    fn offset(w: i32, cell: i32) -> i32 {
        ((w - cols(w, cell) * cell) / 2).max(0)
    }

    pub fn origin(i: usize, w: i32, cell: i32) -> (i32, i32) {
        let (c, i) = (cols(w, cell), i as i32);
        (offset(w, cell) + i % c * cell, i / c * cell)
    }

    pub fn index_at((x, y): (i32, i32), w: i32, cell: i32, n: usize) -> Option<usize> {
        let (c, x) = (cols(w, cell), x - offset(w, cell));
        if x < 0 || y < 0 || x / cell >= c {
            return None;
        }
        let i = (y / cell * c + x / cell) as usize;
        (i < n).then_some(i)
    }

    pub fn max_scroll(n: usize, w: i32, h: i32, cell: i32) -> i32 {
        let c = cols(w, cell);
        ((n as i32 + c - 1) / c * cell - h).max(0)
    }
}

pub mod color {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_and_sanitize() {
        let c = Config {
            fences: vec![
                Fence { id: 3, w: 5, alpha: 0, color: 0xFF12_3456, title: "a\nb".into(), ..Fence::default() },
                Fence { id: 3, ..Fence::default() },
                Fence { id: 0, ..Fence::default() },
            ],
            ..Config::default()
        };
        let c: Config = serde_json::from_str::<Config>(&serde_json::to_string(&c).unwrap()).unwrap().sanitized();
        assert_eq!(c.fences.len(), 1);
        let f = &c.fences[0];
        assert_eq!((f.w, f.alpha, f.color, f.title.as_str()), (Fence::MIN.0, 64, 0x12_3456, "ab"));
        assert_eq!(c.next_id(), 4);
        assert_eq!(Config::default().next_id(), 1);
    }

    #[test]
    fn partial_json_uses_defaults() {
        let c: Config = serde_json::from_str(r#"{"fences":[{"id":1}]}"#).unwrap();
        assert!(c.auto_update && c.autostart);
        assert_eq!(c.fences[0].w, 360);
    }

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
    }

    #[test]
    fn grid_layout() {
        assert_eq!(grid::cols(250, 80), 3);
        assert_eq!(grid::origin(4, 250, 80), (85, 80));
        assert_eq!(grid::index_at((85, 80), 250, 80, 5), Some(4));
        assert_eq!(grid::index_at((165, 80), 250, 80, 5), None);
        assert_eq!(grid::index_at((2, 10), 250, 80, 5), None);
        assert_eq!(grid::max_scroll(7, 250, 100, 80), 140);
        assert_eq!(grid::max_scroll(1, 250, 100, 80), 0);
    }

    #[test]
    fn colors() {
        assert_eq!(color::shade(0x10_20F0, 20), 0x24_34FF);
        assert_eq!(color::shade(0x05_0505, -10), 0);
        assert_eq!(color::contrast(0xFF_FFFF), 0x20_2020);
        assert_eq!(color::contrast(0), 0xF5_F5F5);
    }
}
