use super::reg::reg_write;
use std::path::{Path, PathBuf};
use windows::{
    Win32::{
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::{
            Com::{CoTaskMemFree, IDataObject},
            Registry::REG_DWORD,
        },
        UI::Shell::*,
    },
    core::PCWSTR,
};

const CLSID: &str = "{645FF040-5081-101B-9F08-00AA002F954E}";
const HIDE: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\HideDesktopIcons\\NewStartPanel";

pub fn path() -> PathBuf {
    PathBuf::from(format!("::{CLSID}"))
}

pub fn is(p: &Path) -> bool {
    p == path()
}

pub fn info() -> (i32, Vec<u16>) {
    let mut i = SHFILEINFOW::default();
    unsafe {
        if let Ok(pidl) = SHGetKnownFolderIDList(&FOLDERID_RecycleBinFolder, 0, None) {
            let flags = SHGFI_PIDL | SHGFI_SYSICONINDEX | SHGFI_DISPLAYNAME;
            SHGetFileInfoW(PCWSTR(pidl as *const u16), FILE_FLAGS_AND_ATTRIBUTES(0), Some(&mut i), size_of::<SHFILEINFOW>() as u32, flags);
            CoTaskMemFree(Some(pidl as _));
        }
    }
    let n = i.szDisplayName.iter().position(|&c| c == 0).unwrap_or(0);
    (i.iIcon, i.szDisplayName[..n].to_vec())
}

pub fn in_data(data: &IDataObject) -> bool {
    let target = path();
    unsafe {
        let Ok(items) = SHCreateShellItemArrayFromDataObject::<_, IShellItemArray>(data) else { return false };
        (0..items.GetCount().unwrap_or(0)).any(|i| {
            items.GetItemAt(i).and_then(|it| it.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING)).is_ok_and(|name| {
                let same = name.to_string().is_ok_and(|n| Path::new(&n) == target);
                CoTaskMemFree(Some(name.0 as _));
                same
            })
        })
    }
}

pub fn show_on_desktop(show: bool) {
    if cfg!(debug_assertions) {
        return;
    }
    if !reg_write(HIDE, CLSID, REG_DWORD, &(!show as u32).to_le_bytes()) {
        crate::report::log("cannot change the desktop Recycle Bin visibility");
    }
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}
