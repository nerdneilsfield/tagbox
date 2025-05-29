
# TagBox CLI Help

TagBox is a CLI tool to manage file metadata using local SQLite + full-text search. Supports structured imports, DSL search, and integration with external launchers.

---

## Global Flags

* `--log-level <info|warn|debug>` — control logging verbosity (default: info)
* `--quiet` — suppress normal output (overrides log level)

---

## Commands

### `import <path>`

Import a file or directory of files.

* `-d` or `--delete` — delete original after import (copy and delete)
* `--category` — specify the category path (e.g., "Tech/Programming/Rust" or "Tech/Programming" or "Tech")
* `--title` — specify the title of the file
* `--authors` — specify the authors (comma-separated)
* `--year` — specify the year
* `--publisher` — specify the publisher
* `--source` — specify the source
* `--tags` — specify tags (comma-separated)
* `--summary` — specify summary
* `--meta-file` — JSON file to set file attributes
* `-i` or `--interactive` — interactive mode - prompt for metadata after extraction

```sh
tagbox import ./papers --delete --title "Paper 1" --authors "Author1,Author2"
tagbox import ./papers --category "Tech/AI/Papers" --tags "ai,research"
tagbox import ./document.pdf --interactive
```

### `import-url <url>`

Download and import a file from a URL.

* `--rename <name>` — override filename
* `--category` — specify the category path (e.g., "Tech/Programming/Rust")
* `--title` — specify the title of the file
* `--authors` — specify the authors (comma-separated)
* `--year` — specify the year
* `--publisher` — specify the publisher
* `--source` — specify the source
* `--tags` — specify tags (comma-separated)
* `--summary` — specify summary
* `--meta-file` — JSON file to set file attributes

```sh
tagbox import-url https://example.com/book.pdf --rename rust.pdf --category "Tech/Programming"
```

### `search <query>`

Search files using DSL (`tag:Rust author:Alice`) or free text.

* `--json` — output result as JSON
* `--columns` — comma-separated fields (e.g., title,path,authors)
* `--limit`, `--offset` — pagination

```sh
tagbox search "tag:Rust author:Gray" --columns title,path --json
```

### `preview <id>`

Show a file’s metadata (title, tags, authors, path).

Options:

* `--only-meta` — only show metadata, no summary or path
* `--open` — open file with system default program
* `--cd` — print path to containing folder (useful for `cd $(...)` workflows)

```sh
tagbox preview abc123 --only-meta
# or open the file directly
tagbox preview abc123 --open
# or get the folder path
cd $(tagbox preview abc123 --cd)
```

### `edit <id>`

Edit file metadata for an existing file.

* `-i` or `--interactive` — interactive mode - prompt for each field
* `--mv` — move file to new category path after update
* `-t` or `--title` — new title
* `-a` or `--authors` — new authors (comma-separated)
* `--category` — new category (e.g., "Tech/Programming/Rust")
* `--tags` — new tags (comma-separated)
* `--summary` — new summary
* `--year` — new year
* `--publisher` — new publisher
* `--source` — new source

```sh
# Edit specific fields
tagbox edit abc123 --title "New Title" --authors "Author1,Author2" --category "Tech/Programming/Rust" --mv

# Interactive edit mode
tagbox edit abc123 --interactive

# Update category and move file
tagbox edit abc123 --category "Research/AI/Papers" --mv
```

### `rebuild [id]`

Rebuild file storage paths according to current configuration.

* `[id]` — specific file ID to rebuild (optional, if not provided rebuilds all files)
* `--apply` — actually move files (default: dry run mode)
* `--workers` — number of parallel workers for batch operations (default: 4)

```sh
# Preview what would be moved for a specific file
tagbox rebuild abc123

# Actually move a specific file
tagbox rebuild abc123 --apply

# Preview what would be moved for all files
tagbox rebuild

# Actually move all files that need reorganization
tagbox rebuild --apply --workers 8
```

### `link <id1> <id2>`

Link two files as semantically related.

* `--relation <type>` — optional label (e.g., reference)

```sh
tagbox link abc123 def456 --relation derived_from
```

### `unlink <id1> <id2>`

Remove semantic link between two files.

* `--batch` — unlink many
* `--ids <file>` — file of ID pairs

```sh
tagbox unlink abc123 def456
```

### `query-debug <dsl>`

Show SQL generated from DSL query and count preview.

```sh
tagbox query-debug "tag:Rust -tag:旧版"
```

### `author`

Manage author entries.

* `add <name>`
* `remove <id>`
* `merge <from> <to>`

```sh
tagbox author add "山海"
```

### `config`

Edit runtime parameters.

* `get <key>` / `set <key> <value>`

```sh
tagbox config set import.paths.rename_template "{title}_{year}"
```

### `export`

Dump files in JSON or CSV format.

```sh
tagbox export --json
```

### `stats`

Show tag usage, top authors, access heatmap.

```sh
tagbox stats
```

### `serve`

Launch local MCP-compatible server endpoint.

```sh
tagbox serve
```

### `stdio`

Accepts JSON from stdin and outputs JSON to stdout.
Supports commands like `search`, `import`, `preview`, etc.
Compatible with Raycast, Flow, Rofi, and other tools.

#### JSON Format Specification (inspired by JSON-RPC 2.0):

```json
{
  "jsonrpc": "2.0",       // optional, defaults to '2.0'
  "id": "abc123",         // optional, returned in response
  "cmd": "search",        // required, valid command name
  "args": {                // required, parameters match CLI arguments
    "query": "tag:Rust",
    "json": true,
    "limit": 10
  }
}
```

#### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": "abc123",            // matches request ID if provided
  "result": { ... },         // command-specific result (e.g., list of FileEntry)
  "error": null              // or an object with { code, message }
}
```

#### Example Usage

```sh
echo '{"cmd":"search","args":{"query":"tag:Rust"}}' | tagbox stdio
```

echo '{"cmd":"search","args":{"query":"tag\:Rust"}}' | tagbox stdio

```

---

## Notes
- Commands return rich JSON or pretty output based on `--json` and `--columns`
- TagBox stores all metadata locally; no network dependencies
- `.tagboxrc` will support persistent config override (future)

```


## Dependecies

- `clap` for command application
- `tabled` for table print
- `log + env_logger` for logger, logger can be set to stdout or file or both
- `indicatif` for progress(multiple import files)
