use super::{TITLE, metrics, render, update};
use crate::domain::{
    Tab,
    order::{self, Meta},
};
use crate::{
    app::{Item, with},
    shell, store,
    win::*,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use windows::Win32::Foundation::HWND;

fn meta(p: &Path) -> Meta {
    let md = fs::metadata(p).ok();
    let modified = md.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    Meta {
        name: p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        dir: md.is_some_and(|m| m.is_dir()),
        modified,
    }
}

fn load(tab: &Tab, px: i32) -> Vec<Item> {
    let paths: Vec<PathBuf> = fs::read_dir(store::tab_dir(tab))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| !p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("desktop.ini")))
        .collect();
    let metas: Vec<Meta> = paths.iter().map(|p| meta(p)).collect();
    order::arrange(&metas, tab.sort, &tab.order)
        .into_iter()
        .map(|i| paths[i].clone())
        .map(|path| {
            let (icon, name) = shell::info(&path, px);
            Item { path, name, icon }
        })
        .collect()
}

pub fn reload(h: HWND) {
    let Some(f) = with(|a| a.fence_of(h).cloned()).flatten() else { return };
    let px = metrics(h).1;
    let mut all: Vec<(u64, Vec<Item>)> = f.tabs.iter().map(|t| (t.id, load(t, px))).collect();
    let active = all.iter().position(|(id, _)| *id == f.active().id).map(|i| all.remove(i).1).unwrap_or_default();
    with(|a| {
        a.view(h).map(|v| {
            v.selected.retain(|p| active.iter().any(|i| &i.path == p));
            (v.items, v.cache) = (active, all);
        })
    });
    super::layout::apply(h);
}

pub fn reload_all() {
    with(|a| a.views.iter().map(|v| v.hwnd).collect::<Vec<_>>()).into_iter().flatten().for_each(reload);
}

pub(super) fn set_icon(h: HWND, px: i32) {
    update(h, |f| f.look.icon = px);
    reload(h);
}

pub(super) fn index_at(h: HWND, (x, y): (i32, i32)) -> Option<usize> {
    let (w, t, cell) = (client_rect(h).right, scale(h, TITLE), metrics(h).0);
    with(|a| {
        let v = a.view(h)?;
        (y >= t).then(|| crate::domain::grid::index_at((x, y - t - scale(h, 4) + v.scroll), w, cell, v.items.len()))?
    })
    .flatten()
}

pub(super) fn item_at(h: HWND, p: (i32, i32)) -> Option<PathBuf> {
    let i = index_at(h, p)?;
    with(|a| a.view(h)?.items.get(i).map(|it| it.path.clone())).flatten()
}

pub(super) fn hover(h: HWND, i: Option<usize>) {
    if with(|a| a.view(h).map(|v| std::mem::replace(&mut v.hover, i) != i)).flatten() == Some(true) {
        render(h);
    }
}
