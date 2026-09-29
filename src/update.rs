use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, os::windows::process::CommandExt, path::PathBuf, process::Command};

const REPO: &str = "vincentxjoubert-lang/rust-desktop-icons";
const MAX_MSI: u64 = 64 << 20;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Current,
    Failed,
    Installing,
}

type Res<T> = Result<T, Box<dyn Error>>;

pub fn parse(v: &str) -> Option<[u64; 3]> {
    let mut it = v.trim().trim_start_matches('v').split('.').map(|p| p.parse().ok());
    let r = [it.next()??, it.next()??, it.next()??];
    it.next().is_none().then_some(r)
}

pub fn run() -> Outcome {
    match fetch() {
        Ok(None) => Outcome::Current,
        Ok(Some(msi)) => match install(&msi) {
            Ok(()) => Outcome::Installing,
            Err(_) => Outcome::Failed,
        },
        Err(_) => Outcome::Failed,
    }
}

fn get(url: &str, limit: u64) -> Res<Vec<u8>> {
    let prefix = format!("https://github.com/{REPO}/releases/download/");
    if !url.starts_with(&prefix) && !url.starts_with("https://api.github.com/") {
        return Err("untrusted url".into());
    }
    let mut r = ureq::get(url)
        .header("User-Agent", concat!("rust-desktop-icons/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call()?;
    Ok(r.body_mut().with_config().limit(limit).read_to_vec()?)
}

fn fetch() -> Res<Option<PathBuf>> {
    let rel: serde_json::Value = serde_json::from_slice(&get(&format!("https://api.github.com/repos/{REPO}/releases/latest"), 1 << 20)?)?;
    let latest = parse(rel["tag_name"].as_str().ok_or("no tag")?).ok_or("bad tag")?;
    if latest <= parse(env!("CARGO_PKG_VERSION")).ok_or("bad version")? {
        return Ok(None);
    }
    let asset = |suffix: &str| {
        rel["assets"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|a| a["name"].as_str().is_some_and(|n| n.ends_with(suffix)))
            .and_then(|a| a["browser_download_url"].as_str())
            .ok_or("missing asset")
    };
    let sum = String::from_utf8(get(asset(".msi.sha256")?, 1024)?)?;
    let msi = get(asset(".msi")?, MAX_MSI)?;
    if !verify(&msi, &sum) {
        return Err("checksum mismatch".into());
    }
    let path = env::temp_dir().join(format!("rust-desktop-icons-{}.{}.{}.msi", latest[0], latest[1], latest[2]));
    fs::write(&path, msi)?;
    Ok(Some(path))
}

fn verify(data: &[u8], sum: &str) -> bool {
    let hex: String = Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect();
    sum.split_whitespace().next().is_some_and(|s| s.eq_ignore_ascii_case(&hex))
}

fn install(msi: &PathBuf) -> Res<()> {
    let sys = env::var_os("SystemRoot").map(PathBuf::from).ok_or("no SystemRoot")?;
    Command::new(sys.join("System32").join("msiexec.exe"))
        .arg("/i")
        .arg(msi)
        .args(["/qn", "/norestart"])
        .creation_flags(0x0800_0000)
        .spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert_eq!(parse("v1.2.3"), Some([1, 2, 3]));
        assert_eq!(parse("0.10.0"), Some([0, 10, 0]));
        assert!(parse("v0.2.0") > parse("0.1.9"));
        assert_eq!(parse("1.2"), None);
        assert_eq!(parse("1.2.3.4"), None);
        assert_eq!(parse("1.x.3"), None);
    }

    #[test]
    fn checksum() {
        let sum = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  app.msi";
        assert!(verify(b"abc", sum));
        assert!(verify(b"abc", &sum.to_uppercase()));
        assert!(!verify(b"abd", sum));
        assert!(!verify(b"abc", ""));
    }
}
