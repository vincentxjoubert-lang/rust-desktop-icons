use super::view;
use crate::{app::with, domain::snap, win::*};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::{
        WMSZ_BOTTOM, WMSZ_BOTTOMLEFT, WMSZ_BOTTOMRIGHT, WMSZ_LEFT, WMSZ_RIGHT, WMSZ_TOP, WMSZ_TOPLEFT, WMSZ_TOPRIGHT,
    },
};

const DIST: i32 = 16;

fn adjust(h: HWND, lp: LPARAM, f: impl FnOnce(snap::Rect, snap::Rect, i32) -> snap::Rect) {
    let r = unsafe { &mut *(lp.0 as *mut RECT) };
    let a = work_area(r);
    let [left, top, right, bottom] = f([r.left, r.top, r.right, r.bottom], [a.left, a.top, a.right, a.bottom], scale(h, DIST));
    *r = RECT { left, top, right, bottom };
}

pub(super) fn begin(h: HWND) {
    let r = window_rect(h);
    let start = ([r.left, r.top, r.right, r.bottom], cursor_pos());
    view(h, |v| v.drag = Some(start));
}

pub(super) fn end(h: HWND) {
    view(h, |v| v.drag = None);
}

pub(super) fn moving(h: HWND, lp: LPARAM) {
    let drag = with(|a| a.view(h).and_then(|v| v.drag)).flatten();
    adjust(h, lp, |r, a, d| snap::moving(drag.map_or(r, |(start, c0)| snap::follow(start, c0, cursor_pos())), a, d));
}

pub(super) fn sizing(h: HWND, wp: WPARAM, lp: LPARAM) {
    let e = wp.0 as u32;
    let sides = [
        matches!(e, WMSZ_LEFT | WMSZ_TOPLEFT | WMSZ_BOTTOMLEFT),
        matches!(e, WMSZ_TOP | WMSZ_TOPLEFT | WMSZ_TOPRIGHT),
        matches!(e, WMSZ_RIGHT | WMSZ_TOPRIGHT | WMSZ_BOTTOMRIGHT),
        matches!(e, WMSZ_BOTTOM | WMSZ_BOTTOMLEFT | WMSZ_BOTTOMRIGHT),
    ];
    adjust(h, lp, |r, a, d| snap::sizing(r, a, d, sides));
}
