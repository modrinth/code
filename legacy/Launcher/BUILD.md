# Building Owyx (Windows + Linux)

One codebase. **Windows** ships `Owyx.exe` (WebView2). **Linux** cloud / desktop
dev uses a native WebKitGTK window and a debug ELF. Data:

| Host | Player data |
|------|-------------|
| Windows | `%USERPROFILE%/owyx/` |
| Linux | `~/owyx/` |

**Default for agents / local verify = smoke (debug), not release.**  
Full Windows release is only for shipping / hand-off of the final exe.

Work from:

```bash
cd Launcher/apps/launcher
```

On Windows PowerShell the same directory works:

```powershell
cd Launcher/apps/launcher
```

---

## Build gates (fast → slow)

| Gate | Command | Typical time | What it proves |
|------|---------|--------------|----------------|
| **1. Compile check** | `npm run check` then `npm run check:rust` | seconds | TS + Rust typecheck / compile without linking a full app binary |
| **2. Linux smoke (debug ELF)** | `npm run build:smoke` or `npm run build:smoke:linux` | ~minutes | Runnable debug `src-tauri/target/debug/owyx` (ELF, not `.exe`) |
| **3. Windows smoke (debug)** | `npm run build:smoke` on a Windows host | ~minutes | Runnable debug `owyx.exe` (no release LTO / opt) |
| **4. Ship (Windows release)** | `npm run build:release` | much longer | Optimized ship `Owyx.exe` |

Prefer gate 1 while iterating. Use gate 2 on Cloud Linux (native WebKit) or gate 3
on Windows to confirm the native app builds and the window opens. Use gate 4 only
when handing off `Launcher/Owyx.exe`.

Linux **release** `.deb` / AppImage is **not** a current gate — see `PLAN.md` backlog.

Catalog zip overlay cap is **512 MB** (`MAX_PACK_BYTES` in `catalog.rs`). Do not raise
it without a streaming download. Foreg-sized (~2 GB) packs stay in backlog.

---

## Linux native dev & smoke (recommended for Cloud Agent)

Cloud Desktop VMs (Ubuntu 22.04 / 24.04) should develop against a **real Tauri
window**, not Vite-in-browser and not Wine + `Owyx.exe`.

### One-time system deps

```bash
# repo root
bash Launcher/scripts/linux-dev-setup.sh
```

Installs WebKitGTK 4.1, GTK 3, Ayatana (or `libappindicator3-dev` fallback),
librsvg, libsoup 3, build-essential / pkg-config, and ensures **Rust ≥ 1.85**.
Idempotent. Does **not** install Wine or MinGW.

Headless CI that only compiles can set `OWYX_LINUX_DEV_GUI=0` to skip optional
`libxdo-dev`.

### npm + compile gate

```bash
cd Launcher/apps/launcher
npm ci
npm run check && npm run check:rust
```

### Dev: native window (`dev:tauri`)

Needs a display. Cloud Desktop is usually `DISPLAY=:1` or `:0`.

```bash
export DISPLAY="${DISPLAY:-:1}"
npm run dev:tauri
```

This is `tauri dev`: Vite on `:1420` **plus** a WebKitGTK window. Rust `invoke`
(catalog download, install, skin-to-disk, login) is live. Works the same on
Windows (`dev:tauri` → WebView2).

**Smoke without Minecraft:** login UI, catalog fetch (if site API is on
`127.0.0.1:3001`), download progress UI, overlay error paths, navigation.
Smoke means: **window opens, UI is not blank, invoke does not crash, logs write
to `~/owyx/logs/`**.

Pre-release QA (2026-08-30) additionally ran **Play + 2 Modrinth mods** on Cloud
Linux (`dev:tauri`, Fabric 1.20.1). Windows `build:release` stays an owner
hand-off — do not run it on Cloud.

Optional catalog against a local site:

```bash
OWYX_API_BASE_URL=http://127.0.0.1:3001 npm run dev:tauri
```

If the API is down, Servers empty/offline is fine — do not treat that as a
launcher crash.

