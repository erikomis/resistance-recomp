#!/bin/zsh
# Start the port in the background, to be driven by press.sh / snap.sh.
#
#   tools/start.sh <name> [eboot|game]
#
# The run directory is $RES_OUT/game.cwd (or CWD=<dir>) and is KEPT between
# runs, so the first-run install (the *.done cache files) and the saves
# (PROFILE.SAV, quick.sav) persist. Input comes from PAD_FILE=$RES_OUT/pad
# (press.sh writes it), the log goes to $RES_OUT/<name>.log and the backend
# rewrites $RES_OUT/<name>/frame.ppm every 120 presents. stop.sh ends it.
here=${0:A:h}; source "$here/env.sh"
name=${1:?usage: start.sh <name> [eboot|game]}
res_target ${2:-eboot} || exit 1
res_disc || exit 1

mkdir -p "$RES_OUT/$name"
cwd=${CWD:-$RES_OUT/game.cwd}; mkdir -p "$cwd/dev_hdd0" "$cwd/dev_hdd1"; cd "$cwd" || exit 1
: > "$RES_OUT/pad"
PS3_HDD0_ROOT=$cwd/dev_hdd0 PS3_HDD1_ROOT=$cwd/dev_hdd1 \
PAD_FILE=$RES_OUT/pad PS3RECOMP_METAL_FRAME_DUMP=$RES_OUT/$name/frame.ppm \
    nohup caffeinate -i "$B" "$E" > "$RES_OUT/$name.log" 2>&1 &
echo $! > "$RES_OUT/game.pid"
echo "started pid $(cat "$RES_OUT/game.pid"), log $RES_OUT/$name.log"
