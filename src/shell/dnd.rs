use crate::win::wide_path;
use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, POINT, SIZE},
        Graphics::Gdi::{DeleteObject, HBITMAP},
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree, IDataObject},
            Ole::{DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE, IDropSource},
        },
        UI::Shell::{Common::ITEMIDLIST, *},
    },
    core::{PCWSTR, Result},
};

pub fn pick_folder(h: HWND) -> Option<PathBuf> {
    unsafe {
        let d: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
        d.SetOptions(d.GetOptions().ok()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM).ok()?;
        d.Show(Some(h)).ok()?;
        let p = d.GetResult().ok()?.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as _));
        s.map(PathBuf::from)
    }
}

pub struct DragImage {
    pub bitmap: HBITMAP,
    pub size: (i32, i32),
    pub offset: (i32, i32),
}

fn attach(data: &IDataObject, img: DragImage) {
    let shdi = SHDRAGIMAGE {
        sizeDragImage: SIZE { cx: img.size.0, cy: img.size.1 },
        ptOffset: POINT { x: img.offset.0, y: img.offset.1 },
        hbmpDragImage: img.bitmap,
        crColorKey: COLORREF(0xFFFF_FFFF),
    };
    let helper: Result<IDragSourceHelper2> = unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) };
    let r = helper.and_then(|h| unsafe {
        let _ = h.SetFlags(DSH_ALLOWDROPDESCRIPTIONTEXT.0 as u32);
        h.InitializeFromBitmap(&shdi, data)
    });
    if r.is_err() {
        let _ = unsafe { DeleteObject(img.bitmap.into()) };
    }
}

pub fn drag_out(h: HWND, paths: &[PathBuf], image: Option<DragImage>) {
    unsafe {
        let pidls: Vec<*mut ITEMIDLIST> =
            paths.iter().map(|p| ILCreateFromPathW(PCWSTR(wide_path(p).as_ptr()))).filter(|p| !p.is_null()).collect();
        let raw: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| *p as *const _).collect();
        let data = SHCreateShellItemArrayFromIDLists(&raw).and_then(|a| a.BindToHandler::<_, IDataObject>(None, &BHID_DataObject));
        if let Ok(data) = data {
            if let Some(img) = image {
                attach(&data, img);
            }
            let _ = SHDoDragDrop(Some(h), &data, None::<&IDropSource>, DROPEFFECT_MOVE | DROPEFFECT_COPY | DROPEFFECT_LINK);
        }
        for p in pidls {
            ILFree(Some(p));
        }
    }
}
