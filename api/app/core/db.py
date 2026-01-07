from sqlalchemy.ext.asyncio import create_async_engine
from sqlmodel.ext.asyncio.session import AsyncSession
from sqlalchemy.orm import sessionmaker
from .config import settings
from loguru import logger
from typing import AsyncGenerator

engine = create_async_engine(settings.DATABASE_URL, echo=False, future=True)

async def get_session() -> AsyncGenerator[AsyncSession, None]:
    async with AsyncSession(engine) as session:
        logger.debug("DB Session created")
        yield session

async def init_db():
    from sqlmodel import SQLModel
    # Import models to ensure they are registered
    from ..models import word, review
    async with engine.begin() as conn:
        await conn.run_sync(SQLModel.metadata.create_all)
