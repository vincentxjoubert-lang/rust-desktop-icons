use crate::{
    app::with,
    domain::kind::{candidates, transient},
    fence,
    i18n::T,
    report, shell, store,
};
use std::{
    collections::HashSet,
    fs,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Duration,
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer, WM_APP},
};

pub const WM_DESK: u32 = WM_APP + 20;
pub const TIMER: usize = 7;
const OWN_SHORTCUT: &str = "Rust Desktop Icons.lnk";
const SETTLE: Duration = Duration::from_secs(10);
const RETRY_MS: u32 = 3000;

fn settled(p: &Path) -> bool {
    let Ok(m) = fs::metadata(p) else { return false };
    [m.modified(), m.created()].into_iter().flatten().filter_map(|t| t.elapsed().ok()).all(|age| age >= SETTLE)
}

fn entries(desk: &Path) -> HashSet<PathBuf> {
    fs::read_dir(desk).into_iter().flatten().flatten().map(|e| e.path()).collect()
}

fn movable(p: &Path) -> bool {
    let name = store::name(p).to_lowercase();
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
    let (mut moved, mut errors, mut pending) = (0, vec![], HashSet::new());
    if all || cfg.auto_sort {
        for p in entries(&desk).into_iter().filter(|p| (all || !seen.contains(p)) && movable(p)) {
            if !all && !settled(&p) {
                pending.insert(p);
                continue;
            }
            let target = shell::link_target(&p);
            let kinds = candidates(p.extension().and_then(|e| e.to_str()), p.is_dir(), target.as_deref());
            let tab = kinds.into_iter().find_map(|k| cfg.rule_target(k)).and_then(|(f, t)| cfg.tab(f, t));
            if let Some(tab) = tab {
                match store::move_into(&p, &store::tab_dir(tab)) {
                    Ok(_) => moved += 1,
                    Err(e) => errors.push(format!("{} ({e})", store::name(&p))),
                }
            }
        }
    }
    let now = entries(&desk).into_iter().filter(|p| !pending.contains(p)).collect();
    with(|a| a.seen = now);
    if let (Some(t), false) = (tray, pending.is_empty()) {
        unsafe { SetTimer(Some(t), TIMER, RETRY_MS, None) };
    }
    if moved > 0 {
        fence::reload_all();
    }
    report::failures(T::ErrMove, &errors);
}
