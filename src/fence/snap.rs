use crate::{domain::snap, win::*};
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

pub(super) fn moving(h: HWND, lp: LPARAM) {
    adjust(h, lp, snap::moving);
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
