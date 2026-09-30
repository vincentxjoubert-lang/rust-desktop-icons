use super::{reload, update};
use crate::{
    app::{App, with},
    domain::{Kind, Sort, Tab, order},
    i18n::{self, T},
    layered::glyph as g,
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
    with(|a| a.view(h).map(|v| v.items.iter().filter_map(|i| i.path.file_name()).map(|n| n.to_string_lossy().into_owned()).collect()))
        .flatten()
        .unwrap_or_default()
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
            update(h, |f| {
                let kinds = &mut f.active_mut().kinds;
                if kinds.contains(&kind) {
                    kinds.retain(|x| *x != kind);
                } else {
                    kinds.push(kind);
                }
            });
        }
        _ => return false,
    }
    true
}

pub(super) fn move_within(h: HWND, moved: &[PathBuf], before: Option<PathBuf>) {
    let name = |p: &PathBuf| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let moved: Vec<String> = moved.iter().map(name).collect();
    let before = before.as_ref().map(name);
    set_order(h, Sort::Custom, order::reorder(&names(h), &moved, before.as_deref()));
}
