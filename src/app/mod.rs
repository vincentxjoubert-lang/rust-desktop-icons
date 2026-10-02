mod fences;
mod res;
mod view;

use crate::{
    domain::{Config, Fence},
    fence,
    i18n::{self, T},
    report, rules, settings, shell, store, tray,
    win::*,
};
pub use fences::{hide_recycle_if_placed, new_fence, rebuild, register_verb};
use std::{cell::RefCell, collections::HashSet, path::PathBuf};
pub use view::{Item, View};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::{HBITMAP, HFONT},
        System::{Ole::OleInitialize, Threading::CreateMutexW},
        UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

pub struct App {
    pub cfg: Config,
    pub views: Vec<View>,
    fonts: Vec<(i32, HFONT)>,
    glyphs: Vec<(char, HBITMAP)>,
    pub taskbar: u32,
    pub seen: HashSet<PathBuf>,
    pub desk: Option<shell::Watch>,
    pub panel: Option<settings::Panel>,
    pub recycle_drag: bool,
    save_error: Option<String>,
    save_reported: bool,
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

    pub fn save(&mut self) {
        if let Err(e) = store::save(&self.cfg) {
            report::log(&format!("save failed: {e}"));
            self.save_error.get_or_insert(e.to_string());
        }
    }
}

pub fn save_now() {
    with(|a| a.save());
    check_save();
}

pub fn check_save() {
    if let Some(e) = with(|a| a.save_error.take().filter(|_| !std::mem::replace(&mut a.save_reported, true))).flatten() {
        report::alert(T::ErrSave, &e);
    }
}

pub fn change(f: impl FnOnce(&mut Config)) {
    with(|a| {
        f(&mut a.cfg);
        a.save();
    });
    check_save();
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

pub fn with<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok()?.as_mut().map(f))
}

pub fn run() {
    if std::env::args().any(|a| a == shell::UNINSTALL_ARG) {
        return crate::uninstall::run();
    }
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let name = if cfg!(debug_assertions) { w!("Local\\RustDesktopIcons.dev") } else { w!("Local\\RustDesktopIcons") };
        let _mutex = CreateMutexW(None, true, name);
        let running = GetLastError() == ERROR_ALREADY_EXISTS;
        let new = std::env::args().any(|a| a == shell::NEW_ARG);
        if running {
            tray::request(new);
            return;
        }
        report::install_panic_hook();
        let _ = OleInitialize(None);
        register(fence::CLASS, Some(fence::proc));
        settings::register_class();
        let Some(tray) = tray::create() else { return };
        let existed = store::root().join("config.json").exists();
        let (cfg, damaged) = store::load();
        if !cfg.autostart {
            shell::set_autostart(false);
        }
        tray::updates::enable(cfg.auto_update);
        let taskbar = RegisterWindowMessageW(w!("TaskbarCreated"));
        APP.with(|a| {
            *a.borrow_mut() = Some(App {
                cfg,
                views: vec![],
                fonts: vec![],
                glyphs: vec![],
                taskbar,
                seen: HashSet::new(),
                desk: None,
                panel: None,
                recycle_drag: false,
                save_error: None,
                save_reported: false,
            })
        });
        tray::icon(tray, NIM_ADD);
        if let Some(bad) = damaged {
            report::alert(T::ErrConfig, &bad.display().to_string());
        }
        rebuild();
        hide_recycle_if_placed();
        rules::watch(tray);
        register_verb();
        if new {
            new_fence();
        }
        crate::whatsnew::show(existed);
        tray::updates::spawn(tray);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !fence::key(&msg) {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        tray::icon(tray, NIM_DELETE);
        save_now();
        APP.with(|a| a.borrow_mut().take());
    }
}
