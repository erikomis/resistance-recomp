#!/bin/zsh
# Save the latest frame of a start.sh run as $RES_OUT/<name>/<tag>.png.
#
#   tools/snap.sh <name> <tag>
here=${0:A:h}; source "$here/env.sh"
name=${1:?usage: snap.sh <name> <tag>}; tag=${2:?usage: snap.sh <name> <tag>}
d=$RES_OUT/$name
[ -f "$d/frame.ppm" ] || { echo "no frame yet in $d" >&2; exit 1; }
cp "$d/frame.ppm" "$d/$tag.ppm" && "$PY" "$here/ppm2png.py" "$d/$tag.ppm" "$d/$tag.png" >/dev/null \
    && rm -f "$d/$tag.ppm" && echo "$d/$tag.png"
