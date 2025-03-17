#!/bin/sh

set -euo pipefail 

SYNCER_FILE="$(find /mnt/SDCARD/App -type f -name 'syncer-daemon')"
SYNCER_ROOT="$(dirname "$SYNCER_FILE")"

if [ "$SYNCER_ROOT" == "" ]; then 
    echo "ERROR: Could not determine run root." >&2;
    exit -1; 
fi

exec 1>>"$SYNCER_ROOT/daemon-wrapper.out"
exec 2>>"$SYNCER_ROOT/daemon-wrapper.err"

if [ ! -d "$SYNCER_ROOT" ]; then 
    echo "ERROR: Could not find syncer root; path $SYNCER_ROOT does not exist." >&2;
    exit -1;
fi

cd "$SYNCER_ROOT"

if [ ! -f "./syncer-daemon" ]; then 
    echo "ERROR: Daemon not found."
    exit -1 
fi
export NO_COLOR=1
export ROM_SYNC_LOG=trace
export RUST_BACKTRACE=1
./syncer-daemon > "$SYNCER_ROOT/daemon.out" 2> "$SYNCER_ROOT/daemon.err" & 