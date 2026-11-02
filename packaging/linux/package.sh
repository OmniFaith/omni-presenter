#!/usr/bin/env bash
set -euo pipefail

tag=$1
target=$2
case "$target" in
  x86_64-unknown-linux-gnu) arch=x86_64 ;;
  aarch64-unknown-linux-gnu) arch=aarch64 ;;
  *) printf 'Unsupported Linux target: %s\n' "$target" >&2; exit 1 ;;
esac

root=$(pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
binary="$root/target/$target/release/omni-presenter"
name="omni-presenter-$tag-$target"
mkdir -p "$root/dist" "$work/flatpak" "$work/AppDir/usr/share/doc/omni-presenter"
install -m755 "$binary" "$work/flatpak/omni-presenter"
cp packaging/linux/org.omnipresenter.OmniPresenter.{yml,desktop,svg} packaging/linux/omni-presenter-launch "$work/flatpak/"
chmod +x "$work/flatpak/omni-presenter-launch"
cp LICENSE "$work/flatpak/LICENSE"
cp assets/bible-source/LICENSE "$work/flatpak/bible-LICENSE"
cp assets/fonts/OFL.txt "$work/flatpak/NotoEmoji-OFL.txt"
cp "$work/flatpak/"*LICENSE "$work/flatpak/NotoEmoji-OFL.txt" "$work/AppDir/usr/share/doc/omni-presenter/"

curl --fail --location --retry 3 "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$arch.AppImage" -o "$work/linuxdeploy.AppImage"
chmod +x "$work/linuxdeploy.AppImage"
export APPIMAGE_EXTRACT_AND_RUN=1
export LDAI_OUTPUT="$root/dist/$name.AppImage"
rsvg-convert -w 256 -h 256 "$work/flatpak/org.omnipresenter.OmniPresenter.svg" -o "$work/org.omnipresenter.OmniPresenter.png"
"$work/linuxdeploy.AppImage" --appdir "$work/AppDir" --executable "$binary" \
  --desktop-file "$work/flatpak/org.omnipresenter.OmniPresenter.desktop" \
  --icon-file "$work/org.omnipresenter.OmniPresenter.png" --output appimage

flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak-builder --user --install-deps-from=flathub --force-clean --state-dir="$work/state" --repo="$work/repo" \
  "$work/build" "$work/flatpak/org.omnipresenter.OmniPresenter.yml"
flatpak build-bundle --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo \
  "$work/repo" "$root/dist/$name.flatpak" org.omnipresenter.OmniPresenter
