from sqlalchemy.ext.asyncio import create_async_engine
from sqlmodel.ext.asyncio.session import AsyncSession
from sqlalchemy.orm import sessionmaker
from .config import settings
from loguru import logger
from typing import AsyncGenerator

# Enable pre_ping and pool recycling to avoid stale connections from long-lived or idle sessions
engine = create_async_engine(
    settings.DATABASE_URL,
    echo=False,
    future=True,
    pool_pre_ping=True,
    pool_recycle=1800,  # recycle connections every 30 minutes
)

async_session_factory = sessionmaker(
    bind=engine,
    class_=AsyncSession,
    expire_on_commit=False,
)

async def get_session() -> AsyncGenerator[AsyncSession, None]:
    async with async_session_factory() as session:
        logger.debug("DB Session created")
        yield session

async def init_db():
    from sqlmodel import SQLModel
    # Import models to ensure they are registered
    from ..models import word, review
    async with engine.begin() as conn:
        await conn.run_sync(SQLModel.metadata.create_all)
