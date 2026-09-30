pub mod updates;

use crate::{
    app::{new_fence, rebuild, with},
    i18n::T,
    layered::glyph as g,
    prefs, rules, settings,
    win::*,
};
use windows::{
    Win32::{
        Foundation::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

const WM_TRAY: u32 = WM_APP + 1;
const WM_NEW: u32 = WM_APP + 3;
const WM_SETTINGS: u32 = WM_APP + 4;
const CLASS: PCWSTR = if cfg!(debug_assertions) { w!("RustDesktopIcons.Tray.dev") } else { w!("RustDesktopIcons.Tray") };
const URL: &str = "https://github.com/vincentxjoubert-lang/rust-desktop-icons";

pub fn hwnd() -> Option<HWND> {
    unsafe { FindWindowW(CLASS, None) }.ok()
}

pub fn request(new: bool) {
    if let Some(h) = hwnd() {
        let _ = unsafe { PostMessageW(Some(h), if new { WM_NEW } else { WM_SETTINGS }, WPARAM(0), LPARAM(0)) };
    }
}

pub fn create() -> Option<HWND> {
    register(CLASS, Some(proc));
    unsafe { CreateWindowExW(WINDOW_EX_STYLE(0), CLASS, PCWSTR::null(), WS_POPUP, 0, 0, 0, 0, None, None, Some(inst()), None).ok() }
}

pub fn icon(h: HWND, op: NOTIFY_ICON_MESSAGE) {
    unsafe {
        let size = GetSystemMetrics(SM_CXSMICON);
        let icon = LoadImageW(Some(inst()), PCWSTR(1 as _), IMAGE_ICON, size, size, LR_DEFAULTCOLOR | LR_SHARED)
            .map(|i| HICON(i.0))
            .unwrap_or_default();
        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: h,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAY,
            hIcon: icon,
            ..Default::default()
        };
        for (d, s) in nid.szTip.iter_mut().zip("Rust Desktop Icons".encode_utf16()) {
            *d = s;
        }
        let _ = Shell_NotifyIconW(op, &nid);
    }
}

unsafe extern "system" fn proc(h: HWND, m: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match m {
        WM_TRAY if lp.0 as u32 == WM_RBUTTONUP => menu_at(h),
        WM_TRAY if lp.0 as u32 == WM_LBUTTONDBLCLK => settings::open(),
        WM_NEW => new_fence(),
        WM_SETTINGS => settings::open(),
        WM_CLOSE => unsafe { PostQuitMessage(0) },
        rules::WM_DESK => rules::changed(h),
        WM_TIMER if wp.0 == rules::TIMER => rules::run(Some(h), false),
        updates::WM_UPDATE => updates::done(wp.0, lp.0 != 0),
        _ if with(|a| a.taskbar) == Some(m) => {
            icon(h, NIM_ADD);
            rebuild();
        }
        _ => return unsafe { DefWindowProcW(h, m, wp, lp) },
    }
    LRESULT(0)
}

fn menu_at(h: HWND) {
    let Some((m, rtl)) = with(|a| {
        let m = menu();
        a.entry(m, 1, T::NewFence, false, g::ADD);
        a.entry(m, 7, T::Settings, false, g::SETTINGS);
        separator(m);
        let l = prefs::lang_menu(a);
        a.sub(m, T::Language, l, g::GLOBE);
        a.entry(m, 2, T::Autostart, a.cfg.autostart, g::POWER);
        a.entry(m, 3, T::AutoUpdate, a.cfg.auto_update, g::SYNC);
        a.entry(m, 4, T::CheckUpdates, false, g::REFRESH);
        a.entry(m, 5, T::About, false, g::INFO);
        separator(m);
        a.entry(m, 6, T::Quit, false, g::CLOSE);
        (m, a.rtl())
    }) else {
        return;
    };
    match popup(h, m, rtl) {
        1 => new_fence(),
        2 => prefs::toggle_autostart(),
        3 => prefs::toggle_auto_update(),
        4 => updates::check_now(h),
        5 => {
            let text = format!("Rust Desktop Icons {}\nMIT License\n{URL}", env!("CARGO_PKG_VERSION"));
            msgbox(Some(h), &text, MB_OK | MB_ICONINFORMATION, rtl);
        }
        6 => unsafe { PostQuitMessage(0) },
        7 => settings::open(),
        n => {
            prefs::set_lang(n);
        }
    }
}
