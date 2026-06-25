# 需求文档: Neko Words

## 1. 项目目标

Neko Words 是一个本地优先的单词本工具。核心目标是保持简单：一个 Rust CLI，一个 Rust server，一个本地 SQLite 数据库。server 启动后同时提供 API 和内置极简 HTML 页面。

## 2. 主要使用方式

### CLI

用户可以通过本地 `neko-words` CLI 添加单词和复习。

- `neko-words add <word> --tag en`: 添加单词。
- `neko-words add --tag en`: 进入交互式添加。
- `neko-words review --tag en`: 在终端中复习。
- 当配置为 `mode = "server"` 时，CLI 通过 server 的 API 端口工作。
- 当配置为 `mode = "local"` 时，CLI 直接读写本地 SQLite。

### 内置网页

用户也可以启动：

```bash
neko-words server
```

然后在浏览器打开 server 根路径：

```text
http://127.0.0.1:8002/
```

页面只需要覆盖两个高频功能：

- 添加单词。
- 复习到期单词。

页面必须支持手机端浏览器，布局应能在窄屏上正常输入、查看答案和评分。

## 3. 功能需求

### 添加单词

- 默认语言为 `en`。
- 添加时调用 OpenAI-compatible LLM 生成结构化信息：
  - 单词原文。
  - 中文释义。
  - 多条例句及中文翻译。
- 重复添加同一个语言下的同一个单词时，不新增重复词条，而是重置复习计划。

### 复习

- 获取到期单词列表。
- 先显示单词和例句提示。
- 用户可以显示答案。
- 支持四档评分：
  - `again`
  - `hard`
  - `good`
  - `easy`
- 支持撤销上一次评分。
- 网页端可使用浏览器语音能力朗读单词。

### API

server 在 `/api/v1` 下提供 API：

- `POST /api/v1/words/`
- `GET /api/v1/reviews/due`
- `POST /api/v1/reviews/{word_id}/log`
- `POST /api/v1/reviews/{word_id}/undo`
- `GET /api/v1/export`
- `POST /api/v1/import`

## 4. 数据与配置

- SQLite 是唯一运行时数据库。
- 默认数据库路径为 `~/.neko-words/neko-words.sqlite3`。
- 运行配置来自 `~/.neko-words/config.toml`。
- 正常本地流程不依赖 `.env` 或 `NEKO_*` 环境变量。
- API keys 配置在 `[llm]` 中。

## 5. 非功能需求

- 保持单机和本地网络使用简单。
- 默认不要求 Node、Vite、Nginx 或独立前端服务。
- Docker 部署只需要运行 Rust server；server 同时提供 API 和 HTML。
- CLI 提示保持面向普通用户，不暴露不必要的实现细节。
