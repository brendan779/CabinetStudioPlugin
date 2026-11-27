#!/bin/zsh
# Builds the plugin (docs/PLAN.md): the component into plugin/cabinet-studio, checked by
# solidcraft-cli. --test runs its test cases; --zip packs it for Preferences › Plugins › Install plugin…
set -euo pipefail
# A Python with Pillow, for the pictures, wherever it is on the PATH.
py=""
for p in $(which -a python3 2>/dev/null) /Library/Frameworks/Python.framework/Versions/Current/bin/python3; do
  if [[ -x $p ]] && "$p" -I -c 'import PIL' 2>/dev/null; then py=$p; break; fi
done
[[ -n $py ]] || { echo "no python3 with Pillow: pip install pillow"; exit 1; }
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
root=${0:A:h:h}
cli=${SOLIDCRAFT_CLI:-/Applications/SolidCraft.app/Contents/MacOS/solidcraft-cli}
cd "$root"
# The manifest and the dialogs' pictures come from Cabinet Studio's core (scripts/make_manifest.py).
mkdir -p target
host=$(rustc -vV | sed -n 's/^host: //p')
cargo run -q --example catalog --target "$host" > target/catalog.json
cargo run -q --example catalog --target "$host" -- --boards > src/boards.tsv
"$py" -I scripts/make_manifest.py
cargo build -q --release --target wasm32-unknown-unknown
wasm-tools component new target/wasm32-unknown-unknown/release/cabinet_plugin.wasm \
  -o plugin/cabinet-studio/cabinet-studio.wasm
"$cli" plugin check plugin/cabinet-studio
for arg in "$@"; do
  case $arg in
    --test)
      cargo run -q --example check_boards --target "$host"
      # A hook reply SolidCraft throws away is only echoed in a test's log: it fails the gate here.
      logs="$root/target/test-logs"
      rm -rf "${logs:?}"
      "$cli" plugin test plugin/cabinet-studio --require-cases --out "$logs"
      if grep -h "hook reply ignored" "$logs"/*.log 2>/dev/null | sort -u | grep .; then
        echo "a dialog hook's reply was ignored (above)"; exit 1
      fi ;;
    --zip)
      # A new name for each version: the macOS file picker can grey out a file replaced while
      # it was open, so a rebuilt zip of the same name sometimes couldn't be picked.
      version=$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' plugin/cabinet-studio/plugin.json)
      zip="target/cabinet-studio-$version.zip"
      rm -f "$zip"
      (cd plugin/cabinet-studio && zip -qr "../../$zip" plugin.json cabinet-studio.wasm pictures icons)
      # The zip is what users install: check it too, so a file left out of it shows here.
      "$cli" plugin check "$zip" | grep -q "no problems" || { "$cli" plugin check "$zip"; exit 1; }
      echo "packed $zip" ;;
  esac
done
