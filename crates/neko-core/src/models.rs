use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Example {
    pub sentence: String,
    pub translation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Word {
    pub id: String,
    pub word: String,
    pub language: String,
    pub translation: String,
    pub examples: Vec<Example>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Review {
    pub word_id: String,
    pub interval: i64,
    pub ease_factor: f64,
    pub streak: i64,
    pub next_review_at: DateTime<Utc>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub history: Vec<ReviewHistoryEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReviewHistoryEntry {
    pub date: DateTime<Utc>,
    pub grade: Grade,
    pub interval: i64,
    pub ease: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DueReview {
    pub word: Word,
    pub review: Review,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AddWordResult {
    pub word: Word,
    pub duplicate: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl std::str::FromStr for Grade {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "again" => Ok(Self::Again),
            "hard" => Ok(Self::Hard),
            "good" => Ok(Self::Good),
            "easy" => Ok(Self::Easy),
            other => anyhow::bail!("unknown review grade: {other}"),
        }
    }
}

impl Grade {
    pub fn quality(self) -> i64 {
        match self {
            Self::Again => 0,
            Self::Hard => 2,
            Self::Good => 4,
            Self::Easy => 5,
        }
    }
}
