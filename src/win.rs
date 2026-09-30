use std::{os::windows::ffi::OsStrExt, path::Path};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Dwm::{DWM_TIMING_INFO, DwmGetCompositionTimingInfo},
        Graphics::Gdi::{GetMonitorInfoW, HBITMAP, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect},
        System::{
            LibraryLoader::GetModuleHandleW,
            Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
        },
        UI::{
            Controls::Dialogs::*,
            HiDpi::{GetDpiForSystem, GetDpiForWindow},
            Input::KeyboardAndMouse::{GetKeyState, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VIRTUAL_KEY},
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

pub fn wide_path(p: &Path) -> Vec<u16> {
    p.as_os_str().encode_wide().chain([0]).collect()
}

pub fn inst() -> HINSTANCE {
    unsafe { GetModuleHandleW(None).map(|m| HINSTANCE(m.0)).unwrap_or_default() }
}

pub fn window_rect(h: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe { GetWindowRect(h, &mut r).ok() };
    r
}

pub fn client_rect(h: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe { GetClientRect(h, &mut r).ok() };
    r
}

pub fn work_area(r: &RECT) -> RECT {
    let mut mi = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    let _ = unsafe { GetMonitorInfoW(MonitorFromRect(r, MONITOR_DEFAULTTONEAREST), &mut mi) };
    mi.rcWork
}

pub fn scale(h: HWND, v: i32) -> i32 {
    v * unsafe { GetDpiForWindow(h) }.max(96) as i32 / 96
}

pub const WM_MOUSELEAVE: u32 = 0x02A3;

pub fn xy(lp: LPARAM) -> (i32, i32) {
    (lp.0 as i16 as i32, (lp.0 >> 16) as i16 as i32)
}

pub fn track_leave(h: HWND) {
    let mut tme = TRACKMOUSEEVENT { cbSize: size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
    let _ = unsafe { TrackMouseEvent(&mut tme) };
}

pub fn cursor_pos() -> (i32, i32) {
    let mut p = POINT::default();
    let _ = unsafe { GetCursorPos(&mut p) };
    (p.x, p.y)
}

pub fn key_down(k: VIRTUAL_KEY) -> bool {
    (unsafe { GetKeyState(k.0 as i32) }) < 0
}

fn qpc_ms(ticks: i64) -> f64 {
    let mut freq = 0i64;
    let _ = unsafe { QueryPerformanceFrequency(&mut freq) };
    ticks as f64 * 1000. / freq.max(1) as f64
}

pub fn now_ms() -> f64 {
    let mut t = 0i64;
    let _ = unsafe { QueryPerformanceCounter(&mut t) };
    qpc_ms(t)
}

pub fn next_vblank_ms() -> Option<f64> {
    let mut info = DWM_TIMING_INFO { cbSize: size_of::<DWM_TIMING_INFO>() as u32, ..Default::default() };
    unsafe { DwmGetCompositionTimingInfo(HWND::default(), &mut info) }.ok()?;
    (info.qpcRefreshPeriod > 0).then(|| qpc_ms((info.qpcVBlank + info.qpcRefreshPeriod) as i64))
}

pub fn sys_scale(v: i32) -> i32 {
    v * unsafe { GetDpiForSystem() }.max(96) as i32 / 96
}

pub fn register(class: PCWSTR, proc: WNDPROC) {
    let wc = WNDCLASSW {
        style: CS_DBLCLKS,
        lpfnWndProc: proc,
        hInstance: inst(),
        lpszClassName: class,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        ..Default::default()
    };
    unsafe { RegisterClassW(&wc) };
}

pub fn choose_color(h: HWND, init: u32) -> Option<u32> {
    let mut custom = [COLORREF(0); 16];
    let mut cc = CHOOSECOLORW {
        lStructSize: size_of::<CHOOSECOLORW>() as u32,
        hwndOwner: h,
        rgbResult: COLORREF(init),
        lpCustColors: custom.as_mut_ptr(),
        Flags: CC_RGBINIT | CC_FULLOPEN,
        ..Default::default()
    };
    unsafe { ChooseColorW(&mut cc) }.as_bool().then_some(cc.rgbResult.0 & 0xFF_FFFF)
}

pub fn msgbox(h: Option<HWND>, text: &str, style: MESSAGEBOX_STYLE, rtl: bool) -> MESSAGEBOX_RESULT {
    let flags = if rtl { style | MB_RTLREADING | MB_RIGHT } else { style };
    unsafe { MessageBoxW(h, PCWSTR(wide(text).as_ptr()), w!("Rust Desktop Icons"), flags | MB_SETFOREGROUND) }
}

pub fn menu() -> HMENU {
    unsafe { CreatePopupMenu().unwrap_or_default() }
}

pub fn item(m: HMENU, id: usize, text: &str, checked: bool) {
    let label = if checked { format!("{text}\t\u{2713}") } else { text.to_string() };
    unsafe { AppendMenuW(m, MF_STRING, id, PCWSTR(wide(&label).as_ptr())).ok() };
}

pub fn icon(m: HMENU, item: u32, by_position: bool, bmp: HBITMAP) {
    let info = MENUITEMINFOW { cbSize: size_of::<MENUITEMINFOW>() as u32, fMask: MIIM_BITMAP, hbmpItem: bmp, ..Default::default() };
    unsafe { SetMenuItemInfoW(m, item, by_position, &info).ok() };
}

pub fn submenu(m: HMENU, text: &str, child: HMENU) {
    unsafe { AppendMenuW(m, MF_POPUP, child.0 as usize, PCWSTR(wide(text).as_ptr())).ok() };
}

pub fn separator(m: HMENU) {
    unsafe { AppendMenuW(m, MF_SEPARATOR, 0, PCWSTR::null()).ok() };
}

pub fn popup(h: HWND, m: HMENU, rtl: bool) -> usize {
    unsafe {
        let (x, y) = cursor_pos();
        let _ = SetForegroundWindow(h);
        let align = if rtl { TPM_LAYOUTRTL | TPM_RIGHTALIGN } else { TPM_LEFTALIGN };
        let r = TrackPopupMenu(m, TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON | align, x, y, None, h, None);
        let _ = DestroyMenu(m);
        let _ = PostMessageW(Some(h), WM_NULL, WPARAM(0), LPARAM(0));
        r.0 as usize
    }
}
