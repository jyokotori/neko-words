from datetime import datetime
import uuid
from typing import Optional, List, Dict, Any
from sqlmodel import SQLModel, Field, Relationship
from sqlalchemy import Column
from sqlalchemy.dialects.postgresql import JSONB

class ReviewBase(SQLModel):
    interval: int = 0
    ease_factor: float = 2.5
    streak: int = 0
    next_review_at: datetime = Field(default_factory=datetime.utcnow)
    last_reviewed_at: Optional[datetime] = None
    history: List[Dict[str, Any]] = Field(default_factory=list, sa_column=Column(JSONB))

class Review(ReviewBase, table=True):
    __tablename__ = "reviews"
    word_id: uuid.UUID = Field(foreign_key="words.id", primary_key=True)
    
    word: "Word" = Relationship(back_populates="reviews")

from .word import Word
