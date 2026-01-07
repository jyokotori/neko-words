from fastapi import APIRouter, Depends, HTTPException, Body
from datetime import datetime
from sqlmodel import select
from sqlmodel.ext.asyncio.session import AsyncSession
from typing import Any
from app.core.db import get_session
from app.models.word import Word, WordBase
from app.models.review import Review
from app.services.llm import enrich_word
from pydantic import BaseModel

router = APIRouter()

class WordInput(BaseModel):
    word: str
    language: str = "en"

from loguru import logger

@router.post("/", response_model=Word)
async def add_word(
    input: WordInput, 
    session: AsyncSession = Depends(get_session)
):
    # Normalize input
    input.word = input.word.strip().lower()
    
    logger.info(f"Received add_word request for: {input.word}")
    
    # Process with LLM first
    try:
        data = await enrich_word(input.word, input.language)
        base_word_text = data.get("word", input.word).strip().lower()
        
        # Check if the base word already exists
        statement = select(Word).where(Word.word == base_word_text).where(Word.language == input.language)
        results = await session.exec(statement)
        existing_word = results.first()
        
        if existing_word:
            logger.info(f"Word '{base_word_text}' already exists. Resetting review status (Forgotten).")
            # Fetch associated review
            review_stmt = select(Review).where(Review.word_id == existing_word.id)
            review_results = await session.exec(review_stmt)
            review = review_results.first()
            
            if review:
                review.streak = 0
                review.interval = 0
                review.next_review_at = datetime.utcnow()
                review.ease_factor = max(1.3, review.ease_factor - 0.2) # Optional: penalize ease factor slightly
                session.add(review)
                await session.commit()
                await session.refresh(existing_word)
                return existing_word
            else:
                # Should not happen if data integrity is maintained, but handle gracefully
                # If review doesn't exist for some reason, create it
                review = Review(word_id=existing_word.id)
                session.add(review)
                await session.commit()
                await session.refresh(existing_word)
                return existing_word

        # If not exists, create new
        new_word = Word(
            word=base_word_text,
            language=input.language,
            translation=data["translation"],
            examples=data["examples"]
        )
        session.add(new_word)
        await session.commit()
        await session.refresh(new_word)
        
        # Init review
        review = Review(word_id=new_word.id)
        session.add(review)
        await session.commit()
        
        # Refresh to ensure object is not expired before returning
        await session.refresh(new_word)
        return new_word
        
    except Exception as e:
        logger.error(f"Error adding word {input.word}: {e}")
        raise HTTPException(status_code=500, detail=str(e))
