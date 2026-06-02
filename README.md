# Neko Words

Neko Words is a self-hosted vocabulary builder with spaced repetition and LLM-powered word enrichment.

The main CLI/API implementation is now Rust. The existing `web/` frontend remains in the repository, while Docker/web packaging can be updated separately.

## Project Structure

- `crates/neko-core`: shared domain models, review algorithm, LLM client, service layer, and repository trait/SQL implementation.
- `crates/neko-cli`: `neko-words` command-line application.
- `crates/neko-server`: Axum HTTP API server.
- `web/`: React/Vite frontend.
- `api/` and `cli/`: previous Python implementation kept for reference during migration.
- `docs/`: requirements and architecture notes.

## Configuration

CLI and server read/write a single config file:

```text
~/.neko-words/config.toml
```

Windows uses:

```text
%USERPROFILE%\.neko-words\config.toml
```

Runtime business configuration is not read from `NEKO_*` environment variables. If required config is missing, commands such as `neko-words add`, `neko-words review`, and `neko-words server` enter an interactive initializer.

Example config:

```toml
mode = "local"

[local]
db_path = "~/.neko-words/neko-words.sqlite3"

[client_server]
api_base_url = "http://localhost:8002/api/v1"

[server]
bind = "127.0.0.1:8002"
database_url = "postgres://neko:neko@localhost:5432/neko_words"

[llm]
api_key = "sk-your-api-key"
base_url = "https://api.openai.com/v1"
model = "gpt-5.5"
```

## Build

```bash
cargo build
```

## CLI Usage

```bash
cargo run -p neko-cli -- add hello --tag en
cargo run -p neko-cli -- add --tag en
cargo run -p neko-cli -- review --tag en --limit 50
cargo run -p neko-cli -- mode local
cargo run -p neko-cli -- mode server
cargo run -p neko-cli -- config path
cargo run -p neko-cli -- config get
cargo run -p neko-cli -- config set llm.model gpt-5.5
cargo run -p neko-cli -- config init
```

The installed binary name is `neko-words`.

## Server

Start the API with:

```bash
cargo run -p neko-cli -- server
```

or:

```bash
cargo run -p neko-server
```

The HTTP API is mounted under `/api/v1`:

- `POST /api/v1/words/`
- `GET /api/v1/reviews/due`
- `POST /api/v1/reviews/{word_id}/log`
- `POST /api/v1/reviews/{word_id}/undo`

Server mode defaults to PostgreSQL via `[server].database_url`. Local CLI mode uses SQLite at `~/.neko-words/neko-words.sqlite3` by default.

## Development Checks

```bash
cargo fmt
cargo check
cargo test
```