### Smoke: debug ELF

```bash
npm run build:smoke
# alias: npm run build:smoke:linux
```

Output (Cargo package name `owyx`):

```text
Launcher/apps/launcher/src-tauri/target/debug/owyx
```

Confirm it is an ELF, then run:

```bash
file src-tauri/target/debug/owyx
# ELF 64-bit LSB pie executable, …
./src-tauri/target/debug/owyx
```

### Optional CI job

`.github/workflows/launcher-check.yml` job **`linux-smoke`** runs
`npm run build:smoke` only on **`workflow_dispatch`** (not every push/PR), so
the fast `check` job stays cheap. Actions → launcher-check → Run workflow.

---

## 1. Compile check (seconds)

```bash
npm run check          # tsc --noEmit
npm run check:rust     # cargo check in src-tauri
```

PowerShell: the same `npm` scripts.

---

## 2. Windows smoke / debug build

Standard Tauri 2 fast path on a **Windows** host:

```powershell
npm run build:smoke
# equivalent: npm run tauri build -- --debug
```

Output binary (Cargo package name `owyx`; on Windows same file as `Owyx.exe`):

```text
Launcher/apps/launcher/src-tauri/target/debug/owyx.exe
```

Optional copy for a stable hand-off path (same idea as release smoke copy):

```powershell
Copy-Item -Force src-tauri\target\debug\owyx.exe ..\..\Owyx.debug.exe
```

→ `Launcher/Owyx.debug.exe`

### Alternative (frontend already built)

If `dist/` is fresh (`npm run build`), you can skip the Tauri CLI wrapper and only rebuild Rust:

```powershell
npm run build
cd src-tauri
cargo build
# -> target/debug/owyx.exe
```

`tauri build --debug` is preferred when you want the usual before-build frontend step in one command.

---

## 3. Release / ship (Windows hand-off only)

```powershell
npm run build:release
# equivalent: npm run tauri build
```

Output:

```text
Launcher/apps/launcher/src-tauri/target/release/Owyx.exe
```

### Ship copy (repo convention)

After a successful **release** build, copy to:

```text
Launcher/Owyx.exe
```

```powershell
Copy-Item -Force src-tauri\target\release\Owyx.exe ..\..\Owyx.exe
```

This path is the documented hand-off location for friends and local QA (also referenced in `PLAN.md` and `RELEASE_NOTES_v0.1.0.md`). Do **not** treat a debug binary as `Launcher/Owyx.exe`.

Linux `.deb` / AppImage bundling is **not** enabled (`bundle.targets` stays empty).

---

## Fast UI without native invoke (secondary)

Vite-only preview is still useful for CSS/i18n:

```bash
OWYX_API_BASE_URL=http://127.0.0.1:3001 npm run dev
# open http://127.0.0.1:1420 — Download / Play / skin-to-disk are browser stubs
```

Prefer `npm run dev:tauri` when you need real Rust commands.

---

## Legacy: cross-build Windows exe on Linux (UI blocked)

**Not** the Cloud Agent path. Native `dev:tauri` / Linux smoke ELF above is the
supported Linux contour. This appendix stays so we do not re-spend time on
Wine + WebView2.

Verified historically on a Cursor cloud Linux Desktop VM (Ubuntu 24.04, Wine 9.0,
Rust 1.97, mingw-w64 GCC 13): the **native backend** can boot under Wine
(install/meta/logging/window chrome). **The WebView2 UI does not render**
(Chromium/Edge under Wine — see “Known blocker”). Treat as **legacy /
unverified / Windows QA only**.

### 1. Toolchain (one-time)

```bash
# Rust Windows GNU target + MinGW cross-linker
rustup target add x86_64-pc-windows-gnu
sudo apt-get install -y --no-install-recommends mingw-w64

# Wine (64-bit) + 32-bit support
sudo dpkg --add-architecture i386
sudo apt-get update
sudo apt-get install -y --no-install-recommends wine64 wine32 winbind
# If /usr/bin/wine is missing, the binary is /usr/lib/wine/wine64:
sudo ln -sf /usr/lib/wine/wine64 /usr/local/bin/wine64

# Init a 64-bit prefix (data will live under it, see §5)
WINEPREFIX=$HOME/.wine WINEARCH=win64 wine64 wineboot -u
```

