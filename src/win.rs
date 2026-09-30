use std::{os::windows::ffi::OsStrExt, path::Path};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::{GetMonitorInfoW, HBITMAP, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::Dialogs::*,
            HiDpi::{GetDpiForSystem, GetDpiForWindow},
            Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent},
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
    let m = unsafe { CreatePopupMenu().unwrap_or_default() };
    let info = MENUINFO { cbSize: size_of::<MENUINFO>() as u32, fMask: MIM_STYLE, dwStyle: MNS_CHECKORBMP, ..Default::default() };
    let _ = unsafe { SetMenuInfo(m, &info) };
    m
}

pub fn item(m: HMENU, id: usize, text: &str, checked: bool) {
    let flags = if checked { MF_STRING | MF_CHECKED } else { MF_STRING };
    unsafe { AppendMenuW(m, flags, id, PCWSTR(wide(text).as_ptr())).ok() };
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
        let mut p = POINT::default();
        let _ = GetCursorPos(&mut p);
        let _ = SetForegroundWindow(h);
        let align = if rtl { TPM_LAYOUTRTL | TPM_RIGHTALIGN } else { TPM_LEFTALIGN };
        let r = TrackPopupMenu(m, TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON | align, p.x, p.y, None, h, None);
        let _ = DestroyMenu(m);
        let _ = PostMessageW(Some(h), WM_NULL, WPARAM(0), LPARAM(0));
        r.0 as usize
    }
}
