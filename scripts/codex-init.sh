#!/usr/bin/env bash
set -euxo pipefail

########################  系统层：apt ########################
sudo apt-get update -y && sudo apt-get install -y mold   # Ubuntu 24.04 直接有包

########################  工具层：binstall ###################
# 一次性安装 binstall 本身
if ! command -v cargo-binstall &>/dev/null; then
  cargo install --locked cargo-binstall
fi

# 用 binstall 解压所有有预编译包的 CLI
cargo binstall -y \
  sccache \                # 预编译 → 秒装
  cargo-audit \
  cargo-nextest

########################  编译缓存与增量 ######################
export RUSTC_WRAPPER=$(which sccache)   # 告诉 Cargo 用 sccache
export SCCACHE_DIR="$HOME/.cache/sccache"
export SCCACHE_CACHE_SIZE="5G"
export CARGO_INCREMENTAL=1             # 只对 debug 有效
export RUSTFLAGS="-C link-arg=-fuse-ld=mold -C debuginfo=1"

# sqlx-cli 仍需编译（要启用 sqlite 特性）
cargo install --locked sqlx-cli --no-default-features --features sqlite

########################  依赖、数据库、检查 ##################
cargo fetch --locked
export DATABASE_URL="sqlite://${PWD}/.sqlx-data/tagbox.db"
mkdir -p .sqlx-data && : > .sqlx-data/tagbox.db
cargo run --bin tagbox-init-db
(cd tagbox-core && cargo sqlx prepare -- --lib)