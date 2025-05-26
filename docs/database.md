# TagBox 数据库设计文档

[database.png](./database.png)

## 一、设计目标

TagBox 的数据库旨在实现以下核心能力：

* 高效管理本地文件的结构化元信息（如标题、作者、标签、分类）
* 支持多对多的标签、作者关系，支持作者笔名归一
* 支持全文模糊搜索（含中文）
* 支持文件之间的语义双链与来源管理
* 支持软删除机制与扩展访问记录功能

## 二、范式合规性分析

| 范式  | 要求                | 是否满足 | 说明                                              |
| --- | ----------------- | ---- | ----------------------------------------------- |
| 1NF | 所有字段为原子值，不能嵌套重复数据 | ✅    | 所有结构化字段（如 tags、authors）均已拆为独立关系表                |
| 2NF | 所有非主属性必须完全依赖主键    | ✅    | 主键采用 UUID，所有字段直接依赖 UUID 主键，无部分依赖                |
| 3NF | 消除传递依赖，字段依赖只指向主键  | ✅    | tags、categories、authors、links 等均拆分为独立逻辑实体，无冗余字段 |

数据库已在保持轻量的基础上，基本满足 1NF\~3NF 规范，确保结构清晰、扩展性强。

## 三、核心实体与关系结构

### 1. files 文件主表

| 字段名            | 类型       | 说明                   |
| -------------- | -------- | -------------------- |
| id             | TEXT     | UUID，主键              |
| initial\_hash  | TEXT     | 首次导入时的 hash          |
| current\_hash  | TEXT     | 当前 hash，用于识别是否变更     |
| relative\_path | TEXT     | 相对路径                 |
| filename       | TEXT     | 实际文件名                |
| title          | TEXT     | 标题                   |
| year           | INTEGER  | 出版年份                 |
| publisher      | TEXT     | 出版社                  |
| category\_id   | TEXT     | 所属分类，外键指向 categories |
| source\_url    | TEXT     | 来源链接                 |
| summaries      | TEXT     | 多条摘要（JSON 格式）        |
| created\_at    | DATETIME | 创建时间                 |
| updated\_at    | DATETIME | 更新时间                 |
| is\_deleted    | BOOLEAN  | 是否逻辑删除               |
| deleted\_at    | DATETIME | 删除标记时间（可选）           |

### 2. tags 标签表

| 字段名         | 类型       | 说明              |
| ----------- | -------- | --------------- |
| id          | TEXT     | UUID 主键         |
| name        | TEXT     | 标签显示名           |
| path        | TEXT     | 层级路径（如 技术/Rust） |
| created\_at | DATETIME | 创建时间            |

### 3. file\_tags 多对多标签关系表

| 字段名      | 类型   | 说明          |
| -------- | ---- | ----------- |
| file\_id | TEXT | 外键，指向 files |
| tag\_id  | TEXT | 外键，指向 tags  |

### 4. categories 分类表

| 字段名    | 类型   | 说明            |
| ------ | ---- | ------------- |
| id     | TEXT | UUID 主键       |
| path   | TEXT | 分类路径（如 书籍/编程） |
| parent | TEXT | 父分类 ID（可空）    |

### 5. authors 作者表

| 字段名         | 类型       | 说明               |
| ----------- | -------- | ---------------- |
| id          | TEXT     | UUID 主键          |
| name        | TEXT     | 显示名              |
| real\_name  | TEXT     | 本名（可空）           |
| aliases     | TEXT     | 笔名 JSON 数组       |
| bio         | TEXT     | 简介               |
| homepage    | TEXT     | 主页 / 个人网站 / 社交链接 |
| created\_at | DATETIME | 创建时间             |
| updated\_at | DATETIME | 更新时间             |
| is\_deleted | BOOLEAN  | 是否逻辑删除           |

### 6. file\_authors 多对多作者关系表

| 字段名        | 类型   | 说明            |
| ---------- | ---- | ------------- |
| file\_id   | TEXT | 外键，指向 files   |
| author\_id | TEXT | 外键，指向 authors |

### 7. author\_aliases 作者归一表

| 字段名           | 类型       | 说明                  |
| ------------- | -------- | ------------------- |
| alias\_id     | TEXT     | 笔名作者 ID（指向 authors） |
| canonical\_id | TEXT     | 主作者 ID（指向 authors）  |
| note          | TEXT     | 说明信息（如 人工归一）        |
| merged\_at    | DATETIME | 归一时间                |

### 8. file\_links 文件间关联表

| 字段名         | 类型       | 说明                           |
| ----------- | -------- | ---------------------------- |
| source\_id  | TEXT     | 外键，来源文件 ID（files）            |
| target\_id  | TEXT     | 外键，目标文件 ID（files）            |
| relation    | TEXT     | 关系类型，如 reference/translation |
| comment     | TEXT     | 备注                           |
| created\_at | DATETIME | 建立时间                         |

### 9. file\_search FTS5 虚拟表（全文索引）

```sql
CREATE VIRTUAL TABLE file_search USING fts5(
  title, tags, summaries, authors,
  content='files', content_rowid='id'
);
```

## 四、扩展功能表（未来）

### 10. file\_access\_log 文件访问日志（可选）

| 字段名          | 类型       | 说明              |
| ------------ | -------- | --------------- |
| file\_id     | TEXT     | 被访问的文件 ID       |
| accessed\_at | DATETIME | 访问时间            |
| method       | TEXT     | CLI / GUI / API |

## 五、规范与实践建议

* 所有主表建议添加：`created_at`, `updated_at`, `is_deleted` 字段，支持软删除策略
* 标签与分类均建议建立唯一索引，防止重复
* 建议定期维护清理 `is_deleted=true` 的记录（如 30 天后删除）
* 全文搜索字段建议在写入/编辑时保持同步更新
* 外键使用 ON DELETE CASCADE 以保障关系一致性
