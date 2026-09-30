use super::{BORDER, TITLE, fence_of, metrics, render, update, xy};
use crate::{
    app::with,
    domain::{Fence, Zone, anim, grid, zone},
    win::*,
};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

const MARGIN: i32 = 16;

fn progress(h: HWND, f: &Fence) -> f32 {
    with(|a| a.view(h).map(|v| v.unroll)).flatten().unwrap_or(if f.rolled { 0. } else { 1. })
}

pub(super) fn full(h: HWND, f: &Fence) -> i32 {
    if !f.look.auto_height {
        return f.h;
    }
    let n = with(|a| a.view(h).map(|v| v.items.len())).flatten().unwrap_or(0);
    let floor = work_area(&window_rect(h)).bottom - window_rect(h).top - scale(h, MARGIN);
    grid::fit(n, f.w, metrics(h).0, scale(h, TITLE + 4 + 8), (scale(h, Fence::MIN.1), floor))
}

pub(super) fn resize(h: HWND, f: &Fence) {
    let height = anim::height(scale(h, TITLE), full(h, f), progress(h, f));
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
        if !f.rolled && !f.look.auto_height {
            f.h = r.bottom - r.top;
        }
    });
}

pub(super) fn hit(h: HWND, lp: LPARAM) -> u32 {
    let (r, (x, y)) = (window_rect(h), xy(lp));
    let f = fence_of(h);
    let folded = f.as_ref().is_some_and(|f| progress(h, f) < 1.);
    let z = zone((r.right - r.left, r.bottom - r.top), (x - r.left, y - r.top), scale(h, BORDER), scale(h, TITLE), folded);
    let z = match f {
        Some(f) if f.locked => z.fixed(),
        Some(f) if f.look.auto_height => z.width_only(),
        _ => z,
    };
    match z {
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
