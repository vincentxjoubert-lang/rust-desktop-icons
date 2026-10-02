use super::{fence_of, update, view};
use crate::{app::with, i18n::T, report, rules, shell, store, win::*};
use std::path::Path;
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

pub(super) fn confirm(h: HWND, q: T) -> bool {
    with(|a| (a.t(q), a.rtl())).is_some_and(|(q, rtl)| msgbox(Some(h), q, MB_YESNO | MB_ICONQUESTION, rtl) == IDYES)
}

pub(super) fn evacuate(dir: &Path) -> Vec<String> {
    let Some(desk) = shell::desktop() else { return vec![String::from("Desktop")] };
    let (moved, errors) = store::evacuate(dir, &desk);
    moved.into_iter().for_each(rules::ignore);
    errors
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
    view(h, |v| v.watches.clear());
    let errors: Vec<String> = f.tabs.iter().filter(|t| t.portal.is_none()).flat_map(|t| evacuate(&store::tab_dir(t))).collect();
    if !errors.is_empty() {
        report::failures(T::ErrDelete, &errors);
        return super::bind(h);
    }
    with(|a| {
        a.views.retain(|v| v.hwnd != h);
        a.cfg.fences.retain(|x| x.id != f.id);
        a.save();
    });
    if f.tabs.iter().any(|t| t.recycle) {
        shell::recycle::show_on_desktop(true);
    }
    unsafe { DestroyWindow(h).ok() };
}
