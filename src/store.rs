use crate::domain::{Config, Tab};
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

pub fn root() -> PathBuf {
    env::var_os("LOCALAPPDATA").map_or_else(env::temp_dir, PathBuf::from).join("RustDesktopIcons")
}

pub fn fence_dir(id: u64) -> PathBuf {
    root().join("fences").join(id.to_string())
}

pub fn tab_dir(t: &Tab) -> PathBuf {
    t.portal.clone().unwrap_or_else(|| fence_dir(t.id))
}

pub fn load() -> Config {
    load_from(&root())
}

pub fn save(c: &Config) -> io::Result<()> {
    save_to(&root(), c)
}

fn load_from(dir: &Path) -> Config {
    fs::read(dir.join("config.json")).ok().and_then(|b| serde_json::from_slice::<Config>(&b).ok()).unwrap_or_default().sanitized()
}

fn save_to(dir: &Path, c: &Config) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join("config.json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(c)?)?;
    fs::rename(tmp, dir.join("config.json"))
}

pub fn unique(dir: &Path, name: &OsStr) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let p = Path::new(name);
    let stem = p.file_stem().unwrap_or(name).to_string_lossy();
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    (2..).map(|n| dir.join(format!("{stem} ({n}){ext}"))).find(|p| !p.exists()).unwrap_or(first)
}

pub fn move_into(src: &Path, dir: &Path) -> io::Result<PathBuf> {
    let name = src.file_name().ok_or(io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(dir)?;
    let dst = unique(dir, name);
    fs::rename(src, &dst).or_else(|e| {
        if !src.is_file() {
            return Err(e);
        }
        fs::copy(src, &dst)?;
        fs::remove_file(src)
    })?;
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Fence, Look};

    fn tmp(tag: &str) -> PathBuf {
        let d = env::temp_dir().join(format!("rdi-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn save_then_load() {
        let d = tmp("cfg");
        let mut c = Config::default();
        c.fences.push(Fence::new(7, "Apps", (100, 100, 360, 240), Look::default()));
        save_to(&d, &c).unwrap();
        assert_eq!(load_from(&d), c);
        fs::write(d.join("config.json"), "{broken").unwrap();
        assert_eq!(load_from(&d), Config::default());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn move_with_unique_names() {
        let d = tmp("mv");
        let (a, b) = (d.join("a"), d.join("b"));
        fs::create_dir_all(&a).unwrap();
        fs::write(a.join("x.txt"), "1").unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(b.join("x.txt"), "0").unwrap();
        let dst = move_into(&a.join("x.txt"), &b).unwrap();
        assert_eq!(dst, b.join("x (2).txt"));
        assert_eq!(fs::read_to_string(dst).unwrap(), "1");
        assert!(!a.join("x.txt").exists());
        fs::remove_dir_all(d).unwrap();
    }
}
