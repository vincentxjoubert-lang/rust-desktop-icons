use super::{TITLE, ghost, items, metrics, recycle, render, view};
use crate::{app::with, domain::grid, rules, shell, win::*};
use std::{collections::HashSet, path::PathBuf};
use windows::Win32::{
    Foundation::*,
    Graphics::Gdi::ClientToScreen,
    UI::Input::KeyboardAndMouse::{DragDetect, ReleaseCapture, SetCapture},
};

const GAP: i32 = 4;

fn content(h: HWND, (x, y): (i32, i32)) -> (i32, i32) {
    let scroll = view(h, |v| v.scroll).unwrap_or(0);
    (x, y - scale(h, TITLE) - scale(h, GAP) + scroll)
}

pub(super) fn all(h: HWND) {
    let paths = view(h, |v| v.items.iter().map(|i| i.path.clone()).collect()).unwrap_or_default();
    replace(h, paths);
}

pub(super) fn replace(h: HWND, paths: HashSet<PathBuf>) {
    view(h, |v| v.selected = paths);
    render(h);
}

pub(super) fn selection(h: HWND) -> Vec<PathBuf> {
    view(h, |v| v.items.iter().filter(|i| v.selected.contains(&i.path)).map(|i| i.path.clone()).collect()).unwrap_or_default()
}

pub(super) fn focus(h: HWND, p: &PathBuf) -> Vec<PathBuf> {
    if !view(h, |v| v.selected.contains(p)).unwrap_or(false) {
        replace(h, HashSet::from([p.clone()]));
    }
    selection(h)
}

fn drag(h: HWND, paths: &[PathBuf], at: (i32, i32)) {
    if paths.iter().any(|p| shell::recycle::is(p)) {
        return recycle::drag(h, at, content(h, at));
    }
    if let Some(desk) = shell::desktop() {
        paths.iter().filter_map(|p| p.file_name()).for_each(|n| rules::ignore(desk.join(n)));
    }
    let image = items::index_at(h, at).and_then(|i| ghost::image(h, i, content(h, at)));
    shell::drag_out(h, paths, image, false);
}

pub(super) fn down(h: HWND, p: (i32, i32), ctrl: bool) {
    let Some(path) = items::item_at(h, p) else {
        let c = content(h, p);
        with(|a| {
            a.view(h).map(|v| {
                v.band = Some((c, c));
                if !ctrl {
                    v.selected.clear();
                }
            })
        });
        unsafe { SetCapture(h) };
        return render(h);
    };
    if ctrl {
        view(h, |v| v.selected.insert(path.clone()) || v.selected.remove(&path));
        return render(h);
    }
    let paths = focus(h, &path);
    let mut pt = POINT { x: p.0, y: p.1 };
    let _ = unsafe { ClientToScreen(h, &mut pt) };
    if unsafe { DragDetect(h, pt) }.as_bool() {
        drag(h, &paths, p);
    } else {
        replace(h, HashSet::from([path]));
    }
}

pub(super) fn track(h: HWND, p: (i32, i32)) -> bool {
    let c = content(h, p);
    let (w, cell) = (client_rect(h).right, metrics(h).0);
    let done = with(|a| {
        let v = a.view(h)?;
        let (start, _) = v.band?;
        v.band = Some((start, c));
        let hit = grid::in_rect(v.items.len(), w, cell, (start.0, start.1, c.0, c.1));
        v.selected = hit.into_iter().map(|i| v.items[i].path.clone()).collect();
        Some(())
    });
    let active = done.flatten().is_some();
    if active {
        render(h);
    }
    active
}

pub(super) fn up(h: HWND) {
    if with(|a| a.view(h).and_then(|v| v.band.take())).flatten().is_some() {
        let _ = unsafe { ReleaseCapture() };
        render(h);
    }
}
