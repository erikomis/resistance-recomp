#!/bin/zsh
# Headless (or windowed) timed run that keeps a frame every 10 s.
#
#   tools/run.sh <name> <seconds> [eboot|game]
#
# Writes $RES_OUT/<name>.log and $RES_OUT/<name>/tNNN.png. The game runs in
# $RES_OUT/<name>.cwd/ with dev_hdd0/ and dev_hdd1/ inside it (the install
# cache, PROFILE.SAV, quick.sav), cleared before each run unless KEEP=1 or
# CWD=<dir> names a directory to reuse (CWD=$RES_OUT/game.cwd shares the one
# start.sh uses, so the first-run install is not redone).
#
# Environment passes through to the port: PAD_SCRIPT="<sec>:<mask>,..." for a
# timed press (masks below), PS3RECOMP_METAL_HEADLESS=1 for no window, and any
# toolkit diagnostic. See tools/env.sh for the paths.
#
# Button masks: START 0x0008, UP 0x0010, RIGHT 0x0020, DOWN 0x0040,
# LEFT 0x0080, TRIANGLE 0x1000, CIRCLE 0x2000, CROSS 0x4000.
# A first boot needs two STARTs: the legal screen, then the install prompt,
# e.g. PAD_SCRIPT="40:0x0008,70:0x0008".
here=${0:A:h}; source "$here/env.sh"
name=${1:?usage: run.sh <name> <seconds> [eboot|game]}; secs=${2:-120}
res_target ${3:-eboot} || exit 1
res_disc || exit 1

dir=$RES_OUT/$name; rm -rf "$dir"; mkdir -p "$dir"
cwd=${CWD:-$RES_OUT/$name.cwd}
[ -z "$KEEP" ] && [ -z "$CWD" ] && rm -rf "$cwd"
mkdir -p "$cwd/dev_hdd0" "$cwd/dev_hdd1"; cd "$cwd" || exit 1

PS3_HDD0_ROOT=${PS3_HDD0_ROOT:-$cwd/dev_hdd0} PS3_HDD1_ROOT=${PS3_HDD1_ROOT:-$cwd/dev_hdd1} \
PS3RECOMP_METAL_FRAME_DUMP=$dir/frame.ppm \
    caffeinate -i "$B" "$E" > "$RES_OUT/$name.log" 2>&1 &
P=$!
start=$(date +%s); last=0; t=0
while kill -0 $P 2>/dev/null; do
    sleep 1; t=$(( $(date +%s) - start ))
    if [ $(( t - last )) -ge 10 ]; then
        last=$t
        [ -f "$dir/frame.ppm" ] && cp "$dir/frame.ppm" "$dir/t$(printf %03d $t).ppm"
    fi
    [ $t -ge $secs ] && break
done
pkill -9 -P $P 2>/dev/null; kill -9 $P 2>/dev/null; wait $P 2>/dev/null
rm -f "$dir/frame.ppm"
for f in "$dir"/t*.ppm(N); do "$PY" "$here/ppm2png.py" "$f" "${f%.ppm}.png" >/dev/null && rm -f "$f"; done
echo "log=$RES_OUT/$name.log lines=$(wc -l < "$RES_OUT/$name.log") ran=${t}s frames=$(ls "$dir" | wc -l | tr -d ' ')"
