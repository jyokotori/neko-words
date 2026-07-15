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
    pub tag: String,
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

/// Schema-agnostic intermediate format for manual migration between backends
/// (SQLite local mode <-> Postgres server mode). Serialized to/from JSON.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExportData {
    pub version: u32,
    pub words: Vec<Word>,
    pub reviews: Vec<Review>,
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
            "1" | "again" => Ok(Self::Again),
            "2" | "hard" => Ok(Self::Hard),
            "3" | "good" => Ok(Self::Good),
            "4" | "easy" => Ok(Self::Easy),
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

#[cfg(test)]
mod tests {
    use super::Grade;

    #[test]
    fn grade_accepts_numeric_shortcuts() {
        assert_eq!("1".parse::<Grade>().unwrap(), Grade::Again);
        assert_eq!("2".parse::<Grade>().unwrap(), Grade::Hard);
        assert_eq!("3".parse::<Grade>().unwrap(), Grade::Good);
        assert_eq!("4".parse::<Grade>().unwrap(), Grade::Easy);
    }
}
