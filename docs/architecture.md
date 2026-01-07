# 架构文档: Neko Words

## 1. 项目结构 (Directory Structure)
- `docs/`: 文档 (Requirement, Architecture)
- `api/`: 后端服务 (Python + FastAPI + UV)
- `web/`: 前端应用 (Web)
- `cli/`: 命令行工具 (Python + Typer + UV)

## 2. 系统架构图

```mermaid
graph TD
    User[用户]
    CLI[CLI 工具 (api/cli)]
    Web[Web 前端 (web)]
    Backend[Python 后端 API (api)]
    LLM[LLM 服务 (OpenAI/DeepSeek)]
    DB[(PostgreSQL 数据库)]

    User -->|命令行添加/复习| CLI
    User -->|浏览器复习| Web
    CLI -->|HTTP| Backend
    Web -->|HTTP| Backend
    Backend -->|Prompt| LLM
    Backend -->|SQL| DB
```

## 3. 组件 (Components)

### 3.1 数据库 (PostgreSQL)
- **Words 表**:
    - `id`: UUID (PK)
    - `word`: String (Unique constraint on word+language)
    - `language`: String (default 'en', support 'jp', etc.)
    - `translation`: String (Main translation)
    - `examples`: JSONB (List of objects: `[{ "sentence": "...", "translation": "..." }]`)
    - `created_at`: Timestamp
- **Reviews 表**:
    - `word_id`: FK -> Words.id
    - `next_review_at`: Timestamp
    - `last_reviewed_at`: Timestamp
    - `interval`: Integer (Days)
    - `ease_factor`: Float (Default 2.5)
    - `streak`: Integer
    - `history`: JSONB (Log of past reviews)

### 3.2 后端 (api/)
- **技术栈**: Python 3.12+, FastAPI, SQLAlchemy/SQLModel (Async), UV for dependency management.
- **职责**:
    - RESTful API.
    - LLM 交互 (获取 JSON 结构化数据).
    - Spaced Repetition 算法 (SM-2 implementation).

### 3.3 CLI 工具 (cli/)
- **技术栈**: Python 3.12+, Typer, UV.
- **特色功能**:
    - **Add (交互模式)**: `nekowords add` -> REPL loop. 支持批量粘贴 (空格分隔).
    - **Review ("摸鱼"模式)**: `nekowords review`
        - 界面极简，纯文本。
        - **流程**:
            1. 显示单词 -> (按空格)
            2. 显示第一个例句 (提示) -> (按空格)
            3. 显示释义 + 例句翻译 + 完整信息 -> (按空格) 默认评分 (Good)
        - **快捷键**:
            - `Space`: 下一步 / 默认评分 (Good)
            - `1`, `2`, `3`, `4`: 评分 (Again, Hard, Good, Easy) -> 自动跳下一词
            - `Enter`: 跳过当前单词
            - `Ctrl+Z`: 撤销上一个评分
            - `Ctrl+C`: 退出

### 3.4 前端 (web/)
- **技术栈**: React, Vite, TailwindCSS.
- **特色功能**:
    - **Review Mode**:
        - 支持 TTS (Text-to-Speech) 朗读单词和例句。
        - 三阶段显示 (单词 -> 提示 -> 答案).
        - 快捷键支持 (同 CLI).

## 4. API 设计 (Draft)

- `POST /api/words`: 添加单词
    - Body: `{ "word": "text", "language": "en" }`
    - Resp: `{ "id": "...", "translation": "...", "examples": [...] }`
- `GET /api/reviews/due`: 获取待复习列表
    - Query: `?limit=50&language=en`
- `POST /api/reviews/{id}/log`: 提交复习记录
    - Body: `{ "grade": "good" }` (grades: again, hard, good, easy)
- `POST /api/reviews/{id}/undo`: 撤销上次复习 (Optional)

## 5. 部署架构
使用 `docker-compose.yml` 根目录编排:
1. `db`: Postgres:16-alpine
2. `backend`: `api/Dockerfile`
3. `web`: `web/Dockerfile` (Nginx serving build)
