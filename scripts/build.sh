#!/bin/zsh
# Builds the plugin (docs/PLAN.md): the component into plugin/cabinet-studio, checked by
# solidcraft-cli. --test runs its test cases; --zip packs it for Preferences › Plugins › Install plugin…
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
root=${0:A:h:h}
cli=${SOLIDCRAFT_CLI:-/Applications/SolidCraft.app/Contents/MacOS/solidcraft-cli}
cd "$root"
cargo build -q --release --target wasm32-unknown-unknown
wasm-tools component new target/wasm32-unknown-unknown/release/cabinet_plugin.wasm \
  -o plugin/cabinet-studio/cabinet-studio.wasm
"$cli" plugin check plugin/cabinet-studio
for arg in "$@"; do
  case $arg in
    --test) "$cli" plugin test plugin/cabinet-studio --require-cases ;;
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
