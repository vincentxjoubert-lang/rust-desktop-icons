use super::{TITLE, metrics, render, update};
use crate::{
    app::{Item, with},
    shell, store,
    win::*,
};
use std::{fs, path::PathBuf};
use windows::Win32::Foundation::HWND;

pub fn reload(h: HWND) {
    let Some(dir) = with(|a| a.fence_of(h).map(|f| store::tab_dir(f.active()))).flatten() else { return };
    let px = metrics(h).1;
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| !p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("desktop.ini")))
        .collect();
    paths.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    let items: Vec<Item> = paths
        .into_iter()
        .map(|path| {
            let (icon, name) = shell::info(&path, px);
            Item { path, name, icon }
        })
        .collect();
    with(|a| a.view(h).map(|v| v.items = items));
    render(h);
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
