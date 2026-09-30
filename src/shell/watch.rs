use crate::win::wide_path;
use std::path::Path;
use windows::{
    Win32::{
        Foundation::HWND,
        UI::Shell::{Common::ITEMIDLIST, *},
    },
    core::PCWSTR,
};

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
