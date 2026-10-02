use super::{
    anim,
    kind::Kind,
    model::{Fence, Look, Tab},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
    pub version: Option<String>,
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
            version: None,
        }
    }
}

impl Config {
    pub fn next_id(&self) -> u64 {
        self.fences.iter().flat_map(|f| f.tabs.iter().map(|t| t.id).chain([f.id])).max().map_or(1, |m| m + 1)
    }

    pub fn has_recycle(&self) -> bool {
        self.fences.iter().any(|f| f.tabs.iter().any(|t| t.recycle))
    }

    pub fn place_recycle(&mut self, tab: Option<u64>) {
        self.fences.iter_mut().flat_map(|f| f.tabs.iter_mut()).for_each(|t| t.recycle = Some(t.id) == tab);
    }

    pub fn tab(&self, fence: u64, tab: u64) -> Option<&Tab> {
        self.fences.iter().find(|f| f.id == fence)?.tabs.iter().find(|t| t.id == tab)
    }

    pub fn tab_mut(&mut self, fence: u64, tab: u64) -> Option<&mut Tab> {
        self.fences.iter_mut().find(|f| f.id == fence)?.tabs.iter_mut().find(|t| t.id == tab)
    }

    pub fn toggle_rule(&mut self, fence: u64, tab: u64, kind: Kind) {
        let Some(t) = self.tab_mut(fence, tab) else { return };
        if t.kinds.contains(&kind) {
            t.kinds.retain(|x| *x != kind);
        } else {
            t.kinds.push(kind);
            self.auto_sort = true;
        }
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
        let first = self.fences.iter().flat_map(|f| &f.tabs).find(|t| t.recycle).map(|t| t.id);
        self.place_recycle(first);
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
    fn legacy_json_migrates() {
        let c: Config = serde_json::from_str::<Config>(r#"{"fences":[{"id":1,"title":"Jeux","color":255,"icon":50}],"roll_ms":999}"#)
            .unwrap()
            .sanitized();
        let f = &c.fences[0];
        assert!(c.auto_update && c.autostart && c.auto_sort && f.look.auto_height);
        assert_eq!((f.w, f.look.color, f.look.icon, f.tabs.len(), f.active().title.as_str()), (360, 255, 48, 1, "Jeux"));
        assert_eq!(c.roll_ms, 500);
    }

    #[test]
    fn recycle_bin_lives_in_one_tab() {
        let mut a = Fence::new(1, "A", (0, 0, 200, 200), Look::default());
        a.tabs.push(Tab { id: 5, recycle: true, ..Tab::default() });
        let mut b = Fence::new(2, "B", (0, 0, 200, 200), Look::default());
        b.tabs[0].recycle = true;
        let mut c = Config { fences: vec![a, b], ..Config::default() }.sanitized();
        let holders: Vec<u64> = c.fences.iter().flat_map(|f| &f.tabs).filter(|t| t.recycle).map(|t| t.id).collect();
        assert_eq!(holders, vec![5]);
        c.place_recycle(Some(2));
        assert!(c.fences[1].tabs[0].recycle && !c.fences[0].tabs[1].recycle && c.has_recycle());
        c.place_recycle(None);
        assert!(!c.has_recycle());
    }

    #[test]
    fn rules_pick_first_matching_tab() {
        let mut a = Fence::new(1, "A", (0, 0, 200, 200), Look::default());
        a.tabs.push(Tab { id: 5, kinds: vec![Kind::Images], ..Tab::default() });
        let mut b = Fence::new(2, "B", (0, 0, 200, 200), Look::default());
        b.tabs[0].kinds = vec![Kind::Images, Kind::Apps];
        b.tabs.push(Tab { id: 6, portal: Some("C:\\x".into()), kinds: vec![Kind::Music], ..Tab::default() });
        let mut c = Config { fences: vec![a, b], ..Config::default() };
        assert_eq!(c.rule_target(Kind::Images), Some((1, 5)));
        assert_eq!(c.rule_target(Kind::Apps), Some((2, 2)));
        assert_eq!(c.rule_target(Kind::Music), None);
        c.auto_sort = false;
        c.toggle_rule(1, 5, Kind::Music);
        assert!(c.auto_sort && c.tab(1, 5).unwrap().kinds.contains(&Kind::Music));
        c.auto_sort = false;
        c.toggle_rule(1, 5, Kind::Music);
        assert!(!c.auto_sort && !c.tab(1, 5).unwrap().kinds.contains(&Kind::Music));
        assert_eq!(c.next_id(), 7);
    }
}
