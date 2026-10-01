use crate::win::{cursor_pos, desktop_at};
use windows::{
    Win32::{
        Foundation::{DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS, S_OK},
        System::{
            Ole::{DROPEFFECT, DROPEFFECT_NONE, IDropSource, IDropSource_Impl},
            SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS},
        },
        UI::WindowsAndMessaging::{IDC_ARROW, LoadCursorW, SetCursor},
    },
    core::{BOOL, HRESULT, implement},
};

#[implement(IDropSource)]
struct DesktopFriendly;

impl IDropSource_Impl for DesktopFriendly_Impl {
    fn QueryContinueDrag(&self, escape: BOOL, keys: MODIFIERKEYS_FLAGS) -> HRESULT {
        if escape.as_bool() {
            DRAGDROP_S_CANCEL
        } else if keys.0 & MK_LBUTTON.0 == 0 {
            DRAGDROP_S_DROP
        } else {
            S_OK
        }
    }

    fn GiveFeedback(&self, effect: DROPEFFECT) -> HRESULT {
        if effect != DROPEFFECT_NONE || !desktop_at(cursor_pos()) {
            return DRAGDROP_S_USEDEFAULTCURSORS;
        }
        unsafe { SetCursor(LoadCursorW(None, IDC_ARROW).ok()) };
        S_OK
    }
}

pub fn desktop_friendly() -> IDropSource {
    DesktopFriendly.into()
}
