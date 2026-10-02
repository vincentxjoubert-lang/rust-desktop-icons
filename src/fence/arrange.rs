use super::{read, reload, update, view};
use crate::{
    app::{App, change},
    domain::{Kind, Sort, Tab, order},
    i18n::{self, T},
    layered::glyph as g,
    store,
    win::*,
};
use std::path::PathBuf;
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::HMENU};

const SORT: usize = 70;
const RULE: usize = 80;

pub(super) fn submenus(a: &mut App, m: HMENU, t: &Tab) {
    let sorts = menu();
    for (k, s) in Sort::ALL.iter().enumerate() {
        item(sorts, SORT + k, a.t(i18n::sort(*s)), t.sort == *s);
    }
    a.sub(m, T::SortBy, sorts, g::SORT);
    if t.portal.is_none() {
        let rules = menu();
        for (k, kind) in Kind::ALL.iter().enumerate() {
            item(rules, RULE + k, a.t(i18n::kind(*kind)), t.kinds.contains(kind));
        }
        a.sub(m, T::Rules, rules, g::SYNC);
    }
}

fn names(h: HWND) -> Vec<String> {
    view(h, |v| v.items.iter().map(|i| store::name(&i.path)).collect()).unwrap_or_default()
}

fn set_order(h: HWND, sort: Sort, order: Vec<String>) {
    update(h, |f| {
        let t = f.active_mut();
        (t.sort, t.order) = (sort, order);
    });
    reload(h);
}

pub(super) fn handle(h: HWND, id: usize) -> bool {
    match id {
        k if (SORT..SORT + Sort::ALL.len()).contains(&k) => set_order(h, Sort::ALL[k - SORT], names(h)),
        k if (RULE..RULE + Kind::ALL.len()).contains(&k) => {
            let kind = Kind::ALL[k - RULE];
            if let Some((fence, tab)) = read(h, |f| (f.id, f.active().id)) {
                change(|c| c.toggle_rule(fence, tab, kind));
            }
        }
        _ => return false,
    }
    true
}

pub(super) fn move_within(h: HWND, moved: &[PathBuf], before: Option<PathBuf>) {
    let moved: Vec<String> = moved.iter().map(|p| store::name(p)).collect();
    let before = before.as_deref().map(store::name);
    set_order(h, Sort::Custom, order::reorder(&names(h), &moved, before.as_deref()));
}
