#!/bin/bash
# Builds the copy of Card Attack that is submitted: the host binary and README.txt at the top and, beside it, what `play` reads from
# segments/ (the sources, the compiled decks, the soundtrack's tracks, the data files and the cut), without renders
# or placeholders. Usage: ./package.sh FOLDER (emptied first). Build the host and the machines before.
set -e
[ -n "$1" ] || { echo "usage: $0 FOLDER" >&2; exit 1; }
rm -rf "$1"
mkdir -p "$1"
P=$(cd "$1" && pwd)
cd "$(dirname "$0")"
cp -c host/target/release/card-attack README.txt card-attack.nfo "$P/"
{ git ls-files segments
  find segments -path segments/target -prune -o -type f \( -name '*.cqb' -o -name '*.cqb.meta' -o -name '*.track' \) -print
  find segments -path segments/target -prune -o -path '*/data/*' -type f ! -name '*.py' ! -name '*.exr' ! -path '*dorm3d*' ! -name '*.rgb' ! -path '*/__pycache__/*' -print
  ls segments/cut/*.txt
} | sort -u | grep -v -e '/placeholder.rgb$' -e '/generated.tar.xz$' | while read -r f; do mkdir -p "$P/$(dirname "$f")"; cp -c "$f" "$P/$f"; done
find "$P" -type f -print0 | xargs -0 stat -f %z | awk '{s+=$1; n++} END {printf "%d files, %.1f MB (%.3f GiB)\n", n, s/1e6, s/2^30}'
