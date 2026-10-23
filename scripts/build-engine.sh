#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Audited integration baseline. No automatic tracking of upstream HEAD.
revision=b8630fb39624c76cdce0e196c57f7285d63286d3
source_dir="$PWD/.cache/odin4"
if [ ! -d "$source_dir/.git" ]; then
  git clone https://github.com/Llucs/odin4.git "$source_dir"
fi
git -C "$source_dir" fetch origin "$revision"
if [ -n "$(git -C "$source_dir" status --porcelain)" ]; then
  echo 'Engine checkout has local changes; refusing to overwrite.' >&2; exit 1
fi
git -C "$source_dir" checkout --detach "$revision"
cmake -S "$source_dir" -B "$source_dir/build" -G Ninja -DCMAKE_BUILD_TYPE=Release -DODIN4_BUILD_GUI=OFF -DODIN4_BUILD_TESTS=ON
cmake --build "$source_dir/build" --target odin4 odin4_tests --parallel 2
ctest --test-dir "$source_dir/build" --output-on-failure
triple="$(rustc -vV | sed -n 's/^host: //p')"
mkdir -p src-tauri/binaries
cp "$source_dir/build/odin4" "src-tauri/binaries/odin4-$triple"
cp "$source_dir/LICENSE" src-tauri/binaries/ODIN4-LICENSE
cp "$source_dir/NOTICE" src-tauri/binaries/ODIN4-NOTICE
