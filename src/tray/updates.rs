use crate::{
    app::with,
    i18n::T,
    update::{self, Outcome},
    win::msgbox,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub const WM_UPDATE: u32 = WM_APP + 2;
static AUTO: AtomicBool = AtomicBool::new(false);

pub fn enable(on: bool) {
    AUTO.store(on, Ordering::Relaxed);
}

fn post(h: isize, r: Outcome, manual: bool) {
    unsafe { PostMessageW(Some(HWND(h as _)), WM_UPDATE, WPARAM(r as usize), LPARAM(manual as isize)).ok() };
}

pub fn spawn(h: HWND) {
    let h = h.0 as isize;
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));
        loop {
            if AUTO.load(Ordering::Relaxed) && update::run() == Outcome::Installing {
                return post(h, Outcome::Installing, false);
            }
            thread::sleep(Duration::from_secs(6 * 3600));
        }
    });
}

pub fn check_now(h: HWND) {
    let h = h.0 as isize;
    thread::spawn(move || post(h, update::run(), true));
}

pub fn done(code: usize, manual: bool) {
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
