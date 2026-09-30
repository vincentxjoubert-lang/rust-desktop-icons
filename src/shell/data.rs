use crate::win::wide_path;
use std::path::PathBuf;
use windows::{
    Win32::{
        System::{
            Com::{DVASPECT_CONTENT, FORMATETC, IDataObject, TYMED_HGLOBAL},
            Ole::ReleaseStgMedium,
        },
        UI::Shell::{Common::ITEMIDLIST, *},
    },
    core::{Interface, PCWSTR},
};

const CF_HDROP: u16 = 15;

pub fn format(cf: u16) -> FORMATETC {
    FORMATETC { cfFormat: cf, ptd: std::ptr::null_mut(), dwAspect: DVASPECT_CONTENT.0, lindex: -1, tymed: TYMED_HGLOBAL.0 as u32 }
}

pub fn items<T: Interface>(paths: &[PathBuf], handler: &windows::core::GUID) -> Option<T> {
    unsafe {
        let pidls: Vec<*mut ITEMIDLIST> =
            paths.iter().map(|p| ILCreateFromPathW(PCWSTR(wide_path(p).as_ptr()))).filter(|p| !p.is_null()).collect();
        let raw: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| *p as *const _).collect();
        let out =
            (!raw.is_empty()).then(|| SHCreateShellItemArrayFromIDLists(&raw).and_then(|a| a.BindToHandler::<_, T>(None, handler)).ok());
        for p in pidls {
            ILFree(Some(p));
        }
        out.flatten()
    }
}

pub fn object(paths: &[PathBuf]) -> Option<IDataObject> {
    items(paths, &BHID_DataObject)
}

pub fn dropped(d: HDROP) -> Vec<PathBuf> {
    let n = unsafe { DragQueryFileW(d, u32::MAX, None) };
    (0..n)
        .map(|i| {
            let mut b = vec![0u16; unsafe { DragQueryFileW(d, i, None) } as usize + 1];
            let len = unsafe { DragQueryFileW(d, i, Some(&mut b)) } as usize;
            PathBuf::from(String::from_utf16_lossy(&b[..len]))
        })
        .collect()
}

pub fn files(data: &IDataObject) -> Vec<PathBuf> {
    let Ok(mut medium) = (unsafe { data.GetData(&format(CF_HDROP)) }) else { return vec![] };
    let out = dropped(HDROP(unsafe { medium.u.hGlobal.0 }));
    unsafe { ReleaseStgMedium(&mut medium) };
    out
}
