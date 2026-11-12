#!/bin/zsh
# The release zip RecompLauncher installs from (launcher/README.md,
# "RecompLauncher"): the launcher's .app and recomplauncher.json side by side
# at the top, named as its standards ask for a Mac build:
#
#   launcher/scripts/package-recomplauncher.sh [--app <Resistance Launcher.app>] [--out <dir>]
#
#   --app  an .app built already (e.g. the CI's); without it the app is
#          built here (npx tauri build --bundles app, as tools/rebuild.sh
#          launcher does)
#   --out  where the zip goes (default: out/recomplauncher in the repository)
#
# Writes resistance-recomp-<version>-macos-arm64.zip, <version> being the
# launcher's (src-tauri/tauri.conf.json). The app is checked first: its
# version, the program recomplauncher.json starts, and nothing from the game
# in it (scripts/check_bundle.py); the zip is checked again once written.
# It only makes the file: uploading it to a release is a separate step.
set -e
setopt pipefail
here=${0:A:h}
launcher=${here:h}
root=${launcher:h}
fail() { print -u2 -r -- "ERROR: $*"; exit 1; }

app= out=$root/out/recomplauncher
while (( $# )); do
    case $1 in
    --app) app=${2:A}; shift 2 ;;
    --out) out=${2:A}; shift 2 ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) fail "unknown argument $1 (--app <.app>, --out <dir>)" ;;
    esac
done
[ "$(uname -s)" = Darwin ] || fail "macOS only (ditto, the .app)"

profile=$root/recomplauncher.json
version=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$launcher/src-tauri/tauri.conf.json")
# the program the profile starts and prepares with, inside the zip
program=$(python3 -c '
import json, sys
p = json.load(open(sys.argv[1]))
progs = {p["start"]["program"][0], p["prepare"]["program"][0]}
assert len(progs) == 1, progs
print(progs.pop())' "$profile") || fail "$profile: start and prepare must name one program"

if [ -z "$app" ]; then
    print -r -- "==> building the launcher $version (npx tauri build --bundles app)"
    ( cd "$launcher" && npm ci &&
      CARGO_PROFILE_RELEASE_STRIP=${CARGO_PROFILE_RELEASE_STRIP:-false} \
      CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP=${CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP:-false} \
      npx tauri build --bundles app )
    app=${CARGO_TARGET_DIR:-$launcher/src-tauri/target}/release/bundle/macos/"Resistance Launcher.app"
fi
[ -d "$app" ] || fail "$app: no such .app"
name=${app:t}
[ "$name/${program#*/}" = "$program" ] || fail "recomplauncher.json starts $program, but the app is $name"
[ -x "$app/${program#*/}" ] || fail "$app has no ${program#*/}"
appver=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app/Contents/Info.plist")
[ "$appver" = "$version" ] || fail "$name is version $appver, the launcher's sources say $version"
python3 "$here/check_bundle.py" "$app"

zip=resistance-recomp-$version-macos-arm64.zip
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
print -r -- "==> $zip"
ditto "$app" "$stage/top/$name"
cp "$profile" "$stage/top/recomplauncher.json"
mkdir -p "$out"
rm -f "$out/$zip"
# no ._ files or extended attributes (quarantine): the bundle and the profile only
ditto -c -k --norsrc --noextattr --noqtn "$stage/top" "$out/$zip"

# the zip as RecompLauncher sees it: the .app and the profile on top
mkdir "$stage/check"
ditto -x -k "$out/$zip" "$stage/check"
top=$(ls -A "$stage/check" | sort | tr '\n' '|')
[ "$top" = "$name|recomplauncher.json|" ] || fail "the zip's top level is $top"
[ -x "$stage/check/$program" ] || fail "the zip lost the program's executable bit"
python3 "$here/check_bundle.py" "$stage/check"
print -r -- "$out/$zip ($(( $(stat -f %z "$out/$zip") / 1024 / 1024 )) MB)"
