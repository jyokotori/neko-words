from fastapi import APIRouter
from .endpoints import words, reviews

api_router = APIRouter()
api_router.include_router(words.router, prefix="/words", tags=["words"])
api_router.include_router(reviews.router, prefix="/reviews", tags=["reviews"])
