use super::{View, with};
use crate::{domain::Fence, fence, i18n::T, shell, store, win::*};
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn register_verb() {
    if let Some(label) = with(|a| a.t(T::DesktopVerb)).filter(|_| !cfg!(debug_assertions)) {
        shell::set_desktop_verb(label);
    }
}

pub fn hide_recycle_if_placed() {
    if with(|a| a.cfg.has_recycle()) == Some(true) {
        shell::recycle::show_on_desktop(false);
    }
}

pub fn new_fence() {
    let (px, py) = cursor_pos();
    let s = sys_scale;
    let Some(f) = with(|a| {
        let f = Fence::new(a.cfg.next_id(), a.t(T::Fence), (px - s(180), py - s(120), s(360), s(240)), a.cfg.look.clone());
        a.cfg.fences.push(f.clone());
        a.save();
        f
    }) else {
        return;
    };
    open(&f);
}

fn open(f: &Fence) {
    if let Some(h) = fence::create(f) {
        for t in f.tabs.iter().filter(|t| t.portal.is_none()) {
            let _ = std::fs::create_dir_all(store::tab_dir(t));
        }
        with(|a| a.views.push(View::new(h, f)));
        fence::bind(h);
    }
}

pub fn rebuild() {
    let Some((old, fences)) = with(|a| (std::mem::take(&mut a.views), a.cfg.fences.clone())) else {
        return;
    };
    for v in old {
        unsafe { DestroyWindow(v.hwnd).ok() };
    }
    fences.iter().for_each(open);
}
