from contextlib import asynccontextmanager
from fastapi import FastAPI
from .core.config import settings
from .core.logging import setup_logging
from .core.db import init_db
from .api.v1.api import api_router
from loguru import logger

@asynccontextmanager
async def lifespan(app: FastAPI):
    setup_logging()
    logger.info("Starting up application...")
    await init_db()
    yield
    logger.info("Shutting down application...")

from fastapi.middleware.cors import CORSMiddleware

app = FastAPI(
    title=settings.PROJECT_NAME,
    openapi_url=f"{settings.API_V1_STR}/openapi.json",
    lifespan=lifespan
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=False,  # 使用 * 时不能为 True
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(api_router, prefix=settings.API_V1_STR)

@app.get("/")
async def root():
    return {"message": "Welcome to Neko Words API", "docs": "/docs"}
