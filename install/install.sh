#!/usr/bin/env bash
# Astral — install via terminal (Linux .deb / macOS .dmg).
# Downloads the latest GitHub release for this OS/arch and installs it.
#
#   bash <(curl -sSL https://raw.githubusercontent.com/nguyenthanhthe/astral/main/install/install.sh)
#
# Requires: curl, sudo (Linux), and on macOS the Xcode Command Line Tools.

set -euo pipefail

REPO="nguyenthanhthe/astral"
API="https://api.github.com/repos/${REPO}/releases/latest"
UA="astral-install-script"

say() { printf '\033[1;36mastral\033[0m %s\n' "$*"; }
die() { printf '\033[1;31mastral\033[0m ERROR: %s\n' "$*" >&2; exit 1; }

# ---- detect platform ---------------------------------------------------------
case "$(uname -s)" in
  Linux) OS=linux ;;
  Darwin) OS=macos ;;
  *) die "unsupported OS: $(uname -s)" ;;
esac
case "$(uname -m)" in
  x86_64 | amd64) ARCH=x86_64 ;;
  arm64 | aarch64) ARCH=aarch64 ;;
  *) die "unsupported architecture: $(uname -m)" ;;
esac
say "detected ${OS}/${ARCH}"

# ---- resolve the latest release tag -----------------------------------------
VERSION="$(
  curl -fsSL -H "User-Agent: ${UA}" "$API" |
    grep -o '"tag_name": *"[^"]*"' |
    head -1 |
    sed -E 's/.*"tag_name": *"v?//; s/"//'
)" || die "could not reach ${API}"
[ -n "${VERSION}" ] || die "could not resolve the latest release tag"
say "latest release: v${VERSION}"

# ---- pick the matching asset -------------------------------------------------
case "${OS}:${ARCH}" in
  linux:x86_64)
    ASSET="astral_${VERSION}_amd64.deb"
    PKG=deb
    ;;
  macos:x86_64 | macos:aarch64)
    ASSET="astral_${VERSION}_universal.dmg"
    PKG=dmg
    ;;
  linux:aarch64)
    die "no release build for linux/aarch64 yet — run from a Windows release or build from source"
    ;;
  *) die "no release build for ${OS}/${ARCH}" ;;
esac
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ASSET}"

# ---- download ----------------------------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
say "downloading ${ASSET}"
curl -fL --progress-bar -H "User-Agent: ${UA}" -o "${TMP}/${ASSET}" "${URL}" \
  || die "download failed: ${URL}"

# ---- install ------------------------------------------------------------------
case "${PKG}" in
  deb)
    say "installing with dpkg (sudo required)"
    sudo -v
    sudo dpkg -i "${TMP}/${ASSET}" ||
      { sudo apt-get -y -f install >/dev/null 2>&1 && sudo dpkg -i "${TMP}/${ASSET}"; }
    ;;
  dmg)
    say "mounting dmg and copying astral.app to /Applications"
    MOUNT="$(
      hdiutil attach -nobrowse -readonly "${TMP}/${ASSET}" |
        grep -o '/Volumes/.*' |
        head -1
    )"
    ditto "${MOUNT}/astral.app" /Applications/astral.app
    hdiutil detach "${MOUNT}" >/dev/null
    ;;
esac

say "done — launch Astral from your app launcher."
