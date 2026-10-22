#!/bin/zsh
# Play: the EBOOT in a window, with a controller.
#
#   tools/play.sh
#
# Starts the front end (legal screens, menus); a new campaign exitspawns the
# game.self build in the same window. The window belongs to the launcher
# (build/BCUS98107/Resistance, built here in seconds when missing): the game
# process renders into it through shared IOSurfaces and has no window or Dock
# icon of its own, so the window stays up across the exitspawn. Closing it
# ends the game. A controller is required on macOS
# (DualSense, DualShock or Xbox, through SDL2): the toolkit's keyboard and
# mouse fallback is Windows-only. Close the game window (or Ctrl+C here) to
# quit.
#
# Saves, the install cache and the log live in RES_DATA (default
# ~/Library/Application Support/resistance-recomp/): dev_hdd0/, dev_hdd1/ and
# last.log. The window title shows the frame rate (PS3RECOMP_METAL_FPS=1).
#
# The first boot needs two STARTs: the legal screen, then the "initial setup"
# prompt (the HDD install, done once and kept in RES_DATA).
#
# Needs both builds from tools/rebuild.sh and the mounted disc; see
# tools/env.sh for the paths and overrides.
here=${0:A:h}; source "$here/env.sh"
res_target game || { echo "run tools/rebuild.sh first" >&2; exit 1; }
res_target eboot || { echo "run tools/rebuild.sh first" >&2; exit 1; }
res_disc || exit 1
res_launcher || exit 1

DATA=${RES_DATA:-$HOME/Library/Application Support/resistance-recomp}
mkdir -p "$DATA/dev_hdd0" "$DATA/dev_hdd1"; cd "$DATA" || exit 1
LOG=$DATA/last.log

if [ ! -d "$DATA/dev_hdd0/game/BCUS98107/USRDIR/cache" ]; then
    echo "First boot: press START on the legal screen, then START again on the"
    echo "\"initial setup\" prompt. The install runs once and takes a few minutes."
fi
echo "Disc: $PS3_VFS_ROOT"
echo "Saves and install cache: $DATA"
echo "Log: $LOG"

unset PS3RECOMP_METAL_HEADLESS
PS3_HDD0_ROOT=$DATA/dev_hdd0 PS3_HDD1_ROOT=$DATA/dev_hdd1 \
PS3RECOMP_METAL_FPS=1 \
    caffeinate -i "$L" -- "$B" "$E" > "$LOG" 2>&1
st=$?
echo "The game exited (status $st). Log: $LOG"
exit $st
