# TagBox Tools

TagBox 工具集合，提供各种实用工具来管理和分析文件。

## 工具列表

### tagbox-show-pdf-info

提取并显示 PDF 文件的元信息的工具，支持两种提取方式：

1. **直接提取**：直接使用 lopdf 库提取 PDF 元信息
2. **MetaInfoExtractor 提取**：使用 tagbox-core 的 MetaInfoExtractor 进行提取

#### 使用方法

```bash
# 构建工具
cargo build -p tagbox-tools --bin tagbox-show-pdf-info

# 显示帮助信息
cargo run -p tagbox-tools --bin tagbox-show-pdf-info -- --help

# 提取 PDF 元信息（显示两种方式的结果）
cargo run -p tagbox-tools --bin tagbox-show-pdf-info -- -f /path/to/your/file.pdf

# 仅显示直接提取的结果
cargo run -p tagbox-tools --bin tagbox-show-pdf-info -- -f /path/to/your/file.pdf --direct-only

# 仅显示 MetaInfoExtractor 提取的结果
cargo run -p tagbox-tools --bin tagbox-show-pdf-info -- -f /path/to/your/file.pdf --extractor-only

# 以 JSON 格式输出结果
cargo run -p tagbox-tools --bin tagbox-show-pdf-info -- -f /path/to/your/file.pdf --json
```

#### 功能特性

- ✅ 提取 PDF 基本信息（页数、文件大小）
- ✅ 提取 PDF 元数据（标题、作者、主题、关键词、创建时间等）
- ✅ 提取文本内容（前几页的文本，用于全文搜索）
- ✅ 支持加密或损坏 PDF 的备用提取方案（pdf-extract）
- ✅ 支持两种提取方式的对比测试
- ✅ 支持 JSON 和格式化文本输出
- ✅ 详细的提取过程日志输出

#### 输出示例

工具会显示：
- 📄 直接提取的详细过程和结果
- 🔧 MetaInfoExtractor 的提取结果  
- 📊 格式化的元数据对比
- 📋 附加信息（创建者、生产者等）
- 📄 文件元数据（JSON 格式）
- 🔧 类型元数据（JSON 格式）

### tagbox-init-db

初始化 TagBox 数据库的工具。

## 开发

```bash
# 构建所有工具
cargo build -p tagbox-tools

# 运行测试
cargo test -p tagbox-tools
```

## 依赖项

- `clap`: 命令行参数解析
- `lopdf`: PDF 文件解析
- `pdf-extract`: PDF 文本提取备用方案
- `tracing`: 日志记录
- `tagbox-core`: TagBox 核心功能
- `serde_json`: JSON 序列化 