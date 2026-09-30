use super::{TITLE, WM_CHANGED, actions, fence_of, layout, reload, to_client, update};
use crate::{
    app::with,
    domain::{Tab, grid},
    i18n::T,
    shell, store,
    win::*,
};
use std::path::PathBuf;
use windows::Win32::{Foundation::*, UI::Input::KeyboardAndMouse::DragDetect};

pub fn bind(h: HWND) {
    let Some(dirs) = with(|a| a.fence_of(h).map(|f| f.tabs.iter().map(store::tab_dir).collect::<Vec<_>>())).flatten() else { return };
    with(|a| a.view(h).map(|v| v.watches.clear()));
    let watches = dirs.iter().filter_map(|d| shell::Watch::new(h, d, WM_CHANGED)).collect();
    with(|a| a.view(h).map(|v| (v.watches, v.scroll) = (watches, 0)));
    reload(h);
}

pub(super) fn at(h: HWND, (x, y): (i32, i32)) -> Option<usize> {
    let n = fence_of(h)?.tabs.len();
    (n > 1 && (0..scale(h, TITLE)).contains(&y)).then(|| grid::split(client_rect(h).right, n, x))?
}

pub(super) fn click(h: HWND, screen: (i32, i32)) -> bool {
    let Some(i) = at(h, to_client(h, screen)) else { return false };
    if unsafe { DragDetect(h, POINT { x: screen.0, y: screen.1 }) }.as_bool() {
        return false;
    }
    select(h, i);
    true
}

pub(super) fn hover(h: HWND, screen: (i32, i32)) {
    if let Some(i) = at(h, to_client(h, screen)) {
        select(h, i);
    }
}

pub(super) fn select(h: HWND, i: usize) {
    let hit = with(|a| {
        let f = a.fence_of(h)?;
        if f.tab == i || i >= f.tabs.len() {
            return Some(true);
        }
        let (old, new) = (f.active().id, f.tabs[i].id);
        f.tab = i;
        a.save();
        let v = a.view(h)?;
        let items = std::mem::take(&mut v.items);
        v.cache.push((old, items));
        v.scroll = 0;
        let found = v.cache.iter().position(|(id, _)| *id == new).map(|k| v.cache.remove(k).1);
        let hit = found.is_some();
        v.items = found.unwrap_or_default();
        Some(hit)
    })
    .flatten();
    match hit {
        Some(true) => layout::apply(h),
        Some(false) => reload(h),
        None => {}
    }
}

fn add(h: HWND, portal: Option<PathBuf>) {
    let title = portal.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned());
    with(|a| {
        let (id, name) = (a.cfg.next_id(), a.t(T::NewTab));
        let f = a.fence_of(h)?;
        f.tabs.push(Tab { id, title: title.unwrap_or_else(|| name.into()), portal, ..Tab::default() });
        f.tab = f.tabs.len() - 1;
        let _ = std::fs::create_dir_all(store::tab_dir(f.active()));
        a.save();
        Some(())
    });
    update(h, |_| {});
    bind(h);
}

pub(super) fn new_tab(h: HWND) {
    add(h, None);
}

pub(super) fn new_portal(h: HWND) {
    if let Some(dir) = shell::pick_folder(h) {
        add(h, Some(dir));
    }
}

pub(super) fn remove(h: HWND) {
    let Some(f) = fence_of(h).filter(|f| f.tabs.len() > 1) else { return };
    let tab = f.active().clone();
    if tab.portal.is_none() && !actions::confirm(h, T::ConfirmDeleteTab) {
        return;
    }
    if tab.portal.is_none() && !actions::evacuate(&store::tab_dir(&tab)) {
        return reload(h);
    }
    update(h, |f| {
        f.tabs.retain(|t| t.id != tab.id);
        f.tab = f.tab.min(f.tabs.len() - 1);
    });
    bind(h);
}
