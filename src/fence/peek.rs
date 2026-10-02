use super::{PEEK, anim, flagged, update, view};
use crate::win::{cursor_pos, window_rect};
use windows::Win32::{
    Foundation::*,
    Graphics::Gdi::PtInRect,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

pub(super) fn enter(h: HWND) {
    if view(h, |v| !std::mem::replace(&mut v.inside, true)) == Some(true) {
        anim::start(h);
        unsafe { SetTimer(Some(h), PEEK, 200, None) };
    }
}

pub(super) fn keep_open(h: HWND, f: impl FnOnce()) {
    flagged(h, |v| &mut v.busy, f);
}

pub(super) fn check(h: HWND) {
    let (x, y) = cursor_pos();
    let inside = unsafe { PtInRect(&window_rect(h), POINT { x, y }) }.as_bool();
    if inside || view(h, |v| v.edit.is_some() || v.busy) != Some(false) {
        return;
    }
    let _ = unsafe { KillTimer(Some(h), PEEK) };
    view(h, |v| (v.inside, v.hold) = (false, false));
    anim::start(h);
}

pub(super) fn toggle(h: HWND) {
    view(h, |v| v.hold = true);
    update(h, |f| f.rolled ^= true);
    anim::start(h);
}
