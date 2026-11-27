#!/bin/zsh
# Publishes the plugin as a GitHub release that SolidCraft offers as an update: plugin.json's
# "updates" names this repo, and SolidCraft (0.49+) checks its latest release at start and once a
# day. It offers a stable release tagged with a newer version (v1.2.3) that carries
# cabinet-studio-<version>.zip with a sha256 digest GitHub lists (docs/plugins.md in SolidCraft:
# Updates). Bump "version" in plugin.json first; this builds, tests, checks, tags and publishes it.
#
#   scripts/release.sh "What changed, in a sentence or two"
set -euo pipefail
root=${0:A:h:h}
cd "$root"
notes=${1:?say what changed: scripts/release.sh "notes"}
version=$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' plugin/cabinet-studio/plugin.json)
[[ $version =~ '^[0-9]+\.[0-9]+\.[0-9]+$' ]] || { echo "version $version is not major.minor.patch"; exit 1; }
if gh release view "v$version" >/dev/null 2>&1; then
  echo "v$version is already released: bump \"version\" in plugin/cabinet-studio/plugin.json"; exit 1
fi
[[ -z $(git status --porcelain) ]] || { echo "commit your changes first"; exit 1; }
scripts/build.sh --test --zip
zip="target/cabinet-studio-$version.zip"
git push -q origin HEAD
gh release create "v$version" "$zip" --title "Cabinet Studio $version" --notes "$notes" --target "$(git rev-parse HEAD)"
# SolidCraft only offers an asset whose sha256 GitHub lists: check it's there and matches.
listed=$(gh release view "v$version" --json assets --jq '.assets[0].digest')
local_sum="sha256:$(shasum -a 256 "$zip" | cut -d' ' -f1)"
[[ $listed == "$local_sum" ]] || { echo "GitHub lists $listed for the zip, not $local_sum"; exit 1; }
echo "released v$version: SolidCraft offers it at its next start (Preferences › Plugins › Update)"
