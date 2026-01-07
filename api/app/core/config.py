from pydantic_settings import BaseSettings, SettingsConfigDict
from functools import lru_cache
from typing import Optional, Literal

class Settings(BaseSettings):
    API_V1_STR: str = "/api/v1"
    PROJECT_NAME: str = "Neko Words"

    # Database
    DB_USERNAME: str
    DB_PASSWORD: str
    DB_HOST: str
    DB_PORT: int
    DB_DATABASE: str

    @property
    def DATABASE_URL(self) -> str:
        return f"postgresql+asyncpg://{self.DB_USERNAME}:{self.DB_PASSWORD}@{self.DB_HOST}:{self.DB_PORT}/{self.DB_DATABASE}"

    # LLM Provider: "openai" or "azure"
    LLM_PROVIDER: Literal["openai", "azure"] = "openai"

    # OpenAI (default, easier for most users)
    OPENAI_API_KEY: Optional[str] = None
    OPENAI_BASE_URL: Optional[str] = None  # For custom endpoints / proxies
    OPENAI_MODEL: str = "gpt-4o-mini"

    # Azure OpenAI (optional)
    AZURE_OPENAI_API_KEY: Optional[str] = None
    AZURE_OPENAI_ENDPOINT: Optional[str] = None
    AZURE_OPENAI_API_VERSION: str = "2024-02-15-preview"
    AZURE_DEPLOYMENT_NAME: str = "gpt-4o"

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
        env_prefix="NEKO_"
    )

@lru_cache
def get_settings():
    return Settings()

settings = get_settings()
