#!/bin/sh

set -euo pipefail 

if [ "$0" == "" ]; then 
    echo "ERROR: Could not determine run root." >&2;
    exit -1; 
fi

SYNCER_ROOT="$(dirname "$0")"

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
./syncer-daemon > "$SYNCER_ROOT/daemon.out" 2> "$SYNCER_ROOT/daemon.err" & 