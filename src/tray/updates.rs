use crate::{
    app::with,
    i18n::T,
    report,
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
static BUSY: AtomicBool = AtomicBool::new(false);

pub fn enable(on: bool) {
    AUTO.store(on, Ordering::Relaxed);
}

fn check(h: isize, manual: bool) {
    if BUSY.swap(true, Ordering::AcqRel) {
        return;
    }
    let r = update::run();
    BUSY.store(false, Ordering::Release);
    if manual || r != Outcome::Current {
        let _ = unsafe { PostMessageW(Some(HWND(h as _)), WM_UPDATE, WPARAM(r.encode()), LPARAM(manual as isize)) };
    }
}

pub fn spawn(h: HWND) {
    let h = h.0 as isize;
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));
        loop {
            if AUTO.load(Ordering::Relaxed) {
                check(h, false);
            }
            thread::sleep(Duration::from_secs(6 * 3600));
        }
    });
}

pub fn check_now(h: HWND) {
    let h = h.0 as isize;
    thread::spawn(move || check(h, true));
}

pub fn done(code: usize, manual: bool) {
    match Outcome::decode(code) {
        Outcome::InstallFailed(c) => report::alert(T::ErrInstall, &c.to_string()),
        Outcome::Installed => {}
        r if manual => {
            let Some((ok, fail, rtl)) = with(|a| (a.t(T::UpToDate), a.t(T::UpdateFailed), a.rtl())) else { return };
            let (text, icon) = if r == Outcome::Current { (ok, MB_ICONINFORMATION) } else { (fail, MB_ICONWARNING) };
            msgbox(None, text, MB_OK | icon, rtl);
        }
        _ => {}
    }
}
