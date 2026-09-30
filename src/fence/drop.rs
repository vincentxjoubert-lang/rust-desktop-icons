use super::{fence_of, reload_all, tabs};
use crate::{shell, store};
use std::path::{Path, PathBuf};
use windows::Win32::{
    Foundation::{HWND, POINT},
    UI::Shell::*,
};

fn paths(d: HDROP) -> (Vec<PathBuf>, (i32, i32)) {
    let mut pt = POINT::default();
    let _ = unsafe { DragQueryPoint(d, &mut pt) };
    let n = unsafe { DragQueryFileW(d, u32::MAX, None) };
    let paths = (0..n)
        .map(|i| {
            let mut b = vec![0u16; unsafe { DragQueryFileW(d, i, None) } as usize + 1];
            let len = unsafe { DragQueryFileW(d, i, Some(&mut b)) } as usize;
            PathBuf::from(String::from_utf16_lossy(&b[..len]))
        })
        .collect();
    unsafe { DragFinish(d) };
    (paths, (pt.x, pt.y))
}

pub(super) fn files(h: HWND, d: HDROP) {
    let (paths, pt) = paths(d);
    let Some(f) = fence_of(h) else { return };
    let tab = tabs::at(h, pt).and_then(|i| f.tabs.get(i)).unwrap_or(f.active());
    let (dir, fences, desks) = (store::tab_dir(tab), store::root().join("fences"), shell::desktops());
    let is = |q: Option<&Path>, d: &Path| q.is_some_and(|q| shell::same_path(q, d));
    for p in paths.iter().filter(|p| !is(p.parent(), &dir)) {
        let own = desks.first().is_some_and(|d| is(p.parent(), d)) || is(p.parent().and_then(|q| q.parent()), &fences);
        let public = desks.get(1).is_some_and(|d| is(p.parent(), d));
        if own {
            let _ = store::move_into(p, &dir);
        } else if !(public && store::move_into(p, &dir).is_ok()) {
            let _ = shell::link_into(p, &dir);
        }
    }
    reload_all();
}
