use super::{TABS, WM_CHANGED, fence_of, layout};
use crate::{
    app::{Item, with},
    domain::{
        Tab,
        order::{self, Meta},
    },
    shell, store,
};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::UNIX_EPOCH,
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx},
    UI::WindowsAndMessaging::PostMessageW,
};

pub(super) const WM_LOADED: u32 = WM_CHANGED + TABS as u32;

struct Batch(Vec<(u64, u64, Vec<Item>)>);

fn meta(p: &Path) -> Meta {
    let md = fs::metadata(p).ok();
    let modified = md.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    Meta { name: store::name(p), dir: md.is_some_and(|m| m.is_dir()), modified }
}

fn load(tab: &Tab) -> Vec<Item> {
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
        .map(|i| {
            let (icon, name) = shell::info(&paths[i]);
            Item { path: paths[i].clone(), name, icon }
        })
        .collect()
}

pub(super) fn request(h: HWND, tabs: Vec<Tab>) {
    let Some(jobs) = with(|a| {
        let v = a.view(h)?;
        Some(
            tabs.into_iter()
                .map(|t| {
                    let g = match v.gens.iter_mut().find(|(id, _)| *id == t.id) {
                        Some((_, g)) => {
                            *g += 1;
                            *g
                        }
                        None => {
                            v.gens.push((t.id, 1));
                            1
                        }
                    };
                    (g, t)
                })
                .collect::<Vec<_>>(),
        )
    })
    .flatten() else {
        return;
    };
    let hwnd = h.0 as isize;
    thread::spawn(move || {
        let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let batch = Box::new(Batch(jobs.into_iter().map(|(g, t)| (t.id, g, load(&t))).collect()));
        let raw = Box::into_raw(batch);
        if unsafe { PostMessageW(Some(HWND(hwnd as _)), WM_LOADED, WPARAM(0), LPARAM(raw as isize)) }.is_err() {
            drop(unsafe { Box::from_raw(raw) });
        }
    });
}

pub(super) fn receive(h: HWND, lp: LPARAM) {
    let Batch(results) = *unsafe { Box::from_raw(lp.0 as *mut Batch) };
    let Some(active) = fence_of(h).map(|f| f.active().id) else { return };
    with(|a| {
        let v = a.view(h)?;
        for (id, g, items) in results {
            if !v.gens.iter().any(|&(t, cur)| t == id && cur == g) {
                continue;
            }
            if id == active {
                v.selected.retain(|p| items.iter().any(|i| &i.path == p));
                v.items = items;
            } else {
                v.cache.retain(|(t, _)| *t != id);
                v.cache.push((id, items));
            }
        }
        Some(())
    });
    layout::apply(h);
}
