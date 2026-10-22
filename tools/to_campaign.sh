#!/bin/zsh
# Boot the EBOOT and drive the front end into a new Medium campaign, which
# exitspawns game.self (the level-1 intro movie).
#
#   tools/to_campaign.sh <name>
#
# Expects the first-run install to be done already in the start.sh run
# directory ($RES_OUT/game.cwd): on a fresh one, first boot with
# start.sh and press START twice (legal screen, then the install prompt), or
# use run.sh with CWD=$RES_OUT/game.cwd and PAD_SCRIPT. Frames m1..m10 land in
# $RES_OUT/<name>/. stop.sh ends the run.
here=${0:A:h}; source "$here/env.sh"
n=${1:?usage: to_campaign.sh <name>}
cwd=${CWD:-$RES_OUT/game.cwd}
"$here/start.sh" $n || exit 1
sleep 60
"$here/press.sh" $n m1 0x0008 40 25       # START -> main menu
"$here/press.sh" $n m2 0x4000             # Campaign
"$here/press.sh" $n m3 0x4000 40 8        # New game (a warning follows when a save exists)
if [ -f "$cwd/dev_hdd0/game/BCUS98107/USRDIR/quick.sav" ]; then
    "$here/press.sh" $n m4 0x0010 6 3; "$here/press.sh" $n m5 0x4000 40 10   # Yes
    "$here/press.sh" $n m6 0x0010 6 3; "$here/press.sh" $n m7 0x4000 40 10   # overwrite: Yes
fi
"$here/press.sh" $n m8 0x0040 6 3; "$here/press.sh" $n m9 0x4000 40 8        # Medium
"$here/press.sh" $n m10 0x4000 40 20                                         # Invert-Y off -> exitspawn
