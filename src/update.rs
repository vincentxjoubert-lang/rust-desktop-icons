use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write, os::windows::process::CommandExt, path::PathBuf, process::Command, time::Duration};

const REPO: &str = "vincentxjoubert-lang/rust-desktop-icons";
const MAX_MSI: u64 = 64 << 20;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Current,
    Failed,
    Installed,
    InstallFailed(u32),
}

impl Outcome {
    pub fn encode(self) -> usize {
        match self {
            Outcome::Current => 0,
            Outcome::Failed => 1,
            Outcome::Installed => 2,
            Outcome::InstallFailed(c) => 3 | (c as usize) << 8,
        }
    }

    pub fn decode(v: usize) -> Outcome {
        match v & 0xFF {
            0 => Outcome::Current,
            2 => Outcome::Installed,
            3 => Outcome::InstallFailed((v >> 8) as u32),
            _ => Outcome::Failed,
        }
    }
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
        Ok(Some(msi)) => {
            let r = install(&msi);
            let _ = fs::remove_file(&msi);
            match r {
                Ok(0 | 1641 | 3010) => Outcome::Installed,
                Ok(code) => Outcome::InstallFailed(code),
                Err(e) => {
                    crate::report::log(&format!("msiexec: {e}"));
                    Outcome::InstallFailed(0)
                }
            }
        }
        Err(e) => {
            crate::report::log(&format!("update check: {e}"));
            Outcome::Failed
        }
    }
}

fn get(url: &str, limit: u64) -> Res<Vec<u8>> {
    let prefix = format!("https://github.com/{REPO}/releases/download/");
    if !url.starts_with(&prefix) && !url.starts_with("https://api.github.com/") {
        return Err("untrusted url".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(30))).build().into();
    let mut r = agent
        .get(url)
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
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let path =
        env::temp_dir().join(format!("rust-desktop-icons-{}.{}.{}-{}-{nonce}.msi", latest[0], latest[1], latest[2], std::process::id()));
    fs::OpenOptions::new().write(true).create_new(true).open(&path)?.write_all(&msi)?;
    Ok(Some(path))
}

fn verify(data: &[u8], sum: &str) -> bool {
    let hex: String = Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect();
    sum.split_whitespace().next().is_some_and(|s| s.eq_ignore_ascii_case(&hex))
}

fn install(msi: &PathBuf) -> Res<u32> {
    let sys = env::var_os("SystemRoot").map(PathBuf::from).ok_or("no SystemRoot")?;
    let status = Command::new(sys.join("System32").join("msiexec.exe"))
        .arg("/i")
        .arg(msi)
        .args(["/qn", "/norestart"])
        .creation_flags(0x0800_0000)
        .status()?;
    Ok(status.code().unwrap_or(-1) as u32)
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
    fn outcome_roundtrip() {
        for o in [Outcome::Current, Outcome::Failed, Outcome::Installed, Outcome::InstallFailed(1618)] {
            assert_eq!(Outcome::decode(o.encode()), o);
        }
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
