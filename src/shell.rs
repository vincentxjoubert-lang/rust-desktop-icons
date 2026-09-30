use crate::win::{wide, wide_path};
use std::{
    env,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::HWND,
        Globalization::GetUserDefaultLocaleName,
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree, IPersistFile},
            Registry::*,
        },
        UI::{
            Controls::{IImageList, ILD_TRANSPARENT},
            Shell::{Common::ITEMIDLIST, *},
            WindowsAndMessaging::{HICON, SW_SHOWNORMAL},
        },
    },
    core::*,
};

const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const NAME: &str = "RustDesktopIcons";
const VERB: &str = "Software\\Classes\\DesktopBackground\\Shell\\RustDesktopIcons";
pub const NEW_ARG: &str = "--new";

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

pub fn open(p: &Path) {
    let w = wide_path(p);
    unsafe { ShellExecuteW(None, w!("open"), PCWSTR(w.as_ptr()), None, None, SW_SHOWNORMAL) };
}

fn exe() -> Option<String> {
    env::current_exe().ok().map(|e| e.display().to_string())
}

fn reg_set(key: &str, name: &str, value: &str) {
    let (k, n, v) = (wide(key), wide(name), wide(value));
    let _ = unsafe {
        RegSetKeyValueW(HKEY_CURRENT_USER, PCWSTR(k.as_ptr()), PCWSTR(n.as_ptr()), REG_SZ.0, Some(v.as_ptr().cast()), (v.len() * 2) as u32)
    };
}

pub fn set_autostart(on: bool) {
    if !on {
        let (k, n) = (wide(RUN), wide(NAME));
        let _ = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, PCWSTR(k.as_ptr()), PCWSTR(n.as_ptr())) };
    } else if let Some(exe) = exe() {
        reg_set(RUN, NAME, &format!("\"{exe}\""));
    }
}

pub fn set_desktop_verb(label: &str) {
    let Some(exe) = exe() else { return };
    reg_set(VERB, "MUIVerb", label);
    reg_set(VERB, "Icon", &format!("\"{exe}\",0"));
    reg_set(&format!("{VERB}\\command"), "", &format!("\"{exe}\" {NEW_ARG}"));
}

pub fn link_into(target: &Path, dir: &Path) -> Result<PathBuf> {
    let ext = target.extension().map(|e| e.to_ascii_lowercase());
    if ext.as_ref().is_some_and(|e| e == "lnk" || e == "url") {
        let dst = dir.join(target.file_name().unwrap_or_default());
        if !dst.exists() {
            std::fs::copy(target, &dst)?;
        }
        return Ok(dst);
    }
    let name = format!("{}.lnk", target.file_name().unwrap_or_default().to_string_lossy());
    let dst = dir.join(name);
    if dst.exists() {
        return Ok(dst);
    }
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        link.SetPath(PCWSTR(wide_path(target).as_ptr()))?;
        if let Some(parent) = target.parent() {
            link.SetWorkingDirectory(PCWSTR(wide_path(parent).as_ptr()))?;
        }
        link.cast::<IPersistFile>()?.Save(PCWSTR(wide_path(&dst).as_ptr()), true)?;
    }
    Ok(dst)
}

pub struct Watch(u32, *mut ITEMIDLIST);

impl Watch {
    pub fn new(h: HWND, dir: &Path, msg: u32) -> Option<Self> {
        let w = wide_path(dir);
        unsafe {
            let pidl = ILCreateFromPathW(PCWSTR(w.as_ptr()));
            if pidl.is_null() {
                return None;
            }
            let entry = SHChangeNotifyEntry { pidl, fRecursive: false.into() };
            let id = SHChangeNotifyRegister(h, SHCNRF_ShellLevel | SHCNRF_InterruptLevel, SHCNE_ALLEVENTS.0 as i32, msg, 1, &entry);
            if id == 0 {
                ILFree(Some(pidl));
                return None;
            }
            Some(Self(id, pidl))
        }
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        unsafe {
            let _ = SHChangeNotifyDeregister(self.0);
            ILFree(Some(self.1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};

    #[test]
    fn links_and_copies_shortcuts() {
        let d = env::temp_dir().join(format!("rdi-test-lnk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        let target = d.join("app.txt");
        fs::write(&target, "x").unwrap();
        let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let out = d.join("out");
        fs::create_dir_all(&out).unwrap();
        let lnk = link_into(&target, &out).unwrap();
        assert_eq!(lnk, out.join("app.txt.lnk"));
        assert!(fs::metadata(&lnk).unwrap().len() > 0);
        assert_eq!(link_into(&target, &out).unwrap(), lnk);
        assert_eq!(fs::read_dir(&out).unwrap().count(), 1);
        assert!(same_path(&out, Path::new(&out.to_string_lossy().to_uppercase())));
        assert!(target.exists());
        fs::remove_dir_all(d).unwrap();
    }
}
