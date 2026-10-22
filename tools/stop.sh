#!/bin/zsh
# Stop the run start.sh started (caffeinate and its child), nothing else.
here=${0:A:h}; source "$here/env.sh"
P=$(cat "$RES_OUT/game.pid" 2>/dev/null)
if [ -n "$P" ]; then
    pkill -9 -P $P 2>/dev/null; kill -9 $P 2>/dev/null
    rm -f "$RES_OUT/game.pid"
fi
echo stopped
