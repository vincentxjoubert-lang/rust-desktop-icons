use crate::win::{wide, wide_path};
use std::{
    env,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::HWND,
        Globalization::GetUserDefaultLocaleName,
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::{Com::CoTaskMemFree, Registry::*},
        UI::{
            Controls::{IImageList, ILD_TRANSPARENT},
            Shell::{Common::ITEMIDLIST, *},
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
    let flags = SHGFI_SYSICONINDEX | SHGFI_DISPLAYNAME;
    unsafe { SHGetFileInfoW(PCWSTR(w.as_ptr()), FILE_FLAGS_AND_ATTRIBUTES(0), Some(&mut i), size_of::<SHFILEINFOW>() as u32, flags) };
    let n = i.szDisplayName.iter().position(|&c| c == 0).unwrap_or(i.szDisplayName.len());
    let icon = unsafe { SHGetImageList::<IImageList>(SHIL_EXTRALARGE as i32).and_then(|l| l.GetIcon(i.iIcon, ILD_TRANSPARENT.0)) };
    (icon.unwrap_or_default(), i.szDisplayName[..n].to_vec())
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

pub struct Watch(u32, *mut ITEMIDLIST);

impl Watch {
    pub fn new(h: HWND, dir: &Path, msg: u32) -> Option<Self> {
        let w = wide_path(dir);
        unsafe {
            let pidl = ILCreateFromPathW(PCWSTR(w.as_ptr()));
            if pidl.is_null() {
                return None;
            }
            let entry = SHChangeNotifyEntry { pidl, fRecursive: false.into() };
            let id = SHChangeNotifyRegister(h, SHCNRF_ShellLevel | SHCNRF_InterruptLevel, SHCNE_ALLEVENTS.0 as i32, msg, 1, &entry);
            if id == 0 {
                ILFree(Some(pidl));
                return None;
            }
            Some(Self(id, pidl))
        }
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        unsafe {
            let _ = SHChangeNotifyDeregister(self.0);
            ILFree(Some(self.1));
        }
    }
}
