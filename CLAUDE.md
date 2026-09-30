# Rust Desktop Icons

Open-source Fences 6 alternative for Windows 10/11: customizable desktop rectangles ("fences") that hold icons.

## Stack
Rust 2024 (1.98), `windows` 0.62 (raw Win32/GDI), serde/serde_json, ureq 3 (rustls), sha2. MSI via WiX 5.0.2.

## Commands (verified)
- Test: `cargo test` — single: `cargo test grid_layout`
- Lint / format: `cargo clippy --all-targets -- -D warnings` / `cargo fmt`
- Build: `cargo build --release`
- MSI (local): WiX is installed per-user at `%LOCALAPPDATA%\wixtool\wix.exe`, needs `DOTNET_ROOT=%LOCALAPPDATA%\dotnet`:
  `wix build installer/main.wxs -d Version=0.1.0 -arch x64 -o target/rdi.msi`
- Release: bump `version` in Cargo.toml, commit, push tag `vX.Y.Z` → `.github/workflows/release.yml` builds and publishes MSI + `.sha256`.

## Layout
- `src/domain.rs` pure logic (all unit-tested); `store.rs` persistence; `i18n.rs` 25 languages; `update.rs` updater.
- `src/app.rs` state + tray; `src/fence.rs` fence window; `src/shell.rs`, `src/win.rs` Win32 wrappers.
- `installer/main.wxs` per-user MSI (installs to `%LOCALAPPDATA%\Programs`, owns the HKCU Run value).

## Conventions
- User wants terse code: no comments, compact formatting (`rustfmt.toml` max_width 140).
- Business rules go in `domain.rs` with tests; Win32 code stays thin.
- Global state is `thread_local RefCell` accessed only via `app::with` (try_borrow). Never call Win32 APIs that
  send messages (DestroyWindow, SetWindowPos, TrackPopupMenu, MessageBox, CreateWindow) inside `with`.
- New UI string: add a `T` variant and translate it in all 25 rows of `i18n.rs` (tests check completeness).

## Gotchas
- Debug builds use a separate single-instance mutex, so they run beside the installed app. For visual checks, run the debug exe
  with `LOCALAPPDATA` pointed at a scratch dir (never touch the user's real data) and `RDI_DUMP=<file>` to dump the canvas.
- Background blur was tried and rejected (v0.3): DWM system backdrops go solid on inactive windows, and the
  undocumented SetWindowCompositionAttribute accent rendered opaque black with GDI content. Keep the layered path.
- Debug `RDI_TOP=1` makes fences topmost and unowned so screenshots can capture them.
- Fences are `UpdateLayeredWindow` windows: no WM_PAINT, call `fence::render`; child controls don't show (rename edit is a popup).
- Fence windows are owned by `Progman` and forced to `HWND_BOTTOM`; Explorer restart → `TaskbarCreated` → `rebuild()`.
- The updater only accepts URLs under this repo's releases; the MSI asset must have a matching `<name>.msi.sha256`.
- MSI `UpgradeCode` and component GUIDs must never change.
