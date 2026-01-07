import os
from pathlib import Path
from dotenv import load_dotenv

# Load .env file from CLI project directory (for local development with `uv run`)
# override=False ensures environment variables take priority
_env_path = Path(__file__).parent.parent / ".env"
if _env_path.exists():
    load_dotenv(_env_path, override=False)


class Settings:
    """Settings class that reads from environment variables.
    
    For production: Set NEKO_API_BASE_URL in your shell profile (~/.zprofile)
    For development: Create a .env file in the cli/ directory
    """

    @property
    def API_BASE_URL(self) -> str:
        return os.environ.get("NEKO_API_BASE_URL", "http://localhost:8002/api/v1")

    @property
    def DEFAULT_LANGUAGE(self) -> str:
        return os.environ.get("NEKO_DEFAULT_LANGUAGE", "en")


settings = Settings()
