#!/usr/bin/env bash

source common.sh

needLocalStore '"min-free" and "max-free" are daemon options'

TODO_NixOS

fake_free=$TEST_ROOT/fake-free
export _NIX_TEST_FREE_SPACE_FILE=$fake_free

reserved_path=$TEST_ROOT/var/nix/db/reserved

expr=$(cat <<EOF
with import ${config_nix}; mkDerivation {
  name = "gc-recheck-after-lock";
  buildCommand = ''
    mkdir \$out
    echo foo > \$out/bar
  '';
}
EOF
)

wait_for_log() {
    local pattern=$1 log=$2
    for _ in {1..150}; do
        [[ -e "$log" ]] && grepQuiet "$pattern" "$log" && return 0
        sleep 0.1
    done
    fail "timed out waiting for '$pattern' in $log"
}

write_fake_free() {
    echo "$1" > "$fake_free".tmp
    mv "$fake_free".tmp "$fake_free"
}

lock_holder_pid=
trap '[[ -n "$lock_holder_pid" ]] && kill "$lock_holder_pid" 2>/dev/null || true' EXIT

# Run an auto-GC build while another process holds the GC lock.
# Set reported free space to $free_after_lock, then release the lock.
# Auto-GC output lands in $auto_gc_log.
run_scenario() {
    local name=$1 min_free=$2 max_free=$3 free_after_lock=$4

    clearStore
    write_fake_free 100
    nix store add-path --name "garbage-$name" ./nar-access.sh > /dev/null
    [[ -f "$reserved_path" ]]

    local gc_sync=$TEST_ROOT/gc-sync-$name
    local lock_holder_log=$TEST_ROOT/gc-lock-holder-$name.log
    mkfifo "$gc_sync"

    _NIX_TEST_GC_SYNC_2=$gc_sync nix-store --gc --print-dead > "$lock_holder_log" 2>&1 &
    lock_holder_pid=$!
    wait_for_log "finding garbage collector roots" "$lock_holder_log"

    auto_gc_log=$TEST_ROOT/auto-gc-$name.log
    nix build --impure -o "$TEST_ROOT/result-$name" --expr "$expr" \
        --min-free "$min_free" --max-free "$max_free" --min-free-check-interval 0 \
        > "$auto_gc_log" 2>&1 &
    local auto_gc_pid=$!
    wait_for_log "running auto-GC" "$auto_gc_log"

    write_fake_free "$free_after_lock"
    # readFile() on the fifo blocks until EOF; closing it releases the holder.
    : > "$gc_sync"
    wait "$lock_holder_pid"
    lock_holder_pid=
    wait "$auto_gc_pid"

    [[ foo = $(cat "$TEST_ROOT/result-$name/bar") ]]
}

# Space recovered during lock wait: skip traversal, keep reserve.
run_scenario recovered 1K 2K 4096
grepQuiet "skipping auto-GC because available space is now" "$auto_gc_log"
grepQuietInverse "finding garbage collector roots" "$auto_gc_log"
[[ -f "$reserved_path" ]]

# Still low after lock wait: recompute target (4K max-free - 2048 avail) and proceed.
run_scenario still-low 3K 4K 2048
grepQuiet "continuing auto-GC to free 2048 bytes" "$auto_gc_log"
grepQuiet "finding garbage collector roots" "$auto_gc_log"
