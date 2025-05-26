# BUILD NOTES

## For Converage

```bash
rustup component add llvm-tools-preview
cargo install grcov

# 配置测试覆盖率收集
export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="tagbox-%p-%m.profraw"
```

## Prepare `sqlx`

Install `sqlx-cli` first.

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

Prepare database file

```bash
# if you use Linux,MacOS, or WSL2
mkdir -p .sqlx-data
touch .sqlx-data/tagbox.db
export DATABASE_URL="sqlite://${PWD}.sqlx-data/tagbox.db"
```

```powershell
# if you use Windows
mkdir -Force .sqlx-data
New-Item -Path .sqlx-data/tagbox.db -ItemType File
$env:DATABASE_URL = "sqlite://${PWD}\.sqlx-data\tagbox.db"
```

Build the `init-db` tools:

```bash
cd tagbox-toolx
cargo run --bin init-db
```

Prepare `sqlx` database schema

```bash
cd tagbox-core
cargo sqlx prepare -- --lib
```
