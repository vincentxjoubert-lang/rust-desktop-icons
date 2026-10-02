use crate::{app::with, i18n::T, store, win::msgbox};
use std::{fs::OpenOptions, io::Write};
use windows::Win32::{
    System::SystemInformation::GetLocalTime,
    UI::WindowsAndMessaging::{MB_ICONWARNING, MB_OK},
};

fn stamp() -> String {
    let t = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
}

fn append(file: &str, msg: &str) {
    let dir = store::root();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(file);
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 1 << 20) {
        let _ = std::fs::rename(&path, dir.join(format!("{file}.old")));
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "[{}] v{} {msg}", stamp(), env!("CARGO_PKG_VERSION"));
    }
}

pub fn log(msg: &str) {
    append("errors.log", msg);
}

pub fn alert(k: T, detail: &str) {
    let (text, rtl) = with(|a| (a.t(k).replace("{}", detail), a.rtl())).unwrap_or_else(|| (detail.to_string(), false));
    log(&text);
    msgbox(None, &text, MB_OK | MB_ICONWARNING, rtl);
}

pub fn failures(k: T, errors: &[String]) {
    if !errors.is_empty() {
        let shown: Vec<&str> = errors.iter().take(10).map(String::as_str).collect();
        alert(k, &shown.join("\n"));
    }
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let bt = std::backtrace::Backtrace::force_capture();
        append("crash.log", &format!("{info}\n{bt}"));
    }));
}
