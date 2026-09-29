use crate::win::{wide, wide_path};
use std::{
    env,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Globalization::GetUserDefaultLocaleName,
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::{Com::CoTaskMemFree, Registry::*},
        UI::{
            Shell::*,
            WindowsAndMessaging::{HICON, SW_SHOWNORMAL},
        },
    },
    core::*,
};

const RUN: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const NAME: PCWSTR = w!("RustDesktopIcons");

pub fn locale() -> String {
    let mut b = [0u16; 85];
    let n = unsafe { GetUserDefaultLocaleName(&mut b) } as usize;
    String::from_utf16_lossy(&b[..n.saturating_sub(1)])
}

pub fn desktop() -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Desktop, KNOWN_FOLDER_FLAG(0), None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as _));
        s.map(PathBuf::from)
    }
}

pub fn info(p: &Path) -> (HICON, Vec<u16>) {
    let mut i = SHFILEINFOW::default();
    let w = wide_path(p);
    unsafe {
        SHGetFileInfoW(
            PCWSTR(w.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut i),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | SHGFI_DISPLAYNAME,
        )
    };
    let n = i.szDisplayName.iter().position(|&c| c == 0).unwrap_or(i.szDisplayName.len());
    (i.hIcon, i.szDisplayName[..n].to_vec())
}

pub fn open(p: &Path) {
    let w = wide_path(p);
    unsafe { ShellExecuteW(None, w!("open"), PCWSTR(w.as_ptr()), None, None, SW_SHOWNORMAL) };
}

pub fn set_autostart(on: bool) {
    unsafe {
        if !on {
            let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN, NAME);
            return;
        }
        let Ok(exe) = env::current_exe() else { return };
        let v = wide(&format!("\"{}\"", exe.display()));
        let _ = RegSetKeyValueW(HKEY_CURRENT_USER, RUN, NAME, REG_SZ.0, Some(v.as_ptr().cast()), (v.len() * 2) as u32);
    }
}
