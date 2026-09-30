use crate::{
    domain::{Config, Fence},
    fence,
    i18n::{self, T},
    shell, store,
    update::{self, Outcome},
    win::*,
};
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, Threading::CreateMutexW},
        UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

const WM_TRAY: u32 = WM_APP + 1;
const WM_UPDATE: u32 = WM_APP + 2;
const TRAY: PCWSTR = w!("RustDesktopIcons.Tray");
const URL: &str = "https://github.com/vincentxjoubert-lang/rust-desktop-icons";
static AUTO_UPDATE: AtomicBool = AtomicBool::new(false);

pub struct Item {
    pub path: PathBuf,
    pub name: Vec<u16>,
    pub icon: HICON,
}

impl Drop for Item {
    fn drop(&mut self) {
        if !self.icon.is_invalid() {
            unsafe { DestroyIcon(self.icon).ok() };
        }
    }
}

pub struct View {
    pub hwnd: HWND,
    pub id: u64,
    pub items: Vec<Item>,
    pub scroll: i32,
    pub edit: Option<HWND>,
    pub hover: Option<usize>,
    pub watch: Option<shell::Watch>,
}

pub struct App {
    pub cfg: Config,
    pub views: Vec<View>,
    fonts: Vec<(i32, HFONT)>,
    taskbar: u32,
}

impl App {
    pub fn lang(&self) -> usize {
        i18n::resolve(self.cfg.lang.as_deref(), &shell::locale())
    }

    pub fn t(&self, k: T) -> &'static str {
        i18n::get(self.lang(), k)
    }

    pub fn rtl(&self) -> bool {
        i18n::rtl(self.lang())
    }

    pub fn view(&mut self, h: HWND) -> Option<&mut View> {
        self.views.iter_mut().find(|v| v.hwnd == h)
    }

    pub fn fence(&mut self, id: u64) -> Option<&mut Fence> {
        self.cfg.fences.iter_mut().find(|f| f.id == id)
    }

    pub fn font(&mut self, px: i32, weight: u32) -> HFONT {
        let key = px * 1000 + weight as i32;
        if let Some(&(_, f)) = self.fonts.iter().find(|(k, _)| *k == key) {
            return f;
        }
        let f = unsafe {
            CreateFontW(
                -px,
                0,
                0,
                0,
                weight as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                ANTIALIASED_QUALITY,
                FF_DONTCARE.0 as u32,
                w!("Segoe UI"),
            )
        };
        self.fonts.push((key, f));
        f
    }

    pub fn save(&self) {
        let _ = store::save(&self.cfg);
    }
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

pub fn with<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok()?.as_mut().map(f))
}

pub fn run() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let name = if cfg!(debug_assertions) { w!("Local\\RustDesktopIcons.dev") } else { w!("Local\\RustDesktopIcons") };
        let _mutex = CreateMutexW(None, true, name);
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        register(TRAY, Some(tray_proc));
        register(fence::CLASS, Some(fence::proc));
        let Ok(tray) = CreateWindowExW(WINDOW_EX_STYLE(0), TRAY, PCWSTR::null(), WS_POPUP, 0, 0, 0, 0, None, None, Some(inst()), None)
        else {
            return;
        };
        let cfg = store::load();
        if !cfg.autostart {
            shell::set_autostart(false);
        }
        AUTO_UPDATE.store(cfg.auto_update, Ordering::Relaxed);
        let taskbar = RegisterWindowMessageW(w!("TaskbarCreated"));
        APP.with(|a| *a.borrow_mut() = Some(App { cfg, views: vec![], fonts: vec![], taskbar }));
        tray_icon(tray, NIM_ADD);
        rebuild();
        background_updates(tray);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !fence::key(&msg) {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        tray_icon(tray, NIM_DELETE);
        APP.with(|a| a.borrow_mut().take());
    }
}

