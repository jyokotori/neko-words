from pydantic_settings import BaseSettings, SettingsConfigDict
from functools import lru_cache

class Settings(BaseSettings):
    API_BASE_URL: str = "http://localhost:8002/api/v1"
    DEFAULT_LANGUAGE: str = "en"
    
    model_config = SettingsConfigDict(
        env_file=".env", 
        env_file_encoding="utf-8", 
        extra="ignore"
    )

@lru_cache
def get_settings():
    return Settings()

settings = get_settings()
