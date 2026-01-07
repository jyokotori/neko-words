# Neko Words 🐱

A simple, self-hosted vocabulary builder with Spaced Repetition (Anki-like) and LLM-powered enrichment.

## 📂 Project Structure

- `api/`: Backend service (Python + FastAPI + SQLModel)
- `web/`: Frontend application (React + Vite)
- `cli/`: Command-line interface (Python + Typer)
- `docs/`: Documentation (Requirements & Architecture)

## 🚀 Quick Start (Docker)

### Option 1: Built-in Database (Recommended)

The simplest way to get started - automatically creates a PostgreSQL database:

```bash
# 1. Configure environment variables
cp .env.example .env
# Edit .env and set your OpenAI API Key

# 2. Start all services
docker-compose up -d

# 3. Open http://localhost:3007
```

### Option 2: External Database

If you already have a PostgreSQL server:

```bash
# 1. Configure environment variables
cp .env.example .env
# Edit .env with database connection info and OpenAI API Key
```

```env
# Database configuration
NEKO_DB_HOST=your-db-host
NEKO_DB_PORT=5432
NEKO_DB_DATABASE=nekowords
NEKO_DB_USERNAME=your-username
NEKO_DB_PASSWORD=your-password
```

```bash
# 2. Start services (without database)
docker-compose -f docker-compose-external-db.yml up -d

# 3. Open http://localhost:3007
```

Web UI: `http://localhost:3007`

### LLM Configuration

**OpenAI (default, recommended):**
```env
NEKO_OPENAI_API_KEY=sk-your-api-key
NEKO_OPENAI_MODEL=gpt-4o-mini
```

**Azure OpenAI:**
```env
NEKO_LLM_PROVIDER=azure
NEKO_AZURE_OPENAI_API_KEY=your-azure-key
NEKO_AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
NEKO_AZURE_DEPLOYMENT_NAME=gpt-4o
```

---

## 🛠 Development Setup

For local development without Docker:

### Prerequisites

- **Python 3.12+** (Managed by `uv`)
- **Node.js 18+**
- **PostgreSQL 16+**
- **uv**: `curl -LsSf https://astral.sh/uv/install.sh | sh`

### 1. Database Setup

```bash
# Option A: Use Docker for database only
docker-compose up -d db

# Option B: Use external PostgreSQL
# Create database named `nekowords`
```

### 2. Backend (`api/`)

```bash
cd api
cp .env.example .env
# Edit .env with your configuration

uv sync
uv run uvicorn app.main:app --host 0.0.0.0 --port 8002 --reload
```

API docs: `http://localhost:8002/docs`

### 3. Frontend (`web/`)

```bash
cd web
npm install
npm run dev
```

Frontend: `http://localhost:3007`

### 4. CLI (`cli/`) (In Progress)

```bash
cd cli
uv sync
uv run nekowords --help
```

## 🛠 Features

- **Add Words**: Automatically brings translation and examples via LLM.
- **Review**: Spaced repetition algorithm (SM-2 adjusted) to help you remember.
- **Multi-platform**: Web UI for review, CLI for quick capture.

## 📄 Documentation

See `docs/` for detailed requirements and architecture.
