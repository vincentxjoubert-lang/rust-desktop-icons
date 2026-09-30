use super::data;
use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        UI::{Shell::*, WindowsAndMessaging::*},
    },
    core::{Interface, PCSTR, PSTR},
};

const FIRST: u32 = 1;
const CMIC_MASK_UNICODE: u32 = 0x4000;

pub struct Native {
    pub cm: IContextMenu,
    menu: HMENU,
}

impl Drop for Native {
    fn drop(&mut self) {
        let _ = unsafe { DestroyMenu(self.menu) };
    }
}

pub fn build(paths: &[PathBuf], extended: bool) -> Option<Native> {
    let cm: IContextMenu = data::items(paths, &BHID_SFUIObject)?;
    let menu = unsafe { CreatePopupMenu() }.ok()?;
    let flags = CMF_NORMAL | CMF_CANRENAME | if extended { CMF_EXTENDEDVERBS } else { 0 };
    unsafe { cm.QueryContextMenu(menu, 0, FIRST, 0x7FFF, flags) }.ok().ok()?;
    Some(Native { cm, menu })
}

impl Native {
    pub fn track(&self, h: HWND, (x, y): (i32, i32)) -> Option<u32> {
        let _ = unsafe { SetForegroundWindow(h) };
        let id = unsafe { TrackPopupMenuEx(self.menu, (TPM_RETURNCMD | TPM_RIGHTBUTTON).0, x, y, h, None) }.0 as u32;
        (id >= FIRST).then(|| id - FIRST)
    }

    pub fn verb(&self, id: u32) -> String {
        let mut b = [0u16; 64];
        let ok = unsafe { self.cm.GetCommandString(id as usize, GCS_VERBW, None, PSTR(b.as_mut_ptr().cast()), b.len() as u32) };
        ok.map(|_| String::from_utf16_lossy(&b[..b.iter().position(|&c| c == 0).unwrap_or(0)])).unwrap_or_default()
    }

    pub fn invoke(&self, id: u32, h: HWND, (x, y): (i32, i32)) {
        let info = CMINVOKECOMMANDINFOEX {
            cbSize: size_of::<CMINVOKECOMMANDINFOEX>() as u32,
            fMask: CMIC_MASK_UNICODE | CMIC_MASK_PTINVOKE,
            hwnd: h,
            lpVerb: PCSTR(id as usize as *const u8),
            lpVerbW: windows::core::PCWSTR(id as usize as *const u16),
            nShow: SW_SHOWNORMAL.0,
            ptInvoke: POINT { x, y },
            ..Default::default()
        };
        let _ = unsafe { self.cm.InvokeCommand(&info as *const _ as *const CMINVOKECOMMANDINFO) };
    }
}

pub fn forward(cm: &IContextMenu, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
    if let Ok(c3) = cm.cast::<IContextMenu3>() {
        let mut r = LRESULT(0);
        return unsafe { c3.HandleMenuMsg2(msg, wp, lp, Some(&mut r)) }.ok().map(|_| r);
    }
    let c2 = cm.cast::<IContextMenu2>().ok()?;
    unsafe { c2.HandleMenuMsg(msg, wp, lp) }.ok().map(|_| LRESULT(0))
}
