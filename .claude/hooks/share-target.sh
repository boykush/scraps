#!/bin/bash

# A built target/ is several GB, and every session worktree would grow its own.
# Linking it to the main checkout's keeps one copy while ./target/... paths
# still resolve. Builds from sessions running at once wait on each other's lock.

cd "$(jq -r '.cwd // "."')" || exit 0
top=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
root=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")

if [ "$top" != "$root" ] && [ ! -e "$top/target" ]; then
  mkdir -p "$root/target" && ln -s "$root/target" "$top/target"
fi
exit 0
