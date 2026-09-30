use super::{arrange, fence_of, items, reload_all, tabs};
use crate::{shell, store};
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::HWND;

pub(super) fn files(h: HWND, paths: Vec<PathBuf>, pt: (i32, i32)) {
    let Some(f) = fence_of(h) else { return };
    let header = tabs::at(h, pt);
    let tab = header.and_then(|i| f.tabs.get(i)).unwrap_or(f.active());
    let (dir, fences, desks) = (store::tab_dir(tab), store::root().join("fences"), shell::desktops());
    let is = |q: Option<&Path>, d: &Path| q.is_some_and(|q| shell::same_path(q, d));
    if header.is_none() && !paths.is_empty() && paths.iter().all(|p| is(p.parent(), &dir)) {
        return arrange::move_within(h, &paths, items::item_at(h, pt));
    }
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
