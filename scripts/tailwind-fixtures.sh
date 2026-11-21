#!/usr/bin/env bash
# Rebuild the Tailwind stylesheets the layer tests in crates/pyle-e2e read,
# from the inputs beside them, with the Tailwind release the sites build
# with. The built files are committed, so the tests and CI need no Tailwind.
#
#   ./scripts/tailwind-fixtures.sh
#
# TAILWINDCSS names the Tailwind CLI to run. The default is the copy Trunk
# downloads for a site that pins `tailwindcss = "4.3.5"` in its Trunk.toml
# (macOS's cache folder). Rerun this when the sites move to a new release,
# and change the version here with them.
set -euo pipefail

version=4.3.5
tailwind=${TAILWINDCSS:-$HOME/Library/Caches/dev.trunkrs.trunk/tailwindcss-$version/tailwindcss}

cd "$(dirname "$0")/../crates/pyle-e2e/tests/fixtures"

# Read all of the help first: `grep -q` stops at its match, and under
# pipefail the CLI's SIGPIPE would fail the check.
help=$("$tailwind" --help 2>&1 || true)
if [[ $help != *"v$version"* ]]; then
  echo "error: $tailwind is not Tailwind v$version; set TAILWINDCSS" >&2
  exit 1
fi

for name in tailwind-ordered tailwind-unordered; do
  "$tailwind" --input "$name.input.css" --output "$name.css"
done
