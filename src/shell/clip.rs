use super::data;
use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::{GlobalFree, HGLOBAL},
        System::{
            Com::{STGMEDIUM, STGMEDIUM_0, TYMED_HGLOBAL},
            DataExchange::RegisterClipboardFormatW,
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::{DROPEFFECT_COPY, DROPEFFECT_MOVE, OleGetClipboard, OleSetClipboard, ReleaseStgMedium},
        },
    },
    core::w,
};

fn effect_format() -> u16 {
    unsafe { RegisterClipboardFormatW(w!("Preferred DropEffect")) as u16 }
}

pub fn set(paths: &[PathBuf], cut: bool) -> bool {
    let Some(obj) = data::object(paths) else { return false };
    unsafe {
        if let Ok(g) = GlobalAlloc(GMEM_MOVEABLE, 4) {
            *(GlobalLock(g) as *mut u32) = if cut { DROPEFFECT_MOVE.0 } else { DROPEFFECT_COPY.0 };
            let _ = GlobalUnlock(g);
            let medium = STGMEDIUM { tymed: TYMED_HGLOBAL.0 as u32, u: STGMEDIUM_0 { hGlobal: g }, ..Default::default() };
            if obj.SetData(&data::format(effect_format()), &medium, true).is_err() {
                let _ = GlobalFree(Some(g));
            }
        }
        OleSetClipboard(&obj).is_ok()
    }
}

pub fn get() -> Option<(Vec<PathBuf>, bool)> {
    unsafe {
        let obj = OleGetClipboard().ok()?;
        let files = data::files(&obj);
        let cut = obj.GetData(&data::format(effect_format())).ok().is_some_and(|mut m: STGMEDIUM| {
            let g: HGLOBAL = m.u.hGlobal;
            let p = GlobalLock(g) as *const u32;
            let v = !p.is_null() && *p & DROPEFFECT_MOVE.0 != 0;
            let _ = GlobalUnlock(g);
            ReleaseStgMedium(&mut m);
            v
        });
        (!files.is_empty()).then_some((files, cut))
    }
}
