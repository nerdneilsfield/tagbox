#!/usr/bin/env bash

## fetch dependencies
rustup component add rustfmt
rustup component add clippy
cargo install cargo-audit
cargo install cargo-nextest
# for benchmark
# cargo install cargo-benchmarks


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
