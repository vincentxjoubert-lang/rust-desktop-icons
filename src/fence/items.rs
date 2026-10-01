use super::{TITLE, fence_of, loader, metrics, render, update};
use crate::{app::with, win::*};
use std::path::PathBuf;
use windows::Win32::Foundation::HWND;

pub fn reload(h: HWND) {
    if let Some(f) = fence_of(h) {
        loader::request(h, f.tabs);
    }
}

pub(super) fn refresh(h: HWND) {
    let dirty = with(|a| a.view(h).map(|v| std::mem::take(&mut v.dirty))).flatten().unwrap_or_default();
    if let Some(f) = fence_of(h) {
        loader::request(h, f.tabs.into_iter().enumerate().filter(|(i, _)| dirty.contains(i)).map(|(_, t)| t).collect());
    }
}

pub fn windows() -> Vec<HWND> {
    with(|a| a.views.iter().map(|v| v.hwnd).collect()).unwrap_or_default()
}

pub fn reload_all() {
    windows().into_iter().for_each(reload);
}

pub(super) fn set_icon(h: HWND, px: i32) {
    update(h, |f| f.look.icon = px);
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
