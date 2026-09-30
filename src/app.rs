use crate::{
    domain::{Config, Fence},
    fence,
    i18n::{self, T},
    rules, settings, shell, store, tray,
    win::*,
};
use std::{cell::RefCell, collections::HashSet, path::PathBuf};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, Threading::CreateMutexW},
        UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

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
    pub inside: bool,
    pub hold: bool,
    pub unroll: f32,
    pub glow: f32,
    pub tick: Option<std::time::Instant>,
    pub watch: Option<shell::Watch>,
}

impl View {
    fn new(hwnd: HWND, f: &Fence) -> Self {
        let unroll = if f.rolled { 0. } else { 1. };
        let glow = if f.look.chameleon { 0. } else { 1. };
        Self {
            hwnd,
            id: f.id,
            items: vec![],
            scroll: 0,
            edit: None,
            hover: None,
            inside: false,
            hold: false,
            unroll,
            glow,
            tick: None,
            watch: None,
        }
    }
}

pub struct App {
    pub cfg: Config,
    pub views: Vec<View>,
    fonts: Vec<(i32, HFONT)>,
    pub taskbar: u32,
    pub seen: HashSet<PathBuf>,
    pub desk: Option<shell::Watch>,
    pub panel: Option<settings::Panel>,
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

    pub fn fence_of(&mut self, h: HWND) -> Option<&mut Fence> {
        let id = self.view(h)?.id;
        self.fence(id)
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
                CLEARTYPE_QUALITY,
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

pub fn change(f: impl FnOnce(&mut Config)) {
    with(|a| {
        f(&mut a.cfg);
        a.save();
    });
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
        let new = std::env::args().any(|a| a == shell::NEW_ARG);
        if GetLastError() == ERROR_ALREADY_EXISTS {
            tray::request(new);
            return;
        }
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        register(fence::CLASS, Some(fence::proc));
        settings::register_class();
        let Some(tray) = tray::create() else { return };
        let cfg = store::load();
        if !cfg.autostart {
            shell::set_autostart(false);
        }
        tray::updates::enable(cfg.auto_update);
        let taskbar = RegisterWindowMessageW(w!("TaskbarCreated"));
        APP.with(|a| {
            *a.borrow_mut() = Some(App { cfg, views: vec![], fonts: vec![], taskbar, seen: HashSet::new(), desk: None, panel: None })
        });
        tray::icon(tray, NIM_ADD);
        rebuild();
        rules::watch(tray);
        register_verb();
        if new {
            new_fence();
        }
        tray::updates::spawn(tray);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !fence::key(&msg) {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        tray::icon(tray, NIM_DELETE);
        APP.with(|a| a.borrow_mut().take());
    }
}

pub fn register_verb() {
    if let Some(label) = with(|a| a.t(T::DesktopVerb)).filter(|_| !cfg!(debug_assertions)) {
        shell::set_desktop_verb(label);
    }
}

pub fn new_fence() {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p).ok() };
    let s = |v: i32| v * unsafe { GetDpiForSystem() }.max(96) as i32 / 96;
    let Some(f) = with(|a| {
        let f = Fence::new(a.cfg.next_id(), a.t(T::Fence), (p.x - s(180), p.y - s(120), s(360), s(240)), a.cfg.look.clone());
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
        for t in f.tabs.iter().filter(|t| t.portal.is_none()) {
            let _ = std::fs::create_dir_all(store::tab_dir(t));
        }
        with(|a| a.views.push(View::new(h, f)));
        fence::bind(h);
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
