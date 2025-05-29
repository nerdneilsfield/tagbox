# TagBox

A modular, offline-first file management system written in Rust. Supports full-text search, tag/category classification, semantic linking, and metadata extraction.

## Features

- **Offline-first**: All data stored locally in SQLite with FTS5 full-text search
- **Three-level categories**: Simple hierarchical organization (e.g., "Tech/Programming/Rust")
- **Interactive editing**: Edit file metadata with interactive prompts
- **File reorganization**: Automatically reorganize files based on category structure
- **Semantic linking**: Link related files with custom relationship types
- **Rich metadata**: Extract and manage titles, authors, tags, summaries, and more
- **Multiple interfaces**: CLI, GUI (FLTK), TUI, and JSON-RPC stdio mode

## Quick Start

```bash
# Initialize configuration
tagbox init-config

# Initialize database
tagbox db init

# Import files with interactive metadata editing
tagbox import /path/to/files --interactive

# Edit file metadata
tagbox edit <file-id> --title "New Title" --category "Tech/Programming/Rust" --mv

# Search files
tagbox search "tag:rust author:someone"

# Rebuild file organization
tagbox rebuild --apply
```

For detailed CLI documentation, see [docs/cli-design.md](docs/cli-design.md).