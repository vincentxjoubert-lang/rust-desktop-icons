use crate::{
    app::with,
    domain::{Kind, kind::transient},
    fence, shell, store,
};
use std::{
    collections::HashSet,
    fs,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer, WM_APP},
};

pub const WM_DESK: u32 = WM_APP + 20;
pub const TIMER: usize = 7;
const OWN_SHORTCUT: &str = "Rust Desktop Icons.lnk";

fn entries(desk: &Path) -> HashSet<PathBuf> {
    fs::read_dir(desk).into_iter().flatten().flatten().map(|e| e.path()).collect()
}

fn movable(p: &Path) -> bool {
    let name = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let hidden = fs::metadata(p).is_ok_and(|m| m.file_attributes() & 0x6 != 0);
    !hidden && name != "desktop.ini" && name != OWN_SHORTCUT.to_lowercase() && !transient(p.extension().and_then(|e| e.to_str()))
}

pub fn watch(tray: HWND) {
    let Some(desk) = shell::desktop() else { return };
    let (seen, w) = (entries(&desk), shell::Watch::new(tray, &desk, WM_DESK));
    with(|a| (a.seen, a.desk) = (seen, w));
}

pub fn changed(tray: HWND) {
    unsafe { SetTimer(Some(tray), TIMER, 2000, None) };
}

pub fn ignore(p: PathBuf) {
    with(|a| a.seen.insert(p));
}

pub fn run(tray: Option<HWND>, all: bool) {
    if let Some(t) = tray {
        let _ = unsafe { KillTimer(Some(t), TIMER) };
    }
    let Some(desk) = shell::desktop() else { return };
    let Some((cfg, seen)) = with(|a| (a.cfg.clone(), a.seen.clone())) else { return };
    let mut moved = 0;
    if all || cfg.auto_sort {
        for p in entries(&desk).into_iter().filter(|p| (all || !seen.contains(p)) && movable(p)) {
            let kind = Kind::of(p.extension().and_then(|e| e.to_str()), p.is_dir());
            let tab = kind
                .and_then(|k| cfg.rule_target(k))
                .and_then(|(f, t)| cfg.fences.iter().find(|x| x.id == f)?.tabs.iter().find(|x| x.id == t));
            if let Some(tab) = tab {
                moved += store::move_into(&p, &store::tab_dir(tab)).is_ok() as usize;
            }
        }
    }
    let now = entries(&desk);
    with(|a| a.seen = now);
    if moved > 0 {
        fence::reload_all();
    }
}
