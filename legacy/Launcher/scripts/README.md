# Launcher scripts

## `linux-dev-setup.sh`

One-time (idempotent) Ubuntu 22.04 / 24.04 setup for **native** Tauri 2 Linux:

- WebKitGTK 4.1, GTK 3, Ayatana app-indicator (or `libappindicator3-dev` fallback), librsvg, libsoup 3
- Build tools (`build-essential`, `pkg-config`, `curl`, `wget`, `file`)
- Rust ≥ 1.85 via `rustup` if `rustc` is missing or too old

Does **not** install Wine or MinGW. Headless CI can skip optional GUI extras with `OWYX_LINUX_DEV_GUI=0`.

```bash
# from the monorepo root
bash Launcher/scripts/linux-dev-setup.sh
cd Launcher/apps/launcher && npm ci
npm run check && npm run check:rust
npm run dev:tauri          # native WebKit window (needs DISPLAY)
npm run build:smoke        # debug ELF → src-tauri/target/debug/owyx
```
