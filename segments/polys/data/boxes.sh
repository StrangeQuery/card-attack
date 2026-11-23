#!/bin/sh
# Draws every brag box the polys show, named in boxes.txt, into boxes.rgb (52 x 37 raw RGB each, in that order):
# a number up to 7800 in chrome, "gold" the 7801 in gold, a number past it in gold (the race to the final count),
# "bright" the final count (final.txt) brighter. Needs Python with Pillow and NumPy (PYTHON, default python3).
set -e
cd "$(dirname "$0")"
PY=${PYTHON:-python3}
work=$(mktemp -d)
: > "$work/boxes.rgb"
for name in $(cat boxes.txt); do
  case $name in
    gold) args="7801 gold" ;;
    bright) args="$(cat final.txt) bright" ;;
    *) if [ "$name" -gt 7801 ]; then args="$name gold"; else args="$name"; fi ;;
  esac
  (cd "$work" && "$PY" "$OLDPWD/bragbox.py" $args && "$PY" -c "import numpy; open('boxes.rgb', 'ab').write(numpy.load('box.npy').tobytes())")
done
mv "$work/boxes.rgb" boxes.rgb
rm -r "$work"
