use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::models::{DueReview, Example, ExportData, Grade, Review, Word};

mod sqlite;

pub use sqlite::SqliteRepository;

#[async_trait]
pub trait WordRepository: Send + Sync {
    async fn migrate(&self) -> Result<()>;
    async fn find_word(&self, word: &str, tag: &str) -> Result<Option<Word>>;
    /// Atomically inserts a word and its initial review, returning `None` when
    /// another writer already inserted the same `(tag, word)` pair.
    async fn insert_word_with_review(
        &self,
        word: &str,
        tag: &str,
        translation: &str,
        examples: &[Example],
    ) -> Result<Option<Word>>;
    async fn reset_review_for_word(&self, word_id: &str) -> Result<Word>;
    async fn due_reviews(&self, tag: &str, limit: i64) -> Result<Vec<DueReview>>;
    async fn log_review(&self, word_id: &str, grade: Grade) -> Result<Review>;
    async fn undo_review(&self, word_id: &str) -> Result<Grade>;
    /// Dump all words and reviews into the schema-agnostic [`ExportData`] format.
    async fn export_all(&self) -> Result<ExportData>;
    /// Upsert all words and reviews from [`ExportData`]. Words missing a review
    /// row get a freshly-initialized review so they show up in `due_reviews`.
    async fn import_all(&self, data: &ExportData) -> Result<()>;
}

/// Parse an RFC3339 timestamp string into a UTC datetime (used by the SQLite
/// backend, which stores timestamps as TEXT).
pub(crate) fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

pub(crate) fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.as_deref().map(parse_datetime).transpose()
}
