mkdir -Force .sqlx-data
Remove-Item -Path .sqlx-data/tagbox.db -Force
New-Item -Path .sqlx-data/tagbox.db -ItemType File

$env:DATABASE_URL = "sqlite://${PWD}\.sqlx-data\tagbox.db"

cd tagbox-core
cargo run --bin tagbox-init-db