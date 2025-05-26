#!/usr/bin/env bash

## fetch dependencies
rustup component add rustfmt clippy
if ! command -v cargo-binstall &> /dev/null; then
    cargo install cargo-binstall
fi
cargo binstall -y cargo-audit cargo-nextest cargo-make sccache
# for benchmark
# cargo install cargo-benchmarks

export RUSTC_WRAPPER=$(which sccache)   # 告诉 Cargo 用 sccache
export SCCACHE_DIR="$HOME/.cache/sccache"
export SCCACHE_CACHE_SIZE="5G"
#export CARGO_INCREMENTAL=1             # 只对 debug 有效 和 sccache 冲突了
export RUSTFLAGS="-C debuginfo=1"


# 提前获得依赖
cargo fetch
# 设置环境变量
export DATABASE_URL="sqlite://${PWD}/.sqlx-data/tagbox.db"

# 准备数据库文件
mkdir -p .sqlx-data
rm -rf .sqlx-data/tagbox.db
touch .sqlx-data/tagbox.db

#install sqlx-cli
cargo install sqlx-cli --no-default-features --features sqlite

# init database
cargo run --bin tagbox-init-db

# # 准备数据库 schema
cd tagbox-core
cargo sqlx prepare -- --lib

## Build all
cd ..
cargo build --all --offline
