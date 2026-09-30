use crate::{app::App, domain::Kind, i18n::T, prefs};

pub const TOP: i32 = 52;
pub const WIDTH: i32 = 440;
const HEADER: i32 = 36;
const ROW: i32 = 38;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Color,
    Opacity,
    IconSize,
    Tint,
    Chameleon,
    Speed,
    ApplyAll,
    AutoSort,
    Rule(u64, u64),
    SortNow,
    Lang,
    Autostart,
    AutoUpdate,
    Check,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ctl {
    Header,
    Toggle(bool),
    Value(String),
    Swatch(Option<u32>),
    Button,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub label: String,
    pub ctl: Ctl,
    pub act: Option<Act>,
}

fn row(label: &str, ctl: Ctl, act: Act) -> Row {
    Row { label: label.trim_end_matches('…').into(), ctl, act: Some(act) }
}

fn header(label: &str) -> Row {
    Row { label: label.into(), ctl: Ctl::Header, act: None }
}

pub fn kind_label(k: Kind) -> T {
    match k {
        Kind::Apps => T::KApps,
        Kind::Images => T::KImages,
        Kind::Documents => T::KDocuments,
        Kind::Videos => T::KVideos,
        Kind::Music => T::KMusic,
        Kind::Archives => T::KArchives,
        Kind::Folders => T::KFolders,
    }
}

pub fn speed_label(a: &App, ms: u32) -> String {
    if ms == 0 { a.t(T::Instant).into() } else { format!("{ms} ms") }
}

pub fn build(a: &App) -> Vec<Row> {
    let (c, l) = (&a.cfg, &a.cfg.look);
    let mut rows = vec![
        header(a.t(T::Appearance)),
        row(a.t(T::Color), Ctl::Swatch(Some(l.color)), Act::Color),
        row(a.t(T::Opacity), Ctl::Value(format!("{}%", (l.alpha as u32 * 100 + 127) / 255)), Act::Opacity),
        row(a.t(T::IconSize), Ctl::Value(format!("{} px", l.icon)), Act::IconSize),
        row(a.t(T::Tint), Ctl::Swatch(l.tint), Act::Tint),
        row(a.t(T::Chameleon), Ctl::Toggle(l.chameleon), Act::Chameleon),
        row(a.t(T::RollSpeed), Ctl::Value(speed_label(a, c.roll_ms)), Act::Speed),
        row(a.t(T::ApplyAll), Ctl::Button, Act::ApplyAll),
        header(a.t(T::Rules)),
        row(a.t(T::AutoSort), Ctl::Toggle(c.auto_sort), Act::AutoSort),
    ];
    for f in &c.fences {
        for t in f.tabs.iter().filter(|t| t.portal.is_none()) {
            let name = if t.id == f.tabs[0].id { t.title.clone() } else { format!("{} › {}", f.tabs[0].title, t.title) };
            let kinds: Vec<&str> = t.kinds.iter().map(|k| a.t(kind_label(*k))).collect();
            let value = if kinds.is_empty() { a.t(T::None).into() } else { kinds.join(", ") };
            rows.push(row(&name, Ctl::Value(value), Act::Rule(f.id, t.id)));
        }
    }
    rows.extend([
        row(a.t(T::SortNow), Ctl::Button, Act::SortNow),
        header(a.t(T::General)),
        row(a.t(T::Language), Ctl::Value(prefs::lang_name(a).into()), Act::Lang),
        row(a.t(T::Autostart), Ctl::Toggle(c.autostart), Act::Autostart),
        row(a.t(T::AutoUpdate), Ctl::Toggle(c.auto_update), Act::AutoUpdate),
        row(a.t(T::CheckUpdates), Ctl::Button, Act::Check),
    ]);
    rows
}

fn height(r: &Row) -> i32 {
    if r.ctl == Ctl::Header { HEADER } else { ROW }
}

pub fn layout(rows: &[Row]) -> Vec<(i32, i32)> {
    rows.iter().scan(0, |y, r| Some((std::mem::replace(y, *y + height(r)), height(r)))).collect()
}

pub fn total(rows: &[Row]) -> i32 {
    rows.iter().map(height).sum()
}

pub fn at(rows: &[Row], y: i32) -> Option<usize> {
    layout(rows).iter().position(|&(top, h)| (top..top + h).contains(&y)).filter(|&i| rows[i].act.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_layout() {
        let rows = vec![header("A"), row("b", Ctl::Button, Act::Check), row("c", Ctl::Toggle(true), Act::AutoSort)];
        assert_eq!(layout(&rows), vec![(0, HEADER), (HEADER, ROW), (HEADER + ROW, ROW)]);
        assert_eq!(total(&rows), HEADER + 2 * ROW);
        assert_eq!(at(&rows, 5), None);
        assert_eq!(at(&rows, HEADER + 1), Some(1));
        assert_eq!(at(&rows, HEADER + ROW + ROW - 1), Some(2));
        assert_eq!(at(&rows, total(&rows)), None);
    }
}
