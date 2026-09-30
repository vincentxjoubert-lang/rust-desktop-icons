use super::{anim, icons, kind::Kind};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

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
}

impl Default for Look {
    fn default() -> Self {
        Self { color: 0x302820, alpha: 217, icon: icons::DEFAULT, tint: None, chameleon: false }
    }
}

impl Look {
    pub const PERCENTS: [u32; 8] = [30, 40, 50, 60, 70, 80, 90, 100];

    pub fn alpha_for(pct: u32) -> u8 {
        (pct.min(100) * 255 / 100) as u8
    }

    fn sanitized(mut self) -> Self {
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
    #[serde(flatten)]
    pub look: Look,
    pub tabs: Vec<Tab>,
    pub tab: usize,
}

impl Default for Fence {
    fn default() -> Self {
        Self { id: 0, title: String::new(), x: 100, y: 100, w: 360, h: 240, rolled: false, look: Look::default(), tabs: vec![], tab: 0 }
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

    fn sanitized(mut self) -> Self {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lang: Option<String>,
    pub auto_update: bool,
    pub autostart: bool,
    pub auto_sort: bool,
    pub roll_ms: u32,
    pub look: Look,
    pub fences: Vec<Fence>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lang: None,
            auto_update: true,
            autostart: true,
            auto_sort: true,
            roll_ms: anim::DEFAULT_MS,
            look: Look::default(),
            fences: vec![],
        }
    }
}

impl Config {
    pub fn next_id(&self) -> u64 {
        self.fences.iter().flat_map(|f| f.tabs.iter().map(|t| t.id).chain([f.id])).max().map_or(1, |m| m + 1)
    }

    pub fn rule_target(&self, kind: Kind) -> Option<(u64, u64)> {
        self.fences.iter().find_map(|f| f.tabs.iter().find(|t| t.portal.is_none() && t.kinds.contains(&kind)).map(|t| (f.id, t.id)))
    }

    pub fn sanitized(mut self) -> Self {
        let mut seen = HashSet::new();
        self.fences = self.fences.into_iter().filter(|f| f.id > 0 && seen.insert(f.id)).map(Fence::sanitized).collect();
        let mut tabs = HashSet::new();
        for f in &mut self.fences {
            f.tabs.retain(|t| t.id > 0 && tabs.insert(t.id));
            if f.tabs.is_empty() {
                f.tabs.push(Tab { id: f.id, ..Tab::default() });
            }
            f.tab = f.tab.min(f.tabs.len() - 1);
        }
        self.look = self.look.sanitized();
        self.roll_ms = *anim::SPEEDS.iter().min_by_key(|s| s.abs_diff(self.roll_ms)).unwrap_or(&anim::DEFAULT_MS);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_and_sanitize() {
        let look = Look { alpha: 0, color: 0xFF12_3456, ..Look::default() };
        let c = Config {
            fences: vec![
                Fence { id: 3, w: 5, look, title: "a\nb".into(), ..Fence::default() },
                Fence { id: 3, ..Fence::default() },
                Fence { id: 0, ..Fence::default() },
            ],
            ..Config::default()
        };
        let c: Config = serde_json::from_str::<Config>(&serde_json::to_string(&c).unwrap()).unwrap().sanitized();
        assert_eq!(c.fences.len(), 1);
        let f = &c.fences[0];
        assert_eq!((f.w, f.look.alpha, f.look.color, f.active().title.as_str()), (Fence::MIN.0, 64, 0x12_3456, "ab"));
        assert_eq!((f.active().id, f.title.as_str()), (3, ""));
        assert_eq!(c.next_id(), 4);
        assert_eq!(Config::default().next_id(), 1);
    }

    #[test]
    fn opacity_steps() {
        assert_eq!((Look::alpha_for(30), Look::alpha_for(100), Look::alpha_for(500)), (76, 255, 255));
    }

    #[test]
    fn legacy_json_migrates() {
        let c: Config = serde_json::from_str::<Config>(r#"{"fences":[{"id":1,"title":"Jeux","color":255,"icon":50}],"roll_ms":999}"#)
            .unwrap()
            .sanitized();
        let f = &c.fences[0];
        assert!(c.auto_update && c.autostart && c.auto_sort);
        assert_eq!((f.w, f.look.color, f.look.icon, f.tabs.len(), f.active().title.as_str()), (360, 255, 48, 1, "Jeux"));
        assert_eq!(c.roll_ms, 500);
    }

    #[test]
    fn rules_pick_first_matching_tab() {
        let mut a = Fence::new(1, "A", (0, 0, 200, 200), Look::default());
        a.tabs.push(Tab { id: 5, kinds: vec![Kind::Images], ..Tab::default() });
        let mut b = Fence::new(2, "B", (0, 0, 200, 200), Look::default());
        b.tabs[0].kinds = vec![Kind::Images, Kind::Apps];
        b.tabs.push(Tab { id: 6, portal: Some("C:\\x".into()), kinds: vec![Kind::Music], ..Tab::default() });
        let c = Config { fences: vec![a, b], ..Config::default() };
        assert_eq!(c.rule_target(Kind::Images), Some((1, 5)));
        assert_eq!(c.rule_target(Kind::Apps), Some((2, 2)));
        assert_eq!(c.rule_target(Kind::Music), None);
        assert_eq!(c.next_id(), 7);
    }
}
