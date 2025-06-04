# TagBox GUI Design Guide (for FLTK-RS)

This guide defines the UI structure, behavior, and interaction logic for a modular, desktop-based file metadata management system, powered by `fltk-rs`. The structure is generic and portable to other Rust-based GUI applications.

---

## 🧭 High-Level UI Philosophy

* Minimalist and modular layout with logical separation of concerns
* DSL-first search model with optional advanced filtering
* Tree-structured browsing by categories (3-level)
* Live preview and inline editing where possible
* Seamless integration with file system (open, cd, path display)

---

## 📄 Page 1: Main File Overview (首页文件总览页)

### Layout Description

* **Top bar**: consists of a single-line input for DSL query, a button for advanced search modal, and a file import button.
* **Left panel**: shows a 3-level expandable category tree with folders and files.
* **Right panel**: shows selected file’s metadata (title, tags, path, authors, summary) with quick actions (open, preview, edit).

### DSL Search Input

* Accepts full DSL expressions: `tag:Rust author:张三 -tag:旧版`
* Pressing Enter triggers a search and updates the file list.
* Shows placeholder when empty: "Search (e.g. tag\:Rust -tag:旧版)"

### Advanced Search Modal

* Triggered by clicking `[Advanced]`
* Modal includes form fields for:

  * Tag(s): text input with auto-complete
  * Title contains: string input
  * Author: dropdown or text input
  * Category: cascading dropdown for level 1 → 2 → 3
  * Year: range input (start to end)
* A “Search” button executes and populates main list

### Category Tree Panel (左侧分类树)

* Default view when app launches
* Level 1 categories are expanded on click, revealing level 2
* Files appear as leaf nodes under level 3
* Clicking a file selects and shows preview in the right panel
* Right-click file/folder to open contextual menu:

  * Open in system file manager
  * Reclassify
  * Delete (soft)

### Preview Panel (右侧预览区域)

* Shows file metadata:

  * Title, Path (clickable), Authors, Tags, Summary (scrollable)
  * Category (breadcrumb style: 编程 > Rust > 网络)
  * File size, last modified date, import date
* Linked Files section:

  * Shows related files (via semantic links)
  * Each file is clickable
* Action buttons:

  * `[Open File]` → opens via OS
  * `[Edit]` → navigates to Page 3
  * `[CD]` → copies containing folder path
  * `[Copy Path]` → copies relative path to clipboard

---

## 🆕 Page 2: File Import Page (导入页面)

### File Selection

* Two input modes:

  1. Select file button or drag-and-drop area
  2. Input URL and `[Download]` button
* Shows filename and path preview

### Metadata Form

* Appears after file is selected or downloaded
* Fields:

  * Title (text input)
  * Authors (tag-style chips or comma list)
  * Year (number input, optional)
  * Publisher (text input)
  * Tags (multi-input + auto-complete + create-new)
  * Summary (multi-line text box, editable)
  * Category:

    * Level 1: dropdown
    * Level 2: dependent on L1
    * Level 3: dependent on L2 or free input

### Actions

* `[Extract Metadata]` → fills form from file meta (PDF title, JSON, etc.)
* `[Import and Move]` → processes file, writes to DB, moves to structured path
* `[Import and Keep Original]` → same but keeps file at original location

---

## ✏️ Page 3: File Edit Page (文件编辑页)

### Structure

* Almost identical to Import Page
* Pre-filled metadata from DB for given file ID

### Special Additions

* `[Re-extract metadata]` → re-parses the current file
* `[Reset to Original]` → undo unsaved changes
* `[Save]` / `[Cancel]` / `[Delete]`
* Linked Files viewer and editor
* Navigation breadcrumbs to return to overview

---

## 🔁 Page 4: Semantic Links View (关联文件展示区)

### Current Embedded Mode (in Preview panel)

* Section shows:

  * "Related Files" header
  * Scrollable list of FileEntry (title + ID)
  * Clickable for jump-to-edit
* Later: `Canvas`-based visual graph planned (optional)

---

## 🧩 Component Guidelines

### Category Tree

* `HoldBrowser` with nesting and indent control
* Supports dynamic expand/load
* Drag-and-drop reclassification (optional)

### DSL Search Parser

* Parser engine shared with CLI DSL (`tagbox-core`)
* Result struct passed as query into preview renderer

### File Entry Preview

* Card-style info layout
* Tags shown as colored chips
* Summary supports markdown rendering (optional)
* Clipboard buttons: path, folder

---

## 🧪 Testing Scenarios

* Fuzz test with 1k+ files, long summaries
* Edge case: same file imported multiple times
* Edge case: circular links (A → B → A)
* Drag files between categories
* Confirm layout works on 1024x768, 4k, ultrawide

---

This document is suitable for reusing in any FLTK-based file interface that supports metadata and structured knowledge management.

