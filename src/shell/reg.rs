use crate::win::wide;
use std::env;
use windows::{Win32::System::Registry::*, core::PCWSTR};

const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const NAME: &str = "RustDesktopIcons";
const VERB: &str = "Software\\Classes\\DesktopBackground\\Shell\\RustDesktopIcons";
pub const NEW_ARG: &str = "--new";

fn exe() -> Option<String> {
    env::current_exe().ok().map(|e| e.display().to_string())
}

fn reg_set(key: &str, name: &str, value: &str) {
    let (k, n, v) = (wide(key), wide(name), wide(value));
    let _ = unsafe {
        RegSetKeyValueW(HKEY_CURRENT_USER, PCWSTR(k.as_ptr()), PCWSTR(n.as_ptr()), REG_SZ.0, Some(v.as_ptr().cast()), (v.len() * 2) as u32)
    };
}

pub fn set_autostart(on: bool) {
    if !on {
        let (k, n) = (wide(RUN), wide(NAME));
        let _ = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, PCWSTR(k.as_ptr()), PCWSTR(n.as_ptr())) };
    } else if let Some(exe) = exe() {
        reg_set(RUN, NAME, &format!("\"{exe}\""));
    }
}

pub fn set_desktop_verb(label: &str) {
    let Some(exe) = exe() else { return };
    reg_set(VERB, "MUIVerb", label);
    reg_set(VERB, "Icon", &format!("\"{exe}\",0"));
    reg_set(&format!("{VERB}\\command"), "", &format!("\"{exe}\" {NEW_ARG}"));
}
