#!/bin/sh
# Builds the native Rust references, one per effect, into refs/: each draws its effect's frames in ordinary Rust,
# independently of the machines, and the host checks every frame CARDIAC computes against it.
set -e
cd "$(dirname "$0")"
mkdir -p refs
for e in plasma cut; do
  rustc -O -o refs/${e}_ref segments/$e/reference.rs
done
rustc -O -o refs/roto4_ref segments/roto/reference4.rs
rustc -O -o refs/tumble_ref segments/tumble/reference.rs
rustc -O -o refs/polys_ref segments/polys/reference.rs
rustc --edition 2021 -O -o refs/deep_ref segments/deep/reference.rs
rustc -O -o refs/openers_ref segments/openers/reference.rs
rustc -O -o refs/mesh_ref segments/mesh/reference.rs
rustc -O -o refs/ui_ref segments/ui/reference.rs
rustc -O -o refs/vga_ref segments/vga/reference.rs
rustc -O -o refs/modex_ref segments/modex/reference.rs
rustc -O -o refs/partychat_ref segments/partychat/reference.rs
rustc -O -o refs/mac_ref segments/mac/reference.rs
rustc -O -o refs/turn_ref segments/turn/reference.rs
echo "references in refs/"
rustc -O -o refs/dierubik_ref segments/dierubik/reference.rs
rustc -O -o refs/bbs_ref segments/bbs/reference.rs
rustc -O -o refs/accel_ref segments/accel/reference.rs
rustc --edition 2021 -O -o refs/train_ref segments/train/reference.rs
rustc --edition 2021 -O -o refs/night_ref segments/night/reference.rs
rustc --edition 2021 -O -o refs/stadium_ref segments/stadium/reference.rs
rustc --edition 2021 -O -o refs/rooms_ref segments/rooms/reference.rs
rustc -O -o refs/rotodg_ref segments/rotodg/reference.rs
rustc --edition 2021 -O -o refs/sqend_ref segments/sqend/reference.rs
rustc --edition 2021 -O -o refs/qr_ref segments/qr/reference.rs
