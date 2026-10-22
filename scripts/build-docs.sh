#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
zensical build --strict
# Isolate the public API from stale/private documentation in other Cargo builds.
RUSTDOCFLAGS="-D warnings" cargo doc -p remain-core --no-deps --locked --target-dir target/documentation
mkdir -p site/api
cp -R target/documentation/doc/. site/api/
