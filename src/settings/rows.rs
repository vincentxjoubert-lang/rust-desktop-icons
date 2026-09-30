use crate::{
    app::App,
    domain::{Look, icons},
    i18n::{self, T},
    layered::glyph as g,
    prefs,
};

pub const TOP: i32 = 56;
pub const WIDTH: i32 = 460;
const HEADER: i32 = 46;
const ROW: i32 = 44;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Color,
    Opacity,
    IconSize,
    Tint,
    Chameleon,
    AutoHeight,
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
    pub glyph: char,
}

fn row(label: &str, ctl: Ctl, act: Act, glyph: char) -> Row {
    Row { label: label.trim_end_matches('…').into(), ctl, act: Some(act), glyph }
}

fn header(label: &str) -> Row {
    Row { label: label.into(), ctl: Ctl::Header, act: None, glyph: ' ' }
}

pub fn speed_label(a: &App, ms: u32) -> String {
    if ms == 0 { a.t(T::Instant).into() } else { format!("{ms} ms") }
}

pub fn build(a: &App) -> Vec<Row> {
    let (c, l) = (&a.cfg, &a.cfg.look);
    let mut rows = vec![
        header(a.t(T::Appearance)),
        row(a.t(T::Color), Ctl::Swatch(Some(l.color)), Act::Color, g::COLOR),
        row(a.t(T::Opacity), Ctl::Value(Look::label(l.percent())), Act::Opacity, g::OPACITY),
        row(a.t(T::IconSize), Ctl::Value(icons::label(l.icon)), Act::IconSize, g::SIZE),
        row(a.t(T::Tint), Ctl::Swatch(l.tint), Act::Tint, g::TINT),
        row(a.t(T::ApplyAll), Ctl::Button, Act::ApplyAll, g::CHECK),
        header(a.t(T::Behavior)),
        row(a.t(T::Chameleon), Ctl::Toggle(l.chameleon), Act::Chameleon, g::EYE),
        row(a.t(T::AutoHeight), Ctl::Toggle(l.auto_height), Act::AutoHeight, g::HEIGHT),
        row(a.t(T::RollSpeed), Ctl::Value(speed_label(a, c.roll_ms)), Act::Speed, g::SPEED),
        header(a.t(T::Rules)),
        row(a.t(T::AutoSort), Ctl::Toggle(c.auto_sort), Act::AutoSort, g::SYNC),
    ];
    for f in &c.fences {
        for t in f.tabs.iter().filter(|t| t.portal.is_none()) {
            let name = if t.id == f.tabs[0].id { t.title.clone() } else { format!("{} › {}", f.tabs[0].title, t.title) };
            let kinds: Vec<&str> = t.kinds.iter().map(|k| a.t(i18n::kind(*k))).collect();
            let value = if kinds.is_empty() { a.t(T::None).into() } else { kinds.join(", ") };
            rows.push(row(&name, Ctl::Value(value), Act::Rule(f.id, t.id), g::FOLDER));
        }
    }
    rows.extend([
        row(a.t(T::SortNow), Ctl::Button, Act::SortNow, g::SORT),
        header(a.t(T::General)),
        row(a.t(T::Language), Ctl::Value(prefs::lang_name(a).into()), Act::Lang, g::GLOBE),
        row(a.t(T::Autostart), Ctl::Toggle(c.autostart), Act::Autostart, g::POWER),
        row(a.t(T::AutoUpdate), Ctl::Toggle(c.auto_update), Act::AutoUpdate, g::DOWNLOAD),
        row(a.t(T::CheckUpdates), Ctl::Value(format!("v{}", env!("CARGO_PKG_VERSION"))), Act::Check, g::REFRESH),
    ]);
    rows
}

fn height(r: &Row) -> i32 {
    if r.ctl == Ctl::Header { HEADER } else { ROW }
}

pub fn layout(rows: &[Row]) -> Vec<(i32, i32)> {
    rows.iter().scan(0, |y, r| Some((std::mem::replace(y, *y + height(r)), height(r)))).collect()
}

pub fn cards(rows: &[Row]) -> Vec<(usize, usize)> {
    let mut out = vec![];
    for (i, r) in rows.iter().enumerate() {
        match (r.ctl == Ctl::Header, out.last_mut()) {
            (true, _) => {}
            (false, Some((_, end))) if *end + 1 == i => *end = i,
            _ => out.push((i, i)),
        }
    }
    out
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
        let rows = vec![header("A"), row("b", Ctl::Button, Act::Check, 'x'), row("c", Ctl::Toggle(true), Act::AutoSort, 'x')];
        assert_eq!(layout(&rows), vec![(0, HEADER), (HEADER, ROW), (HEADER + ROW, ROW)]);
        assert_eq!(total(&rows), HEADER + 2 * ROW);
        assert_eq!(at(&rows, 5), None);
        assert_eq!(at(&rows, HEADER + 1), Some(1));
        assert_eq!(at(&rows, HEADER + ROW + ROW - 1), Some(2));
        assert_eq!(at(&rows, total(&rows)), None);
    }

    #[test]
    fn card_groups() {
        let r = |c| row("r", c, Act::Check, 'x');
        let rows = vec![header("A"), r(Ctl::Button), r(Ctl::Button), header("B"), r(Ctl::Toggle(false))];
        assert_eq!(cards(&rows), vec![(1, 2), (4, 4)]);
    }
}
