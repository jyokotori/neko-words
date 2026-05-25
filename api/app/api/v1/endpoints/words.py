from fastapi import APIRouter, Depends, HTTPException
from datetime import datetime
from sqlmodel import select
from sqlmodel.ext.asyncio.session import AsyncSession
from sqlalchemy.exc import IntegrityError
from app.core.db import get_session
from app.models.word import Word
from app.models.review import Review
from app.services.llm import enrich_word
from pydantic import BaseModel
from loguru import logger

router = APIRouter()


class WordInput(BaseModel):
    word: str
    language: str = "en"


class AddWordResponse(BaseModel):
    word: Word
    duplicate: bool = False


async def _find_word(session: AsyncSession, text: str, language: str) -> Word | None:
    statement = select(Word).where(Word.word == text).where(Word.language == language)
    results = await session.exec(statement)
    return results.first()


async def _reset_review(session: AsyncSession, word: Word) -> Word:
    review_stmt = select(Review).where(Review.word_id == word.id)
    review_results = await session.exec(review_stmt)
    review = review_results.first()

    if review is None:
        review = Review(word_id=word.id)
    else:
        review.streak = 0
        review.interval = 0
        review.next_review_at = datetime.utcnow()
        review.ease_factor = max(1.3, review.ease_factor - 0.2)

    session.add(review)
    await session.commit()
    await session.refresh(word)
    return word


@router.post("/", response_model=AddWordResponse)
async def add_word(
    input: WordInput,
    session: AsyncSession = Depends(get_session),
):
    input.word = input.word.strip().lower()
    logger.info(f"Received add_word request for: {input.word}")

    # Stage 1: pre-LLM duplicate check on raw input
    existing = await _find_word(session, input.word, input.language)
    if existing:
        logger.info(f"[pre-LLM] '{input.word}' already exists; skipping LLM, resetting review.")
        word = await _reset_review(session, existing)
        return AddWordResponse(word=word, duplicate=True)

    # Enrich with LLM
    try:
        data = await enrich_word(input.word, input.language)
    except Exception as e:
        logger.error(f"LLM enrich failed for {input.word}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

    base_word_text = data.get("word", input.word).strip().lower()

    # Stage 2: post-LLM duplicate check on base form
    existing = await _find_word(session, base_word_text, input.language)
    if existing:
        logger.info(f"[post-LLM] '{base_word_text}' exists; resetting review.")
        word = await _reset_review(session, existing)
        return AddWordResponse(word=word, duplicate=True)

    new_word = Word(
        word=base_word_text,
        language=input.language,
        translation=data["translation"],
        examples=data["examples"],
    )
    session.add(new_word)
    try:
        await session.flush()
        session.add(Review(word_id=new_word.id))
        await session.commit()
    except IntegrityError:
        # Another concurrent request inserted the same (language, word) — treat as duplicate.
        await session.rollback()
        logger.info(f"[post-LLM] IntegrityError on insert for '{base_word_text}'; treating as duplicate.")
        existing = await _find_word(session, base_word_text, input.language)
        if existing is None:
            raise HTTPException(status_code=500, detail="Insert conflict but row not found")
        word = await _reset_review(session, existing)
        return AddWordResponse(word=word, duplicate=True)

    await session.refresh(new_word)

    return AddWordResponse(word=new_word, duplicate=False)
