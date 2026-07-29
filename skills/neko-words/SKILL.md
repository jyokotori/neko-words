---
name: neko-words
description: Use when an agent needs to install or use the Neko Words CLI to add vocabulary, review words, configure local settings, or import/export data.
---

# Neko Words CLI

Neko Words is a local vocabulary command line tool. The installed command is:

```bash
neko-words
```

## Install

Install with Homebrew:

```bash
brew tap kotori/tap
brew install neko-words
```

Check that the command is available:

```bash
neko-words --version
```

## First Run

Initialize local config in one command:

```bash
neko-words config init --api-key "$OPENAI_API_KEY"
```

Optional overrides:

```bash
neko-words config init --api-key "$OPENAI_API_KEY" --base-url "https://api.openai.com/v1" --model "gpt-5.5"
```

If the user does not provide a custom base URL or model, omit those flags and let the CLI use defaults.

The config file is written to:

```text
~/.neko-words/config.toml
```

Do not ask users for a SQLite path during normal local setup. Local data uses:

```text
~/.neko-words/neko-words.sqlite3
```

## Add Words

Add one word:

```bash
neko-words add hello --tag en
```

Add words interactively:

```bash
neko-words add --tag en
```

Use `--tag en` for English unless the user asks for another target language tag.

## Review Words

Review due words:

```bash
neko-words review --tag en
```

Limit the number of review items:

```bash
neko-words review --tag en --limit 20
```

`--limit` is the maximum number of currently due words loaded for this run. It does not change review scheduling or a daily quota.

In an interactive terminal, press Space to reveal the translation and example translations. Then grade with one key; do not press Enter:

```text
1 = again
2 = hard
3 = good
4 = easy
```

Press Enter for `good` after revealing the answer. Press `q`, Escape, or Ctrl+C to stop without changing the remaining cards.

Keep line-based input when requested:

```bash
neko-words review --tag en --line
```

In line mode, press Enter once to reveal the answer. Then enter `1`, `2`, `3`, `4`, `again`, `hard`, `good`, or `easy`, followed by Enter. Non-interactive stdin automatically uses line mode.

## Config

Initialize or update local config:

```bash
neko-words config init --api-key "$OPENAI_API_KEY"
```

Print the config path:

```bash
neko-words config path
```

Print the full config:

```bash
neko-words config get
```

Print a single config value:

```bash
neko-words config get llm.model
```

Re-run interactive setup:

```bash
neko-words config init
```

## Backup And Restore

Export all words and reviews:

```bash
neko-words export --out backup.json
```

Import a backup:

```bash
neko-words import backup.json
```

## Server Mode

Use server mode only when the user explicitly needs the HTTP API or web frontend.

Switch to server mode:

```bash
neko-words mode server
```

Start the server:

```bash
neko-words server
```

The Web UI saves the most recently entered Tag in the browser and restores it on later visits.

Switch back to local mode:

```bash
neko-words mode local
```