fn tray_icon(h: HWND, op: NOTIFY_ICON_MESSAGE) {
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

unsafe extern "system" fn tray_proc(h: HWND, m: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match m {
        WM_TRAY if matches!(lp.0 as u32, WM_LBUTTONUP | WM_RBUTTONUP) => tray_menu(h),
        WM_UPDATE => updated(wp.0, lp.0 != 0),
        _ if with(|a| a.taskbar) == Some(m) => {
            tray_icon(h, NIM_ADD);
            rebuild();
        }
        _ => return unsafe { DefWindowProcW(h, m, wp, lp) },
    }
    LRESULT(0)
}

fn tray_menu(h: HWND) {
    let Some((m, rtl)) = with(|a| {
        let (m, l) = (menu(), menu());
        item(l, 100, a.t(T::Auto), a.cfg.lang.is_none());
        for (i, (code, name)) in i18n::LANGS.iter().enumerate() {
            item(l, 101 + i, name, a.cfg.lang.as_deref() == Some(*code));
        }
        item(m, 1, a.t(T::NewFence), false);
        separator(m);
        submenu(m, a.t(T::Language), l);
        item(m, 2, a.t(T::Autostart), a.cfg.autostart);
        item(m, 3, a.t(T::AutoUpdate), a.cfg.auto_update);
        item(m, 4, a.t(T::CheckUpdates), false);
        item(m, 5, a.t(T::About), false);
        separator(m);
        item(m, 6, a.t(T::Quit), false);
        (m, a.rtl())
    }) else {
        return;
    };
    match popup(h, m, rtl) {
        1 => new_fence(),
        2 => {
            with(|a| {
                a.cfg.autostart ^= true;
                shell::set_autostart(a.cfg.autostart);
                a.save();
            });
        }
        3 => {
            with(|a| {
                a.cfg.auto_update ^= true;
                AUTO_UPDATE.store(a.cfg.auto_update, Ordering::Relaxed);
                a.save();
            });
        }
        4 => check_now(h),
        5 => {
            let rtl = with(|a| a.rtl()).unwrap_or(false);
            msgbox(
                Some(h),
                &format!("Rust Desktop Icons {}\nMIT License\n{URL}", env!("CARGO_PKG_VERSION")),
                MB_OK | MB_ICONINFORMATION,
                rtl,
            );
        }
        6 => unsafe { PostQuitMessage(0) },
        n @ 100..=125 => {
            with(|a| {
                a.cfg.lang = n.checked_sub(101).map(|i| i18n::LANGS[i].0.to_string());
                a.save();
            });
        }
        _ => {}
    }
}

fn post(h: isize, r: Outcome, manual: bool) {
    unsafe { PostMessageW(Some(HWND(h as _)), WM_UPDATE, WPARAM(r as usize), LPARAM(manual as isize)).ok() };
}

fn background_updates(h: HWND) {
    let h = h.0 as isize;
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));
        loop {
            if AUTO_UPDATE.load(Ordering::Relaxed) && update::run() == Outcome::Installing {
                return post(h, Outcome::Installing, false);
            }
            thread::sleep(Duration::from_secs(6 * 3600));
        }
    });
}

fn check_now(h: HWND) {
    let h = h.0 as isize;
    thread::spawn(move || post(h, update::run(), true));
}

fn updated(code: usize, manual: bool) {
    let Some((ok, fail, rtl)) = with(|a| (a.t(T::UpToDate), a.t(T::UpdateFailed), a.rtl())) else {
        return;
    };
    if code == Outcome::Installing as usize {
        return unsafe { PostQuitMessage(0) };
    }
    if manual {
        let (text, icon) = if code == Outcome::Current as usize { (ok, MB_ICONINFORMATION) } else { (fail, MB_ICONWARNING) };
        msgbox(None, text, MB_OK | icon, rtl);
    }
}

pub fn new_fence() {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p).ok() };
    let s = |v: i32| v * unsafe { GetDpiForSystem() }.max(96) as i32 / 96;
    let Some(f) = with(|a| {
        let f = Fence {
            id: a.cfg.next_id(),
            title: a.t(T::Fence).into(),
            x: p.x - s(180),
            y: p.y - s(120),
            w: s(360),
            h: s(240),
            ..Fence::default()
        };
        a.cfg.fences.push(f.clone());
        a.save();
        f
    }) else {
        return;
    };
    open(&f);
}

fn open(f: &Fence) {
    if let Some(h) = fence::create(f) {
        let dir = store::fence_dir(f.id);
        let _ = std::fs::create_dir_all(&dir);
        let watch = shell::Watch::new(h, &dir, fence::WM_CHANGED);
        with(|a| a.views.push(View { hwnd: h, id: f.id, items: vec![], scroll: 0, edit: None, hover: None, watch }));
        fence::reload(h);
    }
}

pub fn rebuild() {
    let Some((old, fences)) = with(|a| (std::mem::take(&mut a.views), a.cfg.fences.clone())) else {
        return;
    };
    for v in old {
        unsafe { DestroyWindow(v.hwnd).ok() };
    }
    fences.iter().for_each(open);
}
