# Neko Words

Neko Words is a local-first vocabulary builder with spaced repetition and LLM-powered word enrichment.

The app is SQLite-only. Local CLI mode and server mode both store data in a local SQLite database. Server mode exposes the same data through HTTP and serves a built-in minimal web page for adding and reviewing words. There is no realtime sync. Manual sync uses JSON export/import.

## Project Structure

- `crates/neko-core`: shared domain models, review algorithm, LLM client, service layer, and SQLite repository.
- `crates/neko-cli`: `neko-words` command-line application.
- `crates/neko-server`: Axum HTTP API server and built-in HTML UI.
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
# auth_token = "shared-secret"

[server]
bind = "127.0.0.1:8002"
db_path = "~/.neko-words/neko-words.sqlite3"
# auth_token = "shared-secret"

[llm]
api_key = "sk-your-api-key"
base_url = "https://api.openai.com/v1"
model = "gpt-5.5"
target_language = "Chinese"
```

## Build

```bash
cargo build
```

## CLI Usage

```bash
cargo run -p neko-cli -- config init --api-key sk-your-api-key
cargo run -p neko-cli -- add hello --tag default
cargo run -p neko-cli -- add --tag default
cargo run -p neko-cli -- review --tag default --limit 50
cargo run -p neko-cli -- mode local
cargo run -p neko-cli -- mode server
cargo run -p neko-cli -- config path
cargo run -p neko-cli -- config get
cargo run -p neko-cli -- config init
cargo run -p neko-cli -- export --out backup.json
cargo run -p neko-cli -- import backup.json
```

Interactive reviews use immediate single-key grading: `1` = again, `2` = hard,
`3` = good, and `4` = easy. Press Enter for good or `q` to stop. Use
`review --line` to keep line-based input that requires Enter; non-interactive
stdin automatically uses line mode.

The installed binary name is `neko-words`.

`--tag` is a label for grouping and filtering words. It does not tell the LLM what source language the word is. The LLM detects the source language itself and writes definitions/example translations in `[llm].target_language`.

## Server

Start the API and built-in web page with:

```bash
cargo run -p neko-cli -- server
```

The server listens on `[server].bind` from `~/.neko-words/config.toml`. For a one-off bind address override:

```bash
cargo run -p neko-cli -- server --bind 0.0.0.0:8002
```

When the effective bind address is reachable from other devices and `[server].auth_token` is missing, the CLI prompts for a token. Press Enter to use the generated 16-character token; it is saved to `~/.neko-words/config.toml`. The built-in web page stores the token in browser local storage after you enter it.

or:

```bash
cargo run -p neko-server
```

The HTTP API is mounted under `/api/v1`:

- `POST /api/v1/words/` with JSON body `{ "word": "hello", "tag": "default" }`
- `GET /api/v1/reviews/due?tag=default&limit=50`
- `POST /api/v1/reviews/{word_id}/log`
- `POST /api/v1/reviews/{word_id}/undo`
- `GET /api/v1/export`
- `POST /api/v1/import`

The built-in web page is served from `/` on the same port:

```text
http://127.0.0.1:8002/
```

The page supports adding words and reviewing due cards from desktop or mobile browsers. The CLI can use the same server by switching to server mode:

```bash
cargo run -p neko-cli -- mode server
cargo run -p neko-cli -- add hello --tag default
cargo run -p neko-cli -- review --tag default
```

Server mode uses SQLite at `[server].db_path`. By default this is the same file as local mode, `~/.neko-words/neko-words.sqlite3`.

### Authentication

The API is unauthenticated by default, which is fine while `bind` stays on `127.0.0.1`. Before exposing the server on a non-local address, set `[server].auth_token` to a shared secret: every `/api/v1` request, including `/export` and `/import`, must then send `Authorization: Bearer <token>`. The CLI sends it automatically from `[client_server].auth_token`.

## Manual Sync

`export`/`import` move all words and reviews through a schema-agnostic JSON file:

```bash
cargo run -p neko-cli -- export --out backup.json
cargo run -p neko-cli -- import backup.json
```

Both commands follow the configured `mode`: in local mode they read/write the SQLite database directly; in server mode they call the `GET /export` and `POST /import` endpoints over HTTP. `import` is idempotent by id, and any word that arrives without a review row gets a fresh initial review so it is immediately due.

## Docker

The compose setup runs the Rust server. It mounts your host `~/.neko-words` directory into the server container so the same SQLite database and config are used. The server provides both the API and the built-in web page on port `8002`.

```bash
docker compose up --build
```

## Development Checks

```bash
cargo fmt
cargo check
cargo test
```
