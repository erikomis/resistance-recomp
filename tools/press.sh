#!/bin/zsh
# Press a button in a start.sh run, wait, then snap the frame.
#
#   tools/press.sh <name> <tag> <mask> [polls] [wait-seconds]
#
# Writes "<mask> <polls>" to PAD_FILE ($RES_OUT/pad): the pad reads it on its
# next poll, holds the buttons for <polls> polls (default 40) and empties the
# file. After [wait-seconds] (default 6) the frame is saved as <tag>.png.
# Masks: START 0x0008, UP 0x0010, RIGHT 0x0020, DOWN 0x0040, LEFT 0x0080,
# TRIANGLE 0x1000, CIRCLE 0x2000, CROSS 0x4000 (OR them for chords).
here=${0:A:h}; source "$here/env.sh"
name=${1:?usage: press.sh <name> <tag> <mask> [polls] [wait]}; tag=${2:?tag}; mask=${3:?mask}
echo "$mask ${4:-40}" > "$RES_OUT/pad"
sleep ${5:-6}
"$here/snap.sh" "$name" "$tag" >/dev/null
echo "$tag done"
