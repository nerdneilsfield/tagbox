
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

* `-d` or `--delete` — delete original after import(copy and detele)
* `-c` or `--category`specify the category for file to store(relative path of storage_path)
* `--category-id` use id to specify the category
* `--title` specify the title of the file
....with the same, use can specify many of the file attribute（`files` table) in command
*  `--meta-file` json file use metafile to set the file the sttribute (`files` table)

```sh
tagbox import ./papers --delete --title "papers 1" --author "Wo "
tagbox import ./papers --delete --title "papers 1" --authors  "Wo,Ni"
```

### `import-url <url>`

Download and import a file from a URL.

* `--rename <name>` — override filename
* `-c` or `--category`specify the category for file to store(relative path of storage_path)
* `--category-id` use id to specify the category
* `--title` specify the title of the file
....with the same, use can specify many of the file attribute（`files` table) in command
*  `--meta-file` json file use metafile to set the file the sttribute (`files` table)


```sh
tagbox import-url https://example.com/book.pdf --rename rust.pdf
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
