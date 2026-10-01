use crate::win::wide_path;
use std::path::{Path, PathBuf};
use windows::{
    Win32::{
        System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, IPersistFile, STGM_READ},
        UI::Shell::{IShellLinkW, ShellLink},
    },
    core::*,
};

pub fn link_target(p: &Path) -> Option<String> {
    let ext = p.extension()?.to_ascii_lowercase();
    if ext == "url" {
        let text = std::fs::read_to_string(p).ok()?;
        return text.lines().find_map(|l| l.strip_prefix("URL=")).map(str::to_string);
    }
    if ext != "lnk" {
        return None;
    }
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        link.cast::<IPersistFile>().ok()?.Load(PCWSTR(wide_path(p).as_ptr()), STGM_READ).ok()?;
        let mut buf = [0u16; 1024];
        link.GetPath(&mut buf, std::ptr::null_mut(), 0).ok()?;
        let n = buf.iter().position(|&c| c == 0).unwrap_or(0);
        let path = String::from_utf16_lossy(&buf[..n]);
        let mut args = [0u16; 1024];
        let _ = link.GetArguments(&mut args);
        let m = args.iter().position(|&c| c == 0).unwrap_or(0);
        Some(format!("{path} {}", String::from_utf16_lossy(&args[..m])))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::same_path;
    use std::fs;
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};

    #[test]
    fn links_and_copies_shortcuts() {
        let d = std::env::temp_dir().join(format!("rdi-test-lnk-{}", std::process::id()));
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
