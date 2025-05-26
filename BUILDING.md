# BUILD NOTES


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
