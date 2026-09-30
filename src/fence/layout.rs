use super::{BORDER, TITLE, fence_of, render, update, xy};
use crate::{
    app::with,
    domain::{Fence, Zone, anim, zone},
    win::*,
};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

fn progress(h: HWND, f: &Fence) -> f32 {
    with(|a| a.view(h).map(|v| v.unroll)).flatten().unwrap_or(if f.rolled { 0. } else { 1. })
}

pub(super) fn resize(h: HWND, f: &Fence) {
    let height = anim::height(scale(h, TITLE), f.h, progress(h, f));
    let _ = unsafe { SetWindowPos(h, None, 0, 0, f.w, height, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) };
}

pub(super) fn apply(h: HWND) {
    let Some(f) = fence_of(h) else { return };
    resize(h, &f);
    render(h);
}

pub(super) fn persist(h: HWND) {
    let r = window_rect(h);
    update(h, |f| {
        (f.x, f.y, f.w) = (r.left, r.top, r.right - r.left);
        if !f.rolled {
            f.h = r.bottom - r.top;
        }
    });
}

pub(super) fn hit(h: HWND, lp: LPARAM) -> u32 {
    let (r, (x, y)) = (window_rect(h), xy(lp));
    let folded = fence_of(h).is_some_and(|f| progress(h, &f) < 1.);
    match zone((r.right - r.left, r.bottom - r.top), (x - r.left, y - r.top), scale(h, BORDER), scale(h, TITLE), folded) {
        Zone::Client => HTCLIENT,
        Zone::Caption => HTCAPTION,
        Zone::Left => HTLEFT,
        Zone::Right => HTRIGHT,
        Zone::Top => HTTOP,
        Zone::Bottom => HTBOTTOM,
        Zone::TopLeft => HTTOPLEFT,
        Zone::TopRight => HTTOPRIGHT,
        Zone::BottomLeft => HTBOTTOMLEFT,
        Zone::BottomRight => HTBOTTOMRIGHT,
    }
}
