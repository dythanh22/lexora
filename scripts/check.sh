#!/usr/bin/env sh
# Kiểm tra cục bộ giống CI. Chạy từ thư mục gốc.
set -eu
cargo fmt --all -- --check
cargo clippy -p lexora-core -p lexora-providers --all-targets -- -D warnings
cargo test -p lexora-core -p lexora-providers
npm run check
npm test
