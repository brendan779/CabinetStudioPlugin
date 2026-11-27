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
      rm -f target/cabinet-studio.zip
      (cd plugin/cabinet-studio && zip -qr ../../target/cabinet-studio.zip plugin.json cabinet-studio.wasm pictures icons)
      # The zip is what users install: check it too, so a file left out of it shows here.
      "$cli" plugin check target/cabinet-studio.zip | grep -q "no problems" \
        || { "$cli" plugin check target/cabinet-studio.zip; exit 1; }
      echo "packed target/cabinet-studio.zip" ;;
  esac
done
