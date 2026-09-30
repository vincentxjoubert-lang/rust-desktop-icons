use super::{actions, arrange, cursor, fence_of, items, peek, reload, select, tabs, title, update};
use crate::{
    app::with,
    domain::{Look, icons},
    i18n::T,
    layered::glyph as g,
    settings, shell, store,
    win::*,
};
use windows::Win32::Foundation::HWND;

pub(super) fn context(h: HWND) {
    let p = cursor(h);
    if let Some(i) = tabs::at(h, p) {
        tabs::select(h, i);
    }
    let targets = items::item_at(h, p).map(|t| select::focus(h, &t)).unwrap_or_default();
    let Some(f) = fence_of(h) else { return };
    let Some((m, rtl)) = with(|a| {
        let m = menu();
        if !targets.is_empty() {
            a.entry(m, 30, T::Open, false, g::OPEN);
            a.entry(m, 31, T::Restore, false, g::DESKTOP);
            a.entry(m, 32, T::Delete, false, g::DELETE);
            separator(m);
        }
        let (o, sizes, tint) = (menu(), menu(), menu());
        for (k, &pct) in Look::PERCENTS.iter().enumerate() {
            item(o, 20 + k, &format!("{pct}%"), f.look.alpha == Look::alpha_for(pct));
        }
        for (k, &px) in icons::SIZES.iter().enumerate() {
            item(sizes, 40 + k, &format!("{px} px"), f.look.icon == px);
        }
        item(tint, 50, a.t(T::None), f.look.tint.is_none());
        item(tint, 51, a.t(T::Color), f.look.tint.is_some());
        a.entry(m, 10, T::Rename, false, g::RENAME);
        a.entry(m, 11, T::Color, false, g::COLOR);
        a.sub(m, T::Opacity, o, g::OPACITY);
        a.sub(m, T::IconSize, sizes, g::SIZE);
        a.sub(m, T::Tint, tint, g::TINT);
        arrange::submenus(a, m, f.active());
        a.entry(m, 16, T::Chameleon, f.look.chameleon, g::EYE);
        a.entry(m, 12, T::Roll, f.rolled, g::ROLL);
        separator(m);
        a.entry(m, 60, T::NewTab, false, g::TAB);
        a.entry(m, 61, T::NewPortal, false, g::PORTAL);
        if f.tabs.len() > 1 {
            a.entry(m, 62, T::DeleteTab, false, g::CLOSE);
        }
        a.entry(m, 13, T::OpenFolder, false, g::FOLDER);
        separator(m);
        a.entry(m, 14, T::DeleteFence, false, g::DELETE);
        a.entry(m, 15, T::NewFence, false, g::ADD);
        a.entry(m, 17, T::Settings, false, g::SETTINGS);
        (m, a.rtl())
    }) else {
        return;
    };
    let id = popup(h, m, rtl);
    if arrange::handle(h, id) {
        return;
    }
    match id {
        10 => title::rename(h),
        11 => actions::color(h),
        12 => peek::toggle(h),
        13 => shell::open(&store::tab_dir(f.active())),
        14 => actions::delete(h),
        15 => crate::app::new_fence(),
        16 => {
            update(h, |f| f.look.chameleon ^= true);
            super::anim::start(h);
        }
        17 => settings::open(),
        k @ 20..=27 => update(h, |f| f.look.alpha = Look::alpha_for(Look::PERCENTS[k - 20])),
        k @ 40..=47 => items::set_icon(h, icons::SIZES[k - 40]),
        50 => update(h, |f| f.look.tint = None),
        51 => actions::tint(h),
        60 => tabs::new_tab(h),
        61 => tabs::new_portal(h),
        62 => tabs::remove(h),
        30 => targets.iter().for_each(|p| shell::open(p)),
        31 => {
            targets.iter().for_each(|p| actions::restore(p));
            reload(h);
        }
        32 => {
            targets.iter().for_each(|p| {
                shell::recycle(p);
            });
            reload(h);
        }
        _ => {}
    }
}
