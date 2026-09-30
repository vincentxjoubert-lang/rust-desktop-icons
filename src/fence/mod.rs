mod actions;
mod anim;
mod drop;
mod header;
mod items;
mod layout;
mod menu;
mod paint;
mod peek;
mod snap;
mod tabs;
mod title;

use crate::{
    app::{App, with},
    domain::{Fence, icons},
    shell,
    win::*,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::{FW_NORMAL, FW_SEMIBOLD, HFONT, MONITOR_DEFAULTTONULL, MonitorFromRect, ScreenToClient},
        UI::{
            Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent},
            Shell::HDROP,
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

pub use items::{reload, reload_all};
pub use paint::render;
pub use tabs::bind;
pub use title::key;

pub const CLASS: PCWSTR = w!("RustDesktopIcons.Fence");
pub const WM_CHANGED: u32 = WM_APP + 10;
const TITLE: i32 = 34;
const BORDER: i32 = 6;
const EN_KILLFOCUS: u32 = 0x0200;
const WM_MOUSELEAVE: u32 = 0x02A3;
const MK_CONTROL: usize = 0x0008;
const RELOAD: usize = 1;
const PEEK: usize = 2;
const ANIM: usize = 3;

fn title_font(a: &mut App, h: HWND) -> HFONT {
    a.font(scale(h, 14), FW_SEMIBOLD.0)
}

fn label_font(a: &mut App, h: HWND) -> HFONT {
    a.font(scale(h, 12), FW_NORMAL.0)
}

fn xy(lp: LPARAM) -> (i32, i32) {
    (lp.0 as i16 as i32, (lp.0 >> 16) as i16 as i32)
}

fn to_client(h: HWND, (x, y): (i32, i32)) -> (i32, i32) {
    let mut p = POINT { x, y };
    let _ = unsafe { ScreenToClient(h, &mut p) };
    (p.x, p.y)
}

fn cursor(h: HWND) -> (i32, i32) {
    let mut p = POINT::default();
    let _ = unsafe { GetCursorPos(&mut p) };
    to_client(h, (p.x, p.y))
}

fn fence_of(h: HWND) -> Option<Fence> {
    with(|a| a.fence_of(h).cloned()).flatten()
}

fn update(h: HWND, f: impl FnOnce(&mut Fence)) {
    with(|a| {
        f(a.fence_of(h)?);
        a.save();
        Some(())
    });
    layout::apply(h);
}

fn metrics(h: HWND) -> (i32, i32) {
    let px = fence_of(h).map_or(icons::DEFAULT, |f| f.look.icon);
    (scale(h, icons::cell(px)), scale(h, px))
}

fn top() -> bool {
    cfg!(debug_assertions) && std::env::var_os("RDI_TOP").is_some()
}

pub fn create(f: &Fence) -> Option<HWND> {
    unsafe {
        let r = RECT { left: f.x, top: f.y, right: f.x + f.w, bottom: f.y + f.h };
        let (x, y) = if MonitorFromRect(&r, MONITOR_DEFAULTTONULL).is_invalid() { (100, 100) } else { (f.x, f.y) };
        let owner = if top() { None } else { FindWindowW(w!("Progman"), None).ok() };
        let ex = WS_EX_TOOLWINDOW | WS_EX_ACCEPTFILES | WS_EX_LAYERED;
        let h = CreateWindowExW(ex, CLASS, PCWSTR::null(), WS_POPUP, x, y, f.w, f.h, owner, None, Some(inst()), None).ok()?;
        if f.rolled {
            layout::resize(h, f);
        }
        if top() {
            let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
        Some(h)
    }
}

pub unsafe extern "system" fn proc(h: HWND, m: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match m {
        WM_NCHITTEST => return LRESULT(layout::hit(h, lp) as isize),
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_SIZE => render(h),
        WM_WINDOWPOSCHANGING => {
            let p = unsafe { &mut *(lp.0 as *mut WINDOWPOS) };
            if (p.flags & SWP_NOZORDER).0 == 0 && !top() {
                p.hwndInsertAfter = HWND_BOTTOM;
            }
        }
        WM_GETMINMAXINFO => {
            let i = unsafe { &mut *(lp.0 as *mut MINMAXINFO) };
            i.ptMinTrackSize = POINT { x: scale(h, Fence::MIN.0), y: scale(h, TITLE) };
        }
        WM_EXITSIZEMOVE => layout::persist(h),
        WM_DPICHANGED => {
            let r = unsafe { &*(lp.0 as *const RECT) };
            unsafe { SetWindowPos(h, None, r.left, r.top, r.right - r.left, r.bottom - r.top, SWP_NOZORDER | SWP_NOACTIVATE).ok() };
        }
        WM_MOVING => {
            snap::moving(h, lp);
            return LRESULT(1);
        }
        WM_SIZING => {
            snap::sizing(h, wp, lp);
            return LRESULT(1);
        }
        WM_NCLBUTTONDOWN if wp.0 as u32 == HTCAPTION && tabs::click(h, xy(lp)) => {}
        WM_NCLBUTTONDBLCLK if wp.0 as u32 == HTCAPTION => peek::toggle(h),
        WM_NCMOUSEMOVE => peek::enter(h),
        WM_NCRBUTTONUP if wp.0 as u32 == HTCAPTION => menu::context(h),
        WM_LBUTTONDBLCLK => items::item_at(h, xy(lp)).iter().for_each(|p| shell::open(p)),
        WM_CONTEXTMENU => menu::context(h),
        WM_MOUSEMOVE => {
            let mut tme = TRACKMOUSEEVENT { cbSize: size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
            let _ = unsafe { TrackMouseEvent(&mut tme) };
            peek::enter(h);
            items::hover(h, items::index_at(h, xy(lp)));
        }
        WM_MOUSELEAVE => items::hover(h, None),
        WM_MOUSEWHEEL if wp.0 & MK_CONTROL != 0 => {
            if let Some(f) = fence_of(h) {
                items::set_icon(h, icons::step(f.look.icon, (wp.0 >> 16) as i16 > 0));
            }
        }
        WM_MOUSEWHEEL => {
            let step = (wp.0 >> 16) as i16 as i32 * metrics(h).0 / 120;
            with(|a| a.view(h).map(|v| v.scroll -= step));
            render(h);
        }
        WM_DROPFILES => drop::files(h, HDROP(wp.0 as _)),
        WM_CHANGED => {
            unsafe { SetTimer(Some(h), RELOAD, 150, None) };
        }
        WM_TIMER if wp.0 == RELOAD => {
            let _ = unsafe { KillTimer(Some(h), RELOAD) };
            reload(h);
        }
        WM_TIMER if wp.0 == PEEK => peek::check(h),
        WM_TIMER if wp.0 == ANIM => anim::tick(h),
        WM_COMMAND if (wp.0 >> 16) as u32 == EN_KILLFOCUS => title::finish(h, true),
        _ => return unsafe { DefWindowProcW(h, m, wp, lp) },
    }
    LRESULT(0)
}
