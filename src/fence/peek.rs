use super::{PEEK, anim, update};
use crate::{
    app::with,
    win::{cursor_pos, window_rect},
};
use windows::Win32::{
    Foundation::*,
    Graphics::Gdi::PtInRect,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

pub(super) fn enter(h: HWND) {
    if with(|a| a.view(h).map(|v| !std::mem::replace(&mut v.inside, true))).flatten() == Some(true) {
        anim::start(h);
        unsafe { SetTimer(Some(h), PEEK, 200, None) };
    }
}

pub(super) fn check(h: HWND) {
    let (x, y) = cursor_pos();
    let inside = unsafe { PtInRect(&window_rect(h), POINT { x, y }) }.as_bool();
    if inside || with(|a| a.view(h).map(|v| v.edit.is_some())).flatten() != Some(false) {
        return;
    }
    let _ = unsafe { KillTimer(Some(h), PEEK) };
    with(|a| a.view(h).map(|v| (v.inside, v.hold) = (false, false)));
    anim::start(h);
}

pub(super) fn toggle(h: HWND) {
    with(|a| a.view(h).map(|v| v.hold = true));
    update(h, |f| f.rolled ^= true);
    anim::start(h);
}
