# Neko Words CLI

Command line interface for Neko Words - quickly add vocabulary from your terminal.

## Installation

### Option 1: pipx (Recommended)

Install globally with [pipx](https://pipx.pypa.io/):

```bash
# From source
cd cli
pipx install .

# Now you can use it anywhere
nekowords add hello
```

### Option 2: uv tool

If you use [uv](https://github.com/astral-sh/uv):

```bash
cd cli
uv tool install .

# Use it anywhere
nekowords add hello
```

### Option 3: Development mode

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

Create a config file at `~/.config/nekowords/.env` or `cli/.env`:

```env
API_BASE_URL=http://localhost:8002/api/v1
DEFAULT_LANGUAGE=en
```

Or set environment variables directly:

```bash
export API_BASE_URL=http://your-server:8002/api/v1
```

## Usage

```bash
# Add a word
nekowords add hello
nekowords add "good morning"

# Add with specific language
nekowords add bonjour --lang fr

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
