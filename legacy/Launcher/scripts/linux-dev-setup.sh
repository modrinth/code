#!/usr/bin/env bash
# Install Ubuntu 22.04 / 24.04 packages for Tauri 2 Linux (WebKitGTK 4.1)
# and ensure rustc is new enough for this workspace (edition2024 crates).
#
# Usage (from repo root):
#   bash Launcher/scripts/linux-dev-setup.sh
#
# Idempotent. Does not install Wine, MinGW, or a Windows cross-toolchain.

set -euo pipefail

need_sudo() {
  if [[ "$(id -u)" -eq 0 ]]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    sudo "$@"
  else
    echo "error: need root or sudo to install apt packages" >&2
    exit 1
  fi
}

apt_available() {
  apt-cache show "$1" >/dev/null 2>&1
}

echo "==> Owyx Linux Tauri 2 dev setup"

if [[ ! -f /etc/os-release ]]; then
  echo "error: /etc/os-release missing; this script targets Ubuntu 22.04 / 24.04" >&2
  exit 1
fi

# shellcheck disable=SC1091
. /etc/os-release
echo "    distro: ${PRETTY_NAME:-unknown}"

if [[ "${ID:-}" != "ubuntu" && "${ID_LIKE:-}" != *ubuntu* && "${ID_LIKE:-}" != *debian* ]]; then
  echo "warning: not Ubuntu/Debian; apt packages may have different names" >&2
fi

need_sudo apt-get update -y

CORE_PACKAGES=(
  build-essential
  pkg-config
  curl
  wget
  file
  libwebkit2gtk-4.1-dev
  libgtk-3-dev
  librsvg2-dev
  libsoup-3.0-dev
)

# Tray / app-indicator: prefer Ayatana (Ubuntu 22.04+). Older Debian may only
# ship libappindicator3-dev — try that if Ayatana is not in the cache.
INDICATOR_PKG=""
if apt_available libayatana-appindicator3-dev; then
  INDICATOR_PKG=libayatana-appindicator3-dev
elif apt_available libappindicator3-dev; then
  INDICATOR_PKG=libappindicator3-dev
  echo "    using fallback ${INDICATOR_PKG} (libayatana-appindicator3-dev not available)"
else
  echo "warning: no app-indicator package found; tray may fail to link" >&2
fi

# Optional: xdotool bindings used by some Tauri desktop helpers. Skip on
# headless CI images that do not need a window manager.
OPTIONAL_PACKAGES=()
if [[ "${OWYX_LINUX_DEV_GUI:-1}" != "0" ]] && apt_available libxdo-dev; then
  OPTIONAL_PACKAGES+=(libxdo-dev)
fi

INSTALL_PACKAGES=("${CORE_PACKAGES[@]}")
if [[ -n "${INDICATOR_PKG}" ]]; then
  INSTALL_PACKAGES+=("${INDICATOR_PKG}")
fi
INSTALL_PACKAGES+=("${OPTIONAL_PACKAGES[@]}")

echo "==> apt install: ${INSTALL_PACKAGES[*]}"
need_sudo DEBIAN_FRONTEND=noninteractive apt-get install -y "${INSTALL_PACKAGES[@]}"

ensure_rust() {
  local ver_line major minor
  if ! command -v rustc >/dev/null 2>&1; then
    echo "==> rustc not found; installing rustup stable"
    if [[ ! -x "${HOME}/.cargo/bin/rustup" ]]; then
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    fi
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
  fi

  ver_line="$(rustc --version)"
  echo "    rustc: ${ver_line}"
  major="$(echo "${ver_line}" | sed -n 's/^rustc \([0-9]\+\)\.\([0-9]\+\).*/\1/p')"
  minor="$(echo "${ver_line}" | sed -n 's/^rustc \([0-9]\+\)\.\([0-9]\+\).*/\2/p')"
  if [[ -z "${major}" || -z "${minor}" ]]; then
    echo "error: could not parse rustc --version" >&2
    exit 1
  fi
  if (( major < 1 || (major == 1 && minor < 85) )); then
    echo "==> rustc ${major}.${minor} < 1.85; updating stable"
    if command -v rustup >/dev/null 2>&1; then
      rustup update stable
      rustup default stable
    else
      echo "error: rustc is too old and rustup is not installed" >&2
      exit 1
    fi
  fi
}

ensure_rust

echo
echo "==> success checklist"
echo "    rustc:           $(rustc --version)"
echo "    cargo:           $(cargo --version)"
if command -v pkg-config >/dev/null 2>&1 && pkg-config --exists webkit2gtk-4.1; then
  echo "    webkit2gtk-4.1:  $(pkg-config --modversion webkit2gtk-4.1)"
else
  echo "    webkit2gtk-4.1:  MISSING (pkg-config webkit2gtk-4.1)" >&2
  exit 1
fi
if pkg-config --exists gtk+-3.0; then
  echo "    gtk+-3.0:        $(pkg-config --modversion gtk+-3.0)"
else
  echo "    gtk+-3.0:        MISSING" >&2
  exit 1
fi
echo "    DISPLAY:         ${DISPLAY:-unset (needed for npm run dev:tauri)}"
echo
echo "Next:"
echo "  cd Launcher/apps/launcher && npm ci"
echo "  npm run check && npm run check:rust"
echo "  npm run dev:tauri          # native WebKit window"
echo "  npm run build:smoke        # debug ELF → src-tauri/target/debug/owyx"
echo
