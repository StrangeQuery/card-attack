#!/bin/sh
# Builds everything `card-attack play` needs: unpacks the generated data, compiles every machine with the installed
# rust-cardiac (cargo cardiac 0.2.5, the version the released build used), and builds the host.
# Usage: ./build.sh   (then: host/target/release/card-attack play)
set -e
cd "$(dirname "$0")"

need="0.2.5"
install="Install rust-cardiac $need by following its README: https://github.com/StrangeQuery/rust-cardiac"
command -v cargo-cardiac >/dev/null 2>&1 || { echo "build.sh: cargo-cardiac isn't on the PATH. $install" >&2; exit 1; }
have=$(cargo cardiac --version 2>/dev/null | sed -n 's/^cargo-cardiac \([0-9][0-9.]*\).*/\1/p')
if [ "$have" != "$need" ]; then
    echo "build.sh: this cargo-cardiac is ${have:-too old to say its version}, and Card Attack is built with exactly $need. $install" >&2
    exit 1
fi

echo "== the generated data"
for a in segments/*/generated.tar.xz; do tar -xJf "$a" -C segments; done

echo "== the machines (cargo-cardiac $have)"
cargo cardiac build --manifest-path segments/Cargo.toml

echo "== the host"
(cd host && cargo build --release)
echo "built: host/target/release/card-attack play"
