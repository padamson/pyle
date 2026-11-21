#!/usr/bin/env bash
# Build or serve pyle's theoria gallery on the dependencies Cargo.lock pins.
#
#   ./scripts/gallery.sh serve    # http://localhost:8081
#   ./scripts/gallery.sh build    # target/theoria/gallery/pyle/dist
#
# The app `theoria` generates is its own workspace with its own lockfile.
# Left alone, that lockfile is resolved once, when the app is first built
# (theoria's `main` and every other dependency as of that day), and never
# follows Cargo.lock afterwards. Starting it from the workspace's lockfile
# on each run keeps the gallery on what the tests and the sites get:
# theoria writes around a lockfile that is there, and cargo drops the
# entries the app does not use. Delete this script when `theoria` does it.
#
# THEORIA names the CLI to run, for a build that is not the one on PATH.
set -euo pipefail

cd "$(dirname "$0")/.."

app=target/theoria/gallery/pyle
mkdir -p "$app"
cp Cargo.lock "$app/Cargo.lock"

exec "${THEORIA:-theoria}" "${1:-serve}" "${@:2}"
