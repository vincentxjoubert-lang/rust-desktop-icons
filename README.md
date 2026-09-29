<p align="center"><img src="assets/icon.png" width="96" alt="Rust Desktop Icons logo"></p>

<h1 align="center">Rust Desktop Icons</h1>
<p align="center"><b>An open-source alternative to Fences 6, written entirely in Rust.</b><br>
Group your desktop icons into clean, customizable rectangles, with a tiny memory footprint.</p>

<p align="center">
  <a href="https://github.com/vincentxjoubert-lang/rust-desktop-icons/releases/latest"><img src="https://img.shields.io/github/v/release/vincentxjoubert-lang/rust-desktop-icons" alt="Latest release"></a>
  <a href="https://github.com/vincentxjoubert-lang/rust-desktop-icons/actions/workflows/ci.yml"><img src="https://github.com/vincentxjoubert-lang/rust-desktop-icons/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6" alt="Windows 10 | 11">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="MIT license"></a>
</p>

## Features

- **Fences**: semi-transparent rectangles that live on your desktop, behind every window.
- **Drag & drop** icons from the desktop into a fence; double-click to open them.
- **Customizable**: move, resize, rename, color, opacity (30–100 %), roll up to the title bar.
- **25 languages**, auto-detected from Windows, switchable from the tray menu.
- **Silent automatic updates** from GitHub Releases, verified with SHA-256.
- **Lightweight**: native Win32, no runtime, no web view. About 4.5 MB of private memory in use.
- **Per-user install**: no administrator rights needed, starts with Windows (optional).

## Install

1. Download `rust-desktop-icons-x.y.z-x64.msi` from the [latest release](https://github.com/vincentxjoubert-lang/rust-desktop-icons/releases/latest).
2. Run it. The app starts immediately and shows an icon in the notification area.

> The installer is not code-signed yet, so Windows SmartScreen may warn you: click **More info → Run anyway**.
> You can check the file against the `.sha256` published next to it: `Get-FileHash .\rust-desktop-icons-*.msi`.

## Usage

| Action | How |
| --- | --- |
| Create a fence | Tray icon → **New fence**, or right-click an existing fence |
| Add icons | Drag them from the desktop onto a fence |
| Open an item | Double-click it |
| Move / resize | Drag the title bar / drag an edge |
| Roll up | Double-click the title bar |
| Rename, color, opacity, delete | Right-click the fence |
| Put an item back | Right-click it → **Move to desktop** |

Items dropped into a fence are **moved** into `%LOCALAPPDATA%\RustDesktopIcons\fences\<id>`, not copied or deleted.
Deleting a fence moves its items back to the desktop. Settings live in `%LOCALAPPDATA%\RustDesktopIcons\config.json`.

## Languages

English, 中文, हिन्दी, Español, Français, العربية, বাংলা, Português, Русский, اردو, Bahasa Indonesia, Deutsch, 日本語,
Kiswahili, मराठी, తెలుగు, Türkçe, தமிழ், Tiếng Việt, 한국어, Italiano, ไทย, Polski, Українська, فارسی.

Corrections from native speakers are welcome: every string lives in [`src/i18n.rs`](src/i18n.rs).

## Updates and security

- Every 6 hours (first check 30 s after start), the app asks the GitHub API for the latest release.
- If it is newer, it downloads the `.msi` and its `.sha256` **only** from this repository's release URLs over HTTPS,
  checks the hash, then runs `msiexec /qn`. The installer restarts the app.
- Turn this off from the tray menu (**Automatic updates**). No other network access, no telemetry.

## Build from source

```powershell
cargo test
cargo build --release
dotnet tool install --global wix --version 5.0.2
wix build installer/main.wxs -d Version=0.1.0 -arch x64 -o rust-desktop-icons.msi
```

Releases are built by [GitHub Actions](.github/workflows/release.yml) when a `vX.Y.Z` tag is pushed.

## Architecture

| Module | Role |
| --- | --- |
| `domain.rs` | Pure model and rules: fences, config validation, layout grid, hit zones, colors |
| `store.rs` | Config persistence (atomic write) and safe file moves |
| `i18n.rs` | 25-language string table and locale resolution |
| `update.rs` | Release check, download, SHA-256 verification, silent install |
| `app.rs` | Application state, tray icon and menu, update scheduling |
| `fence.rs` | Fence window: painting, hit-testing, drag & drop, context menu |
| `shell.rs`, `win.rs` | Thin wrappers over Windows Shell, registry and Win32 helpers |

## Limitations (v0.1)

- Only items from your own desktop (or another fence) can be dropped in; other locations are ignored.
- Dragging items out of a fence uses the context menu (**Move to desktop**), not drag & drop.
- Opacity applies to the whole fence, text included.
- **Before uninstalling**, delete your fences so their items go back to the desktop. Uninstalling never deletes your
  files: anything left stays in `%LOCALAPPDATA%\RustDesktopIcons\fences`.
- The fences follow the desktop layer; after an Explorer restart they are recreated automatically.

## License

[MIT](LICENSE). Not affiliated with Stardock; "Fences" is a trademark of its respective owner.
