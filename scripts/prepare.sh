#!/usr/bin/env bash

# 设置环境变量
export DATABASE_URL="sqlite://${PWD}.sqlx-data/tagbox.db"

# 准备数据库文件
mkdir -p .sqlx-data
rm -rf .sqlx-data/tagbox.db
touch .sqlx-data/tagbox.db

# init database
cargo run --bin tagbox-init-db

# 准备数据库 schema
cd tagbox-core
cargo sqlx prepare -- --lib
