use super::{drop, peek, tabs, to_client};
use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::{HWND, POINT, POINTL},
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, DVASPECT_CONTENT, FORMATETC, IDataObject, TYMED_HGLOBAL},
            Ole::*,
            SystemServices::MODIFIERKEYS_FLAGS,
        },
        UI::Shell::{CLSID_DragDropHelper, HDROP, IDropTargetHelper},
    },
    core::{Ref, Result, implement},
};

const CF_HDROP: u16 = 15;

#[implement(IDropTarget)]
struct Target {
    hwnd: isize,
    helper: Option<IDropTargetHelper>,
}

impl Target {
    fn h(&self) -> HWND {
        HWND(self.hwnd as _)
    }

    fn hover(&self, pt: &POINTL) {
        peek::enter(self.h());
        if let Some(i) = tabs::at(self.h(), to_client(self.h(), (pt.x, pt.y))) {
            tabs::select(self.h(), i);
        }
    }

    fn effect(allowed: DROPEFFECT) -> DROPEFFECT {
        [DROPEFFECT_MOVE, DROPEFFECT_COPY, DROPEFFECT_LINK].into_iter().find(|e| allowed.0 & e.0 != 0).unwrap_or(DROPEFFECT_NONE)
    }
}

fn paths(data: &IDataObject) -> Vec<PathBuf> {
    let fmt = FORMATETC {
        cfFormat: CF_HDROP,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    let Ok(mut medium) = (unsafe { data.GetData(&fmt) }) else { return vec![] };
    let out = drop::paths(HDROP(unsafe { medium.u.hGlobal.0 }));
    unsafe { ReleaseStgMedium(&mut medium) };
    out
}

impl IDropTarget_Impl for Target_Impl {
    fn DragEnter(&self, data: Ref<IDataObject>, _: MODIFIERKEYS_FLAGS, pt: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        unsafe {
            *effect = Target::effect(*effect);
            if let (Some(helper), Some(data)) = (&self.helper, data.as_ref()) {
                let _ = helper.DragEnter(self.h(), data, &POINT { x: pt.x, y: pt.y }, *effect);
            }
        }
        self.hover(pt);
        Ok(())
    }

    fn DragOver(&self, _: MODIFIERKEYS_FLAGS, pt: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        unsafe {
            *effect = Target::effect(*effect);
            if let Some(helper) = &self.helper {
                let _ = helper.DragOver(&POINT { x: pt.x, y: pt.y }, *effect);
            }
        }
        self.hover(pt);
        Ok(())
    }

    fn DragLeave(&self) -> Result<()> {
        if let Some(helper) = &self.helper {
            let _ = unsafe { helper.DragLeave() };
        }
        Ok(())
    }

    fn Drop(&self, data: Ref<IDataObject>, _: MODIFIERKEYS_FLAGS, pt: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        if let (Some(helper), Some(d)) = (&self.helper, data.as_ref()) {
            let _ = unsafe { helper.Drop(d, &POINT { x: pt.x, y: pt.y }, *effect) };
        }
        let list = data.as_ref().map(paths).unwrap_or_default();
        unsafe { *effect = DROPEFFECT_NONE };
        drop::files(self.h(), list, to_client(self.h(), (pt.x, pt.y)));
        Ok(())
    }
}

pub(super) fn register(h: HWND) {
    let helper = unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) }.ok();
    let target: IDropTarget = Target { hwnd: h.0 as isize, helper }.into();
    let _ = unsafe { RegisterDragDrop(h, &target) };
}

pub(super) fn revoke(h: HWND) {
    let _ = unsafe { RevokeDragDrop(h) };
}
