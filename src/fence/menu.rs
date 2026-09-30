use super::{actions, cursor, fence_of, items, peek, reload, tabs, title, update};
use crate::{
    app::with,
    domain::{Look, icons},
    i18n::T,
    settings, shell, store,
    win::*,
};
use windows::Win32::Foundation::HWND;

pub(super) fn context(h: HWND) {
    let p = cursor(h);
    if let Some(i) = tabs::at(h, p) {
        tabs::select(h, i);
    }
    let target = items::item_at(h, p);
    let Some(f) = fence_of(h) else { return };
    let Some((m, rtl)) = with(|a| {
        let m = menu();
        if target.is_some() {
            item(m, 30, a.t(T::Open), false);
            item(m, 31, a.t(T::Restore), false);
            item(m, 32, a.t(T::Delete), false);
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
        item(m, 10, a.t(T::Rename), false);
        item(m, 11, a.t(T::Color), false);
        submenu(m, a.t(T::Opacity), o);
        submenu(m, a.t(T::IconSize), sizes);
        submenu(m, a.t(T::Tint), tint);
        item(m, 16, a.t(T::Chameleon), f.look.chameleon);
        item(m, 12, a.t(T::Roll), f.rolled);
        separator(m);
        item(m, 60, a.t(T::NewTab), false);
        item(m, 61, a.t(T::NewPortal), false);
        if f.tabs.len() > 1 {
            item(m, 62, a.t(T::DeleteTab), false);
        }
        item(m, 13, a.t(T::OpenFolder), false);
        separator(m);
        item(m, 14, a.t(T::DeleteFence), false);
        item(m, 15, a.t(T::NewFence), false);
        item(m, 17, a.t(T::Settings), false);
        (m, a.rtl())
    }) else {
        return;
    };
    match popup(h, m, rtl) {
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
        30 => target.iter().for_each(|p| shell::open(p)),
        31 => target.iter().for_each(|p| actions::restore_item(h, p)),
        32 if target.is_some_and(|p| shell::recycle(&p)) => reload(h),
        _ => {}
    }
}