### 2. Cross-compile the exe

```bash
cd Launcher/apps/launcher
npm run build                     # produce dist/ (frontend)
cd src-tauri
cargo build --target x86_64-pc-windows-gnu --bin owyx
# -> target/x86_64-pc-windows-gnu/debug/owyx.exe  (PE32+ x86-64)
```

**Caveat — `export ordinal too large`:** a plain `cargo build` also links the mobile-only `cdylib` (`owyx_lib.dll`) and MinGW `ld` fails on it (`export ordinal too large`) because the whole `windows` crate is re-exported. The desktop `.exe` does **not** need that artifact. Two ways around it:

- Build only the binary: `cargo build --target x86_64-pc-windows-gnu --bin owyx` (still builds the cdylib in some cargo versions), **or**
- Temporarily set the lib `crate-type = ["rlib"]` in `Cargo.toml` for the cross-build, then revert. (Do **not** commit that change — `staticlib`/`cdylib` are needed for mobile.)

### 3. Ship the WebView2 loader shim next to the exe

The exe statically imports `WebView2Loader.dll` (produced by `webview2-com-sys`). Copy it beside the exe or Wine reports `WebView2Loader.dll ... not found (c0000135)`:

```bash
cp target/x86_64-pc-windows-gnu/debug/WebView2Loader.dll <run-dir>/
```

### 4. Run under Wine

```bash
export DISPLAY=:1            # the Desktop X display
export WINEPREFIX=$HOME/.wine
wine64 <run-dir>/Owyx.exe
```

Backend boots cleanly (launcher log line `=== Owyx launcher start ===`), the data layout is created, and a native window opens.

### 5. Where player data lives under Wine

Our code uses `%USERPROFILE%/owyx`. Under Wine that resolves to:

```text
~/.wine/drive_c/users/<user>/owyx/
  ├─ meta/{versions,libraries,assets,natives,java_versions}
  ├─ instances/
  ├─ cache/
  └─ logs/owyx.log
```

(Confirmed created on first run — the Theseus-style `meta/` layout is identical to Windows.)

### 6. Known blocker — WebView2 UI does not render under Wine

Tauri on Windows renders its UI with **WebView2 (Edge/Chromium)**. Under Wine the native window opens but stays blank: `CreateCoreWebView2Environment` never spawns `msedgewebview2.exe`. Attempts made (all still blank):

1. Ship `WebView2Loader.dll` → exe loads, backend runs, blank window (no runtime).
2. Install the Evergreen runtime: `wine64 MicrosoftEdgeWebview2Setup.exe /silent /install` → the installer crashes (`Unhandled exception 0x80000003`) but lays down an Edge runtime under `EdgeCore/…`.
3. Force the fixed-version runtime + disable the Chromium sandbox (Wine can’t provide it):
   ```bash
   export WEBVIEW2_BROWSER_EXECUTABLE_FOLDER='C:\Program Files (x86)\Microsoft\EdgeCore\<ver>'
   export WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--no-sandbox --disable-gpu --single-process --in-process-gpu'
   ```
   → `msedgewebview2.exe` still does not start (Chromium/Edge does not run under Wine 9.0).

**Recommendation:**
- **Linux Cloud Agent:** `bash Launcher/scripts/linux-dev-setup.sh` then `npm run dev:tauri` (native WebKit). Do not retry Wine WebView2.
- **Windows Play / ship exe:** Windows host, MiniMax QA, or release CI — not this Linux contour.

### Proton

Not attempted on this VM (no Steam / Proton / `protontricks` installed). If a Proton prefix is available, the same three env vars from §6 apply; document exact commands when a working Proton is present. Proton bundles a newer Wine + DXVK, so WebView2 *may* fare better, but it is unverified.
