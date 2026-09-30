mod dnd;
mod link;
mod reg;
mod watch;

use crate::win::wide_path;
pub use dnd::{DragImage, drag_out, pick_folder};
pub use link::link_into;
pub use reg::{NEW_ARG, set_autostart, set_desktop_verb};
use std::{
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};
pub use watch::Watch;
use windows::{
    Win32::{
        Globalization::GetUserDefaultLocaleName,
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::Com::CoTaskMemFree,
        UI::{
            Controls::{IImageList, ILD_TRANSPARENT},
            Shell::*,
            WindowsAndMessaging::{HICON, SW_SHOWNORMAL},
        },
    },
    core::*,
};

pub fn locale() -> String {
    let mut b = [0u16; 85];
    let n = unsafe { GetUserDefaultLocaleName(&mut b) } as usize;
    String::from_utf16_lossy(&b[..n.saturating_sub(1)])
}

fn known(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KNOWN_FOLDER_FLAG(0), None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as _));
        s.map(PathBuf::from)
    }
}

pub fn desktop() -> Option<PathBuf> {
    known(&FOLDERID_Desktop)
}

pub fn desktops() -> Vec<PathBuf> {
    [known(&FOLDERID_Desktop), known(&FOLDERID_PublicDesktop)].into_iter().flatten().collect()
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    a.as_os_str().eq_ignore_ascii_case(b.as_os_str()) || a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
}

pub fn recycle(p: &Path) -> bool {
    let from: Vec<u16> = p.as_os_str().encode_wide().chain([0, 0]).collect();
    let mut op = SHFILEOPSTRUCTW { wFunc: FO_DELETE, pFrom: PCWSTR(from.as_ptr()), fFlags: FOF_ALLOWUNDO.0 as u16, ..Default::default() };
    unsafe { SHFileOperationW(&mut op) == 0 && !op.fAnyOperationsAborted.as_bool() }
}

fn list(px: i32) -> i32 {
    (match px {
        ..=16 => SHIL_SMALL,
        17..=32 => SHIL_LARGE,
        33..=48 => SHIL_EXTRALARGE,
        _ => SHIL_JUMBO,
    }) as i32
}

pub fn info(p: &Path, px: i32) -> (HICON, Vec<u16>) {
    let mut i = SHFILEINFOW::default();
    let w = wide_path(p);
    let flags = SHGFI_SYSICONINDEX | SHGFI_DISPLAYNAME;
    unsafe { SHGetFileInfoW(PCWSTR(w.as_ptr()), FILE_FLAGS_AND_ATTRIBUTES(0), Some(&mut i), size_of::<SHFILEINFOW>() as u32, flags) };
    let n = i.szDisplayName.iter().position(|&c| c == 0).unwrap_or(i.szDisplayName.len());
    let icon = unsafe { SHGetImageList::<IImageList>(list(px)).and_then(|l| l.GetIcon(i.iIcon, ILD_TRANSPARENT.0)) };
    (icon.unwrap_or_default(), i.szDisplayName[..n].to_vec())
}

pub fn open(p: &Path) {
    let w = wide_path(p);
    unsafe { ShellExecuteW(None, w!("open"), PCWSTR(w.as_ptr()), None, None, SW_SHOWNORMAL) };
}
