from datetime import datetime
import uuid
from typing import Optional, List, Dict, Any
from sqlmodel import SQLModel, Field, Relationship
from sqlalchemy import Column, UniqueConstraint
from sqlalchemy.dialects.postgresql import JSONB

class WordBase(SQLModel):
    word: str = Field(index=True)
    language: str = Field(default="en", index=True)
    translation: str
    examples: List[Dict[str, str]] = Field(default_factory=list, sa_column=Column(JSONB))

class Word(WordBase, table=True):
    __tablename__ = "words"
    __table_args__ = (
        UniqueConstraint("language", "word", name="uq_words_language_word"),
    )
    id: uuid.UUID = Field(default_factory=uuid.uuid4, primary_key=True)
    created_at: datetime = Field(default_factory=datetime.utcnow)
    
    # Relationships
    reviews: List["Review"] = Relationship(back_populates="word")
