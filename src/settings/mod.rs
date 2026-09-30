mod act;
mod paint;
mod rows;

use crate::{app::with, win::*};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::ScreenToClient,
        UI::{Input::KeyboardAndMouse::VK_ESCAPE, WindowsAndMessaging::*},
    },
    core::*,
};

const CLASS: PCWSTR = w!("RustDesktopIcons.Settings");

pub struct Panel {
    pub hwnd: HWND,
    pub hover: Option<usize>,
    pub close_hot: bool,
    pub scroll: i32,
}

pub fn register_class() {
    register(CLASS, Some(proc));
}

fn hwnd() -> Option<HWND> {
    with(|a| a.panel.as_ref().map(|p| p.hwnd)).flatten()
}

fn close_rect(h: HWND) -> RECT {
    let (w, s) = (client_rect(h).right, |v| scale(h, v));
    RECT { left: w - s(44), top: s(12), right: w - s(14), bottom: s(42) }
}

fn inside(r: &RECT, (x, y): (i32, i32)) -> bool {
    (r.left..r.right).contains(&x) && (r.top..r.bottom).contains(&y)
}

fn height(h: HWND) -> i32 {
    let area = work_area(&window_rect(h));
    content(h).min(area.bottom - area.top - scale(h, 60))
}

pub fn refresh() {
    let Some(h) = hwnd() else { return };
    let (r, ht) = (window_rect(h), height(h));
    if r.bottom - r.top != ht {
        let _ = unsafe { SetWindowPos(h, None, 0, 0, r.right - r.left, ht, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) };
    }
    paint::render(h);
}

pub fn open() {
    if let Some(h) = hwnd() {
        let _ = unsafe { SetForegroundWindow(h) };
        return refresh();
    }
    let s = sys_scale;
    let mut p = POINT::default();
    let _ = unsafe { GetCursorPos(&mut p) };
    let area = work_area(&RECT { left: p.x, top: p.y, right: p.x + 1, bottom: p.y + 1 });
    let w = s(rows::WIDTH);
    let (x, y) = (area.left + (area.right - area.left - w) / 2, area.top + s(60));
    let ex = WS_EX_LAYERED | WS_EX_APPWINDOW;
    let Ok(h) =
        (unsafe { CreateWindowExW(ex, CLASS, w!("Rust Desktop Icons"), WS_POPUP, x, y, w, s(400), None, None, Some(inst()), None) })
    else {
        return;
    };
    with(|a| a.panel = Some(Panel { hwnd: h, hover: None, close_hot: false, scroll: 0 }));
    refresh();
    unsafe {
        let _ = ShowWindow(h, SW_SHOW);
        let _ = SetForegroundWindow(h);
    }
}

fn close() {
    if let Some(h) = hwnd() {
        unsafe { DestroyWindow(h).ok() };
    }
}

fn content(h: HWND) -> i32 {
    scale(h, rows::TOP + 16) + scale(h, with(|a| rows::total(&rows::build(a))).unwrap_or(0))
}

fn row_at(h: HWND, (_, y): (i32, i32)) -> Option<usize> {
    let top = scale(h, rows::TOP);
    if y < top {
        return None;
    }
    with(|a| {
        let scroll = a.panel.as_ref()?.scroll;
        rows::at(&rows::build(a), (y - top + scroll) * 96 / scale(h, 96))
    })
    .flatten()
}

fn track(h: HWND, p: (i32, i32)) {
    track_leave(h);
    let (row, hot) = (row_at(h, p), inside(&close_rect(h), p));
    let changed = with(|a| {
        let panel = a.panel.as_mut()?;
        let changed = panel.hover != row || panel.close_hot != hot;
        (panel.hover, panel.close_hot) = (row, hot);
        Some(changed)
    });
    if changed.flatten() == Some(true) {
        paint::render(h);
    }
}

unsafe extern "system" fn proc(h: HWND, m: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match m {
        WM_NCHITTEST => {
            let mut p = POINT { x: lp.0 as i16 as i32, y: (lp.0 >> 16) as i16 as i32 };
            let _ = unsafe { ScreenToClient(h, &mut p) };
            let caption = p.y < scale(h, rows::TOP) && !inside(&close_rect(h), (p.x, p.y));
            return LRESULT(if caption { HTCAPTION } else { HTCLIENT } as isize);
        }
        WM_MOUSEMOVE => track(h, xy(lp)),
        WM_MOUSELEAVE => {
            with(|a| a.panel.as_mut().map(|p| (p.hover, p.close_hot) = (None, false)));
            paint::render(h);
        }
        WM_LBUTTONUP => {
            let p = xy(lp);
            if inside(&close_rect(h), p) {
                close();
            } else if let Some(i) = row_at(h, p) {
                act::click(h, i);
            }
        }
        WM_MOUSEWHEEL => {
            let step = (wp.0 >> 16) as i16 as i32 * scale(h, 60) / 120;
            let max = (content(h) - client_rect(h).bottom).max(0);
            with(|a| a.panel.as_mut().map(|p| p.scroll = (p.scroll - step).clamp(0, max)));
            paint::render(h);
        }
        WM_KEYDOWN if wp.0 as u16 == VK_ESCAPE.0 => close(),
        WM_DPICHANGED => refresh(),
        WM_DESTROY => {
            with(|a| a.panel = None);
        }
        _ => return unsafe { DefWindowProcW(h, m, wp, lp) },
    }
    LRESULT(0)
}
