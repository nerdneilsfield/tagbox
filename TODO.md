# TagBox TODO

## Future Enhancements

### File Notes System
- [ ] Add `file_notes` table for multiple notes per file
- [ ] Support different note types (summary, note, comment, review, highlight, bookmark)
- [ ] Support note sources (user, ai_generated, extracted, imported)
- [ ] Implement note hierarchy (parent_note_id for nested notes)
- [ ] Add note search and indexing
- [ ] Integrate notes with FTS5 search
- [ ] Add note management APIs

#### Proposed Schema
```sql
CREATE TABLE file_notes (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    type TEXT NOT NULL,        -- 'summary', 'note', 'comment', 'review', 'highlight', 'bookmark'
    source TEXT NOT NULL,      -- 'user', 'ai_generated', 'extracted', 'imported'
    title TEXT,               -- 笔记标题（可选）
    content TEXT NOT NULL,    -- 笔记内容
    language TEXT,            -- 笔记语言
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT,          -- 创建者
    parent_note_id TEXT,      -- 支持嵌套笔记
    is_deleted INTEGER NOT NULL DEFAULT 0,
    metadata TEXT,            -- JSON格式的额外元数据
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_note_id) REFERENCES file_notes(id) ON DELETE SET NULL
);
```

## Current Issues
- [ ] Fix schema inconsistencies between init-db.rs and schema.rs
- [ ] Standardize tags table structure