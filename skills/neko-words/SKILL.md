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

Run the command once to create the local config:

```bash
neko-words
```

The config file is:

```text
~/.neko-words/config.toml
```

For normal local use:

- Choose `local` when asked for storage mode.
- Enter an OpenAI API key when prompted.
- Press Enter for the default OpenAI-compatible API base URL unless the user has a custom one.
- Press Enter for the default model unless the user specifies another model.

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

When the CLI asks for a grade, enter one of:

```text
again
hard
good
easy
```

## Config

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

Set a config value:

```bash
neko-words config set llm.model gpt-5.5
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

Switch back to local mode:

```bash
neko-words mode local
```
