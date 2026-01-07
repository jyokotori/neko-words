from fastapi import APIRouter, Depends, HTTPException
from sqlmodel import select
from sqlmodel.ext.asyncio.session import AsyncSession
from typing import List, Any
from datetime import datetime, timezone, timedelta
from pydantic import BaseModel

from app.core.db import get_session
from app.models.review import Review
from app.models.word import Word

router = APIRouter()

@router.get("/due", response_model=List[dict])
async def get_due_reviews(
    limit: int = 50, 
    language: str = "en", 
    session: AsyncSession = Depends(get_session)
):
    now = datetime.utcnow()
    statement = (
        select(Review, Word)
        .join(Word)
        .where(Word.language == language)
        .where(Review.next_review_at <= now)
        .order_by(Review.streak.asc(), Review.ease_factor.asc(), Review.interval.asc())
        .limit(limit)
    )
    results = await session.exec(statement)
    
    output = []
    for review, word in results:
        output.append({
            "review": review,
            "word": word
        })
    return output

class ReviewLog(BaseModel):
    grade: str # again, hard, good, easy

@router.post("/{word_id}/log")
async def log_review(
    word_id: str, 
    log: ReviewLog, 
    session: AsyncSession = Depends(get_session)
):
    review = await session.get(Review, word_id)
    if not review:
        raise HTTPException(status_code=404, detail="Review not found")
    
    now = datetime.utcnow()
    review.last_reviewed_at = now
    
    # Grading Logic
    quality = 3
    if log.grade == "again": quality = 0
    elif log.grade == "hard": quality = 2
    elif log.grade == "good": quality = 4
    elif log.grade == "easy": quality = 5
    
    if quality < 3:
        review.streak = 0
        review.interval = 1
        review.next_review_at = now + timedelta(minutes=1)
    else:
        if review.streak == 0:
            review.interval = 1
        elif review.streak == 1:
            review.interval = 6
        else:
            review.interval = int(review.interval * review.ease_factor)
        
        review.streak += 1
        review.ease_factor = review.ease_factor + (0.1 - (5 - quality) * (0.08 + (5 - quality) * 0.02))
        if review.ease_factor < 1.3: review.ease_factor = 1.3
        
        review.next_review_at = now + timedelta(days=review.interval)

    # History Update
    history_entry = {
        "date": now.isoformat(),
        "grade": log.grade,
        "interval": review.interval,
        "ease": review.ease_factor
    }
    
    # Ensure list copy
    current_history = list(review.history) if review.history else []
    current_history.append(history_entry)
    review.history = current_history
    
    session.add(review)
    await session.commit()
    await session.refresh(review)
    return {"status": "ok", "next_review": review.next_review_at}

@router.post("/{word_id}/undo")
async def undo_review(
    word_id: str,
    session: AsyncSession = Depends(get_session)
):
    """Undo the last review action for a word."""
    review = await session.get(Review, word_id)
    if not review:
        raise HTTPException(status_code=404, detail="Review not found")
    
    if not review.history or len(review.history) == 0:
        raise HTTPException(status_code=400, detail="No review history to undo")
    
    # Pop the last history entry
    current_history = list(review.history)
    popped = current_history.pop()
    review.history = current_history
    
    # Restore previous state
    if len(current_history) > 0:
        # Restore from the previous entry
        prev = current_history[-1]
        review.interval = prev.get("interval", 0)
        review.ease_factor = prev.get("ease", 2.5)
        review.last_reviewed_at = datetime.fromisoformat(prev.get("date")) if prev.get("date") else None
        # Recalculate streak based on history
        review.streak = len([h for h in current_history if h.get("grade") not in ["again", "hard"]])
    else:
        # Reset to initial state
        review.interval = 0
        review.ease_factor = 2.5
        review.streak = 0
        review.last_reviewed_at = None
    
    # Set next_review_at to now so the card appears again immediately
    review.next_review_at = datetime.utcnow()
    
    session.add(review)
    await session.commit()
    await session.refresh(review)
    
    return {"status": "ok", "undone_grade": popped.get("grade")}
