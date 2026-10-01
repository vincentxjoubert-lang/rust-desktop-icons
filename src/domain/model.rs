use super::{icons, kind::Kind, order::Sort};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn clean(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(64).collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Look {
    pub color: u32,
    pub alpha: u8,
    pub icon: i32,
    pub tint: Option<u32>,
    pub chameleon: bool,
    pub auto_height: bool,
}

impl Default for Look {
    fn default() -> Self {
        Self { color: 0x302820, alpha: 217, icon: icons::DEFAULT, tint: None, chameleon: false, auto_height: true }
    }
}

impl Look {
    pub const PERCENTS: [u32; 8] = [30, 40, 50, 60, 70, 80, 90, 100];

    pub fn percent(&self) -> u32 {
        (self.alpha as u32 * 100 + 127) / 255
    }

    pub fn label(pct: u32) -> String {
        format!("{pct}%")
    }

    pub fn alpha_for(pct: u32) -> u8 {
        (pct.min(100) * 255 / 100) as u8
    }

    pub(super) fn sanitized(mut self) -> Self {
        self.alpha = self.alpha.max(64);
        self.color &= 0xFF_FFFF;
        self.icon = icons::nearest(self.icon);
        self.tint = self.tint.map(|t| t & 0xFF_FFFF);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub portal: Option<PathBuf>,
    pub kinds: Vec<Kind>,
    pub sort: Sort,
    pub order: Vec<String>,
    pub recycle: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fence {
    pub id: u64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub rolled: bool,
    pub locked: bool,
    #[serde(flatten)]
    pub look: Look,
    pub tabs: Vec<Tab>,
    pub tab: usize,
}

impl Default for Fence {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            x: 100,
            y: 100,
            w: 360,
            h: 240,
            rolled: false,
            locked: false,
            look: Look::default(),
            tabs: vec![],
            tab: 0,
        }
    }
}

impl Fence {
    pub const MIN: (i32, i32) = (120, 80);

    pub fn new(id: u64, title: &str, (x, y, w, h): (i32, i32, i32, i32), look: Look) -> Self {
        let tabs = vec![Tab { id, title: title.into(), ..Tab::default() }];
        Self { id, x, y, w, h, look, tabs, ..Self::default() }
    }

    pub fn active(&self) -> &Tab {
        &self.tabs[self.tab.min(self.tabs.len() - 1)]
    }

    pub fn active_mut(&mut self) -> &mut Tab {
        let i = self.tab.min(self.tabs.len() - 1);
        &mut self.tabs[i]
    }

    pub(super) fn sanitized(mut self) -> Self {
        self.w = self.w.clamp(Self::MIN.0, 10_000);
        self.h = self.h.clamp(Self::MIN.1, 10_000);
        self.x = self.x.clamp(-20_000, 20_000);
        self.y = self.y.clamp(-20_000, 20_000);
        self.look = self.look.sanitized();
        if self.tabs.is_empty() {
            self.tabs.push(Tab { id: self.id, title: std::mem::take(&mut self.title), ..Tab::default() });
        }
        self.title.clear();
        for t in &mut self.tabs {
            t.title = clean(&t.title);
            t.kinds.dedup();
        }
        self.tab = self.tab.min(self.tabs.len() - 1);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_steps() {
        assert_eq!((Look::alpha_for(30), Look::alpha_for(100), Look::alpha_for(500)), (76, 255, 255));
        assert_eq!(Look { alpha: Look::alpha_for(70), ..Look::default() }.percent(), 70);
    }
}
