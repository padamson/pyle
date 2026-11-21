#!/usr/bin/env bash
# Build pyle's landing page as the snapshots the browser tests deploy.
#
#   ./scripts/site.sh    # target/site/dev, target/site/v0.1.0, target/site/v0.2.0
#
# pyle has no releases, so 0.1.0 and 0.2.0 are fixtures: two releases, so
# that one is older than the other. Each build is made for the folder it is
# deployed to under `/pyle/` (GitHub project Pages), and `SITE_VERSION` names
# the snapshot the page says it is. Needs Trunk and the
# wasm32-unknown-unknown target.
set -euo pipefail

cd "$(dirname "$0")/.."
root=$PWD

build() {
  local version=$1 folder=$2
  (
    cd crates/site
    SITE_VERSION="$version" trunk build \
      --dist "$root/target/site/$folder" \
      --public-url "/pyle/$folder/"
  )
}

build dev dev
build 0.1.0 v0.1.0
build 0.2.0 v0.2.0
