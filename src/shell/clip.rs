use super::data;
use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::HGLOBAL,
        System::{
            Com::STGMEDIUM,
            DataExchange::RegisterClipboardFormatW,
            Memory::{GlobalLock, GlobalUnlock},
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
    let effect = if cut { DROPEFFECT_MOVE.0 } else { DROPEFFECT_COPY.0 };
    data::set_global(&obj, effect_format(), &effect.to_le_bytes());
    unsafe { OleSetClipboard(&obj).is_ok() }
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
