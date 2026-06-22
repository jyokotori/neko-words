## Language Settings

- Default working language: Chinese (Simplified).
- If the user explicitly requests another language, use that language for responses and user-facing copy.
- Do not use emojis in code.

## Project Conventions

- This repository is a Rust workspace for Neko Words.
- Prefer small, focused changes that preserve the existing CLI/server/core structure.
- Keep the local default database path as `~/.neko-words/neko-words.sqlite3`; do not prompt users for it in the normal local setup flow.
- Runtime configuration should come from `~/.neko-words/config.toml` unless the user explicitly asks to add another source.
- First-run CLI prompts should be written for normal users, not implementation details.
- Keep CLI prompts in English unless the user explicitly asks for another language.
- API keys are required when configuring LLM access. Base URL and model should have sensible defaults.

## Validation

- After Rust code changes, run at least:

```bash
cargo check -p neko-cli
```

- If CLI behavior changes, rebuild before testing the binary:

```bash
cargo build -p neko-cli
```
