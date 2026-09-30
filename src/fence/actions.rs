use super::{fence_of, reload, update};
use crate::{app::with, i18n::T, rules, shell, store, win::*};
use std::{fs, path::Path};
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

pub(super) fn confirm(h: HWND, q: T) -> bool {
    with(|a| (a.t(q), a.rtl())).is_some_and(|(q, rtl)| msgbox(Some(h), q, MB_YESNO | MB_ICONQUESTION, rtl) == IDYES)
}

pub(super) fn restore(p: &Path) {
    if let Some(moved) = shell::desktop().and_then(|d| store::move_into(p, &d).ok()) {
        rules::ignore(moved);
    }
}

pub(super) fn evacuate(dir: &Path) -> bool {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        restore(&e.path());
    }
    fs::remove_dir(dir).is_ok() || !dir.exists()
}

pub(super) fn color(h: HWND) {
    if let Some(c) = fence_of(h).and_then(|f| choose_color(h, f.look.color)) {
        update(h, |f| f.look.color = c);
    }
}

pub(super) fn tint(h: HWND) {
    if let Some(c) = fence_of(h).and_then(|f| choose_color(h, f.look.tint.unwrap_or(0xFF_FFFF))) {
        update(h, |f| f.look.tint = Some(c));
    }
}

pub(super) fn delete(h: HWND) {
    let Some(f) = fence_of(h) else { return };
    if !confirm(h, T::ConfirmDelete) {
        return;
    }
    with(|a| a.view(h).map(|v| v.watch = None));
    let kept = f.tabs.iter().filter(|t| t.portal.is_none()).map(store::tab_dir).filter(|d| !evacuate(d)).count();
    if kept > 0 {
        return super::bind(h);
    }
    with(|a| {
        a.views.retain(|v| v.hwnd != h);
        a.cfg.fences.retain(|x| x.id != f.id);
        a.save();
    });
    unsafe { DestroyWindow(h).ok() };
}

pub(super) fn restore_item(h: HWND, p: &Path) {
    restore(p);
    reload(h);
}
