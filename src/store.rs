use crate::domain::{Config, Tab};
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
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

pub fn load() -> (Config, Option<PathBuf>) {
    load_from(&root())
}

pub fn save(c: &Config) -> io::Result<()> {
    save_to(&root(), c)
}

fn parse(p: &Path) -> Option<Config> {
    serde_json::from_slice::<Config>(&fs::read(p).ok()?).ok()
}

fn load_from(dir: &Path) -> (Config, Option<PathBuf>) {
    let (main, bak) = (dir.join("config.json"), dir.join("config.json.bak"));
    if let Some(c) = parse(&main) {
        return (c.sanitized(), None);
    }
    let bad = main.exists().then(|| {
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let bad = dir.join(format!("config.json.bad-{secs}"));
        fs::rename(&main, &bad).map(|_| bad)
    });
    (parse(&bak).unwrap_or_default().sanitized(), bad.and_then(Result::ok))
}

fn save_to(dir: &Path, c: &Config) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let (tmp, main) = (dir.join("config.json.tmp"), dir.join("config.json"));
    fs::write(&tmp, serde_json::to_vec_pretty(c)?)?;
    if parse(&main).is_some() {
        fs::copy(&main, dir.join("config.json.bak"))?;
    }
    fs::rename(tmp, main)
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

fn copy_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.is_dir() {
        return fs::copy(src, dst).map(|_| ());
    }
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        copy_all(&e.path(), &dst.join(e.file_name()))?;
    }
    Ok(())
}

fn remove_all(p: &Path) -> io::Result<()> {
    if p.is_dir() { fs::remove_dir_all(p) } else { fs::remove_file(p) }
}

fn relocate(src: &Path, dst: &Path) -> io::Result<()> {
    let undo = |e: io::Error| {
        let _ = remove_all(dst);
        e
    };
    copy_all(src, dst).map_err(undo)?;
    remove_all(src).map_err(undo)
}

pub fn move_into(src: &Path, dir: &Path) -> io::Result<PathBuf> {
    let name = src.file_name().ok_or(io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(dir)?;
    let dst = unique(dir, name);
    fs::rename(src, &dst).or_else(|_| relocate(src, &dst))?;
    crate::shell::notify_moved(src, &dst);
    Ok(dst)
}

pub fn evacuate(dir: &Path, to: &Path) -> (Vec<PathBuf>, Vec<String>) {
    let (mut moved, mut errors) = (vec![], vec![]);
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        match move_into(&e.path(), to) {
            Ok(p) => moved.push(p),
            Err(err) => errors.push(format!("{} ({err})", name(&e.path()))),
        }
    }
    if errors.is_empty() {
        let _ = fs::remove_dir(dir);
    }
    (moved, errors)
}

pub fn name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
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
        assert_eq!(load_from(&d), (c.clone(), None));
        save_to(&d, &c).unwrap();
        fs::write(d.join("config.json"), "{broken").unwrap();
        let (restored, bad) = load_from(&d);
        assert_eq!(restored, c);
        assert_eq!(fs::read_to_string(bad.unwrap()).unwrap(), "{broken");
        assert!(!d.join("config.json").exists());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn relocation_across_volumes() {
        let d = tmp("reloc");
        let src = d.join("dir");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("sub").join("f.txt"), "x").unwrap();
        relocate(&src, &d.join("copy")).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read_to_string(d.join("copy").join("sub").join("f.txt")).unwrap(), "x");
        assert!(relocate(&d.join("missing"), &d.join("x")).is_err() && !d.join("x").exists());
        fs::create_dir_all(d.join("desk")).unwrap();
        fs::write(d.join("copy").join("g.txt"), "y").unwrap();
        let (moved, errors) = evacuate(&d.join("copy"), &d.join("desk"));
        assert_eq!((moved.len(), errors.len()), (2, 0));
        assert!(!d.join("copy").exists());
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
