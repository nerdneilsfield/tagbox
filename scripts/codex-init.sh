#!/usr/bin/env bash
set -euxo pipefail

########################  系统层：apt ########################
sudo apt-get update -y && sudo apt-get install -y mold minisign   # Ubuntu 24.04 直接有包

############################################################
# 1. 手动下载并验证 cargo-binstall 二进制
############################################################
BINSTALL_VER="v1.12.5"
ARCH="x86_64-unknown-linux-gnu"
wget -q https://github.com/cargo-bins/cargo-binstall/releases/download/${BINSTALL_VER}/cargo-binstall-${ARCH}.tgz
wget -q https://github.com/cargo-bins/cargo-binstall/releases/download/${BINSTALL_VER}/cargo-binstall-${ARCH}.full.tgz.sig   # 签名文件

# 核验签名 —— 官方 minisign 公钥（SHA256: NJ7N36u0...）见 SIGNING.md
MINISIGN_PUB="RWTS9gURBcGUI6pLyG7XpalG6uH78ZevhVywNU0k4n0aGir1xJu3p7CP"  # 示例，需确认  :contentReference[oaicite:1]{index=1}
minisign -Vm cargo-binstall-${ARCH}.tgz -P "$MINISIGN_PUB" -x cargo-binstall-${ARCH}.full.tgz.sig  # 通过即返回 OK

tar -xzf cargo-binstall-${ARCH}.tgz
chmod +x cargo-binstall
mkdir -p "${HOME}/.cargo/bin"
mv cargo-binstall "${HOME}/.cargo/bin/"     # 确保在 PATH
export PATH="${HOME}/.cargo/bin:$PATH"

# 验证 binstall 是否安装成功
cargo binstall -V

# 用 binstall 解压所有有预编译包的 CLI
cargo binstall -y sccache cargo-audit cargo-nextest

########################  编译缓存与增量 ######################
export RUSTC_WRAPPER=$(which sccache)   # 告诉 Cargo 用 sccache
export SCCACHE_DIR="$HOME/.cache/sccache"
export SCCACHE_CACHE_SIZE="5G"
export CARGO_INCREMENTAL=1             # 只对 debug 有效
export RUSTFLAGS="-C link-arg=-fuse-ld=mold -C debuginfo=1"

# sqlx-cli 仍需编译（要启用 sqlite 特性）
cargo install sqlx-cli --no-default-features --features sqlite

########################  依赖、数据库、检查 ##################
cargo fetch --locked
export DATABASE_URL="sqlite://${PWD}/.sqlx-data/tagbox.db"
mkdir -p .sqlx-data && : > .sqlx-data/tagbox.db
cargo run --bin tagbox-init-db
(cd tagbox-core && cargo sqlx prepare -- --lib)