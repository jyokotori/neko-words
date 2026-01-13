# Neko Words CLI

Command line interface for Neko Words - quickly add vocabulary from your terminal.

## Installation

### Option 1: uv tool (recommended)

If you use [uv](https://github.com/astral-sh/uv):

```bash
cd cli
uv tool install .

# Use it anywhere
nekowords add hello
```

> **⚠️ Important**: When updating the CLI after code changes, you must **uninstall first**, then reinstall:
> ```bash
> uv tool uninstall nekowords-cli
> uv tool install .
> ```
> Using `uv tool install . --force` may not properly update the code!

### Option 2: Development mode

For development or if you prefer not to install globally:

```bash
cd cli
uv sync

# Run with uv
uv run nekowords add hello

# Or create a shell alias
alias nekowords="uv run --project /path/to/neko-words/cli nekowords"
```

## Configuration

The CLI reads configuration from environment variables.

### Environment variables

Set these in your shell profile (`~/.zprofile`, `~/.bashrc`, etc.):

```bash
export NEKO_API_BASE_URL="http://your-server:8002/api/v1"
export NEKO_DEFAULT_LANGUAGE="en"  # optional, defaults to "en"
```

Then restart your terminal or run `source ~/.zprofile`.

### Local `.env` for development

When using `uv run` for development, you can create a `.env` file in the `cli/` directory:

```env
NEKO_API_BASE_URL=http://localhost:8002/api/v1
NEKO_DEFAULT_LANGUAGE=en
```

**Priority**: Environment variables > `.env` file > default values

## Usage

```bash
# Add a word
nekowords add hello

# Add a phrase (use quotes)
nekowords add "roll out"
nekowords add "good morning"

# Interactive mode (enter words/phrases one at a time)
nekowords add

# Add with specific language tag
nekowords add bonjour --tag fr

# Start review session
nekowords review

# Show version
nekowords --version

# Show help
nekowords --help
```

## Commands

| Command | Description |
|---------|-------------|
| `add <word>` | Add a new word to your vocabulary |
| `review` | Start an interactive review session |

## Requirements

- Python 3.12+
- A running Neko Words API server
