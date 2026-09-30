# Rust Desktop Icons

Open-source Fences 6 alternative for Windows 10/11: customizable desktop rectangles ("fences") that hold icons.

## Primary rule (non-negotiable)
- **No god files**: one responsibility per module. When a file mixes concerns or grows past ~250 lines (data tables like
  `i18n.rs` excepted), split it into a folder module (`foo/mod.rs` + focused files) before adding more.
- **Strict DRY**: any logic written twice is factored (helper fn, shared const, `App`/`win` method). Check existing
  helpers (`win.rs`, `App::fence_of`, `app::change`, `fence::update`) before writing new code.

## Stack
Rust 2024 (1.98), `windows` 0.62 (raw Win32/GDI), serde/serde_json, ureq 3 (rustls), sha2. MSI via WiX 5.0.2.

## Commands (verified)
- Test: `cargo test` — single: `cargo test grid_layout`
- Lint / format: `cargo clippy --all-targets -- -D warnings` / `cargo fmt`
- Build: `cargo build --release`
- MSI (local): needs WiX 5.0.2 (`dotnet tool install wix --version 5.0.2`) + `wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`:
  `wix build installer/main.wxs -d Version=0.1.0 -arch x64 -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext -o target/rdi.msi`
- Release: bump `version` in Cargo.toml, commit, push tag `vX.Y.Z` → `.github/workflows/release.yml` builds and publishes MSI + `.sha256`.

## Layout
- `src/domain/` pure logic, all unit-tested: `model.rs` (Config/Fence/Tab/Look), `kind.rs` (file types for rules),
  `order.rs` (per-tab sort + custom order), `anim.rs`, `grid.rs`, `snap.rs`, `icons.rs`, `color.rs`, `zone.rs`.
- `store.rs` persistence (`tab_dir`: portal path or `fences/<tab id>`); `i18n/` (`mod.rs` logic, `table.rs` 25 languages);
  `update.rs` updater (30 s network timeout, waits for msiexec; the MSI closes/relaunches the app);
  `rules.rs` desktop watcher + auto-sort (files younger than 10 s are retried later); `prefs.rs` global prefs;
  `report.rs` errors: `report::alert` (dialog + `errors.log`), `report::log`, panic hook -> `crash.log`;
  `uninstall.rs` (`--uninstall`, run by the MSI on real uninstall only: moves fence contents back to the desktop).
- `src/app/` global state: `mod.rs` App + `with` + startup, `view.rs` per-window state, `res.rs` fonts/glyphs/menu
  entries with icons, `fences.rs` create/rebuild fences; `src/tray/` tray icon/menu (`updates.rs` update scheduling).
- `src/layered/` shared per-pixel window painting (`Frame`: text mask, compositing, present; `Dib`; `glyph.rs`
  Segoe MDL2 icon glyphs, also rendered to menu bitmaps via `App::entry`/`App::sub`).
- `src/shell/` Win32 shell wrappers: `mod.rs` paths/icons/open, `reg.rs` registry, `link.rs`, `watch.rs`, `dnd.rs`
  (folder picker, `drag_out` = OLE drag via SHDoDragDrop; the app calls OleInitialize).
- `src/fence/` fence window: `mod.rs` proc, `layout.rs` geometry, `items.rs` icons, `paint.rs` + `header.rs` rendering,
  `tabs.rs` tabs/portals, `anim.rs` unroll + chameleon fade, `peek.rs` hover, `menu.rs`, `actions.rs`, `title.rs` rename,
  `drop.rs` (from outside / onto a tab header / reorder inside a tab), `select.rs` (click, Ctrl+click, rubber band,
  drag out), `arrange.rs` (per-tab sort and rules submenus), `snap.rs`.
- `src/settings/` settings window (custom drawn): `rows.rs` content/layout, `paint.rs`, `act.rs`, `mod.rs` window.
- `installer/main.wxs` per-user MSI (installs to `%LOCALAPPDATA%\Programs`, owns the HKCU Run value).

## Conventions
- User wants terse code: no comments, compact formatting (`rustfmt.toml` max_width 140).
- Business rules go in `domain.rs` with tests; Win32 code stays thin.
- Global state is `thread_local RefCell` accessed only via `app::with` (try_borrow). Never call Win32 APIs that
  send messages (DestroyWindow, SetWindowPos, TrackPopupMenu, MessageBox, CreateWindow) inside `with`.
- New UI string: add a `T` variant and translate it in all 25 rows of `i18n.rs` (tests check completeness).

## Robustness rules
- Never ignore a user-visible failure: collect errors and call `report::failures`/`report::alert` outside `app::with`.
- `store::save` keeps `config.json.bak`; an unreadable config is renamed `config.json.bad-<ts>` and never overwritten.
- Items keep a system image-list index (`Item.icon: i32`), never an HICON: icons are drawn on demand via
  `shell::draw_icon`. Folder reads + icon lookups run on a thread (`fence/loader.rs`, per-tab generations);
  each tab has its own change message (`WM_CHANGED + tab index`, `WM_LOADED` sits after that range).
- Present layered windows outside `app::with` (UpdateLayeredWindow re-enters the window proc).
- Roll/fade animation never re-renders per frame: `paint::draw(.., open=true)` once into `View.frame`, then
  `Frame::present_top` crops it; ticks use the QPC clock (`win::now_ms`; DWM vblank timing goes stale on an idle desktop, never use it), WM_SIZE skips render
  while animating (`View.sizing` marks self-caused resizes). `render` keeps the open frame as the cache when fully
  unrolled, and a `PREPARE` timer rebuilds it 150 ms after it is invalidated, so animations never start with a full draw. `Canvas::rrect` fast-paths straight rows and `blur` only processes rows with text (tests prove equality).

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
- The MSI closes the running app first (`util:CloseApplication` → WM_CLOSE, handled by the tray window; killed after 5 s),
  otherwise the old process keeps its mutex and the relaunched new version exits silently.
- The desktop right-click verb (`HKCU\Software\Classes\DesktopBackground\Shell\RustDesktopIcons`) is written by the
  release app at startup (localized, runs `exe --new`), removed by the MSI on uninstall. On Windows 11 it only shows under
  "Show more options" (the modern menu needs a packaged IExplorerCommand).
- MSI desktop shortcut: created on first install (checkbox, default on). On upgrades it is only recreated if it still
  exists on the desktop (`SHORTCUTEXISTS` FileSearch), so a shortcut the user deleted never comes back.
