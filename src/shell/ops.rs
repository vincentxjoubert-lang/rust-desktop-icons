use std::{os::windows::ffi::OsStrExt, path::Path, path::PathBuf};
use windows::{
    Win32::{Foundation::HWND, UI::Shell::*},
    core::PCWSTR,
};

fn list(paths: &[PathBuf]) -> Vec<u16> {
    paths.iter().flat_map(|p| p.as_os_str().encode_wide().chain([0])).chain([0]).collect()
}

fn run(h: Option<HWND>, func: u32, paths: &[PathBuf], to: Option<&Path>, flags: FILEOPERATION_FLAGS) -> bool {
    if paths.is_empty() {
        return false;
    }
    let (from, dest) = (list(paths), to.map(|d| list(&[d.to_path_buf()])));
    let mut op = SHFILEOPSTRUCTW {
        hwnd: h.unwrap_or_default(),
        wFunc: func,
        pFrom: PCWSTR(from.as_ptr()),
        pTo: dest.as_ref().map_or(PCWSTR::null(), |d| PCWSTR(d.as_ptr())),
        fFlags: flags.0 as u16,
        ..Default::default()
    };
    unsafe { SHFileOperationW(&mut op) == 0 && !op.fAnyOperationsAborted.as_bool() }
}

pub fn delete(h: HWND, paths: &[PathBuf], permanent: bool) -> bool {
    run(Some(h), FO_DELETE, paths, None, if permanent { FILEOPERATION_FLAGS(0) } else { FOF_ALLOWUNDO })
}

pub fn transfer(h: HWND, paths: &[PathBuf], dest: &Path, cut: bool) -> bool {
    run(Some(h), if cut { FO_MOVE } else { FO_COPY }, paths, Some(dest), FOF_ALLOWUNDO | FOF_RENAMEONCOLLISION)
}
