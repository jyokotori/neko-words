from pydantic_settings import BaseSettings, SettingsConfigDict
from functools import lru_cache
from pathlib import Path


# Global config path: ~/.config/nekowords/.env
GLOBAL_ENV_PATH = Path("~/.config/nekowords/.env").expanduser()


class Settings(BaseSettings):
    API_BASE_URL: str = "http://localhost:8002/api/v1"
    DEFAULT_LANGUAGE: str = "en"

    # Load settings from:
    # 1. ~/.config/nekowords/.env (global)
    # 2. .env in current working directory / project (local override)
    model_config = SettingsConfigDict(
        env_file=(GLOBAL_ENV_PATH, ".env"),
        env_file_encoding="utf-8",
        extra="ignore",
    )


@lru_cache
def get_settings():
    return Settings()


settings = get_settings()
