import sys
from loguru import logger
from .config import settings

def setup_logging():
    logger.remove()
    logger.add(
        sys.stderr,
        format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
        level="INFO",
    )
    logger.add(
        "logs/neko_words.log",
        rotation="10 MB",
        retention="10 days",
        level="DEBUG",
        compression="zip"
    )
    logger.info(f"Logging initialized for {settings.PROJECT_NAME}")
