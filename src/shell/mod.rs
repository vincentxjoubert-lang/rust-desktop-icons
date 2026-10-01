pub mod clip;
mod data;
mod dnd;
mod link;
pub mod menu;
mod ops;
pub mod recycle;
mod reg;
mod source;
mod watch;

use crate::win::wide_path;
pub use data::files;
pub use dnd::{DragImage, drag_out, pick_folder};
pub use link::{link_into, link_target};
pub use ops::{delete, transfer};
pub use reg::{NEW_ARG, UNINSTALL_ARG, set_autostart, set_desktop_verb};
use std::path::{Path, PathBuf};
pub use watch::{Watch, notify_moved};
use windows::{
    Win32::{
        Globalization::GetUserDefaultLocaleName,
        Graphics::Gdi::HDC,
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::Com::CoTaskMemFree,
        UI::{
            Controls::{IImageList, ILD_TRANSPARENT},
            Shell::*,
            WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx, SW_SHOWNORMAL},
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
    let text = |p: &Path| p.to_string_lossy().trim_end_matches('\\').to_lowercase();
    text(a) == text(b) || matches!((std::fs::canonicalize(a), std::fs::canonicalize(b)), (Ok(x), Ok(y)) if text(&x) == text(&y))
}

fn list(px: i32) -> i32 {
    (match px {
        ..=16 => SHIL_SMALL,
        17..=32 => SHIL_LARGE,
        33..=48 => SHIL_EXTRALARGE,
        _ => SHIL_JUMBO,
    }) as i32
}

pub fn info(p: &Path) -> (i32, Vec<u16>) {
    let mut i = SHFILEINFOW::default();
    let w = wide_path(p);
    let flags = SHGFI_SYSICONINDEX | SHGFI_DISPLAYNAME;
    unsafe { SHGetFileInfoW(PCWSTR(w.as_ptr()), FILE_FLAGS_AND_ATTRIBUTES(0), Some(&mut i), size_of::<SHFILEINFOW>() as u32, flags) };
    let n = i.szDisplayName.iter().position(|&c| c == 0).unwrap_or(i.szDisplayName.len());
    (i.iIcon, i.szDisplayName[..n].to_vec())
}

pub fn draw_icon(dc: HDC, index: i32, (x, y): (i32, i32), px: i32) {
    unsafe {
        if let Ok(icon) = SHGetImageList::<IImageList>(list(px)).and_then(|l| l.GetIcon(index, ILD_TRANSPARENT.0)) {
            let _ = DrawIconEx(dc, x, y, icon, px, px, 0, None, DI_NORMAL);
            let _ = DestroyIcon(icon);
        }
    }
}

pub fn open(p: &Path) {
    let w = if recycle::is(p) { wide_path(Path::new("shell:RecycleBinFolder")) } else { wide_path(p) };
    unsafe { ShellExecuteW(None, w!("open"), PCWSTR(w.as_ptr()), None, None, SW_SHOWNORMAL) };
}
