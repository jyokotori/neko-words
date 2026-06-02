use std::sync::Once;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{any::AnyPoolOptions, AnyPool, Row};
use uuid::Uuid;

use crate::{
    models::{DueReview, Example, Grade, Review, Word},
    review::{apply_grade, initial_review, reset_review, undo_last},
};

#[async_trait]
pub trait WordRepository: Send + Sync {
    async fn migrate(&self) -> Result<()>;
    async fn find_word(&self, word: &str, language: &str) -> Result<Option<Word>>;
    async fn insert_word_with_review(
        &self,
        word: &str,
        language: &str,
        translation: &str,
        examples: &[Example],
    ) -> Result<Word>;
    async fn reset_review_for_word(&self, word_id: &str) -> Result<Word>;
    async fn due_reviews(&self, language: &str, limit: i64) -> Result<Vec<DueReview>>;
    async fn log_review(&self, word_id: &str, grade: Grade) -> Result<Review>;
    async fn undo_review(&self, word_id: &str) -> Result<Grade>;
}

#[derive(Clone)]
pub struct SqlxRepository {
    pool: AnyPool,
}

impl SqlxRepository {
    pub async fn connect(database_url: &str) -> Result<Self> {
        install_any_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .with_context(|| format!("failed to connect database: {database_url}"))?;
        Ok(Self { pool })
    }
}

fn install_any_drivers() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(sqlx::any::install_default_drivers);
}

#[async_trait]
impl WordRepository for SqlxRepository {
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS words (
                id TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                language TEXT NOT NULL,
                translation TEXT NOT NULL,
                examples TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(language, word)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reviews (
                word_id TEXT PRIMARY KEY,
                interval INTEGER NOT NULL,
                ease_factor REAL NOT NULL,
                streak INTEGER NOT NULL,
                next_review_at TEXT NOT NULL,
                last_reviewed_at TEXT,
                history TEXT NOT NULL,
                FOREIGN KEY(word_id) REFERENCES words(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_word(&self, word: &str, language: &str) -> Result<Option<Word>> {
        let row = sqlx::query("SELECT * FROM words WHERE word = ? AND language = ?")
            .bind(word)
            .bind(language)
            .fetch_optional(&self.pool)
            .await?;
        row.map(word_from_row).transpose()
    }

    async fn insert_word_with_review(
        &self,
        word: &str,
        language: &str,
        translation: &str,
        examples: &[Example],
    ) -> Result<Word> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let examples_json = serde_json::to_string(examples)?;
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO words (id, word, language, translation, examples, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(word)
        .bind(language)
        .bind(translation)
        .bind(examples_json)
        .bind(created_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        let review = initial_review(id.clone());
        upsert_review_query(&review).execute(&mut *tx).await?;
        tx.commit().await?;

        self.find_word(word, language)
            .await?
            .context("inserted word was not found")
    }

    async fn reset_review_for_word(&self, word_id: &str) -> Result<Word> {
        let mut review = self
            .get_review(word_id)
            .await?
            .unwrap_or_else(|| initial_review(word_id.to_string()));
        reset_review(&mut review);
        upsert_review_query(&review).execute(&self.pool).await?;
        self.get_word_by_id(word_id)
            .await?
            .context("word not found while resetting review")
    }

    async fn due_reviews(&self, language: &str, limit: i64) -> Result<Vec<DueReview>> {
        let rows = sqlx::query(
            r#"
            SELECT
                w.id AS w_id, w.word AS w_word, w.language AS w_language,
                w.translation AS w_translation, w.examples AS w_examples, w.created_at AS w_created_at,
                r.word_id AS r_word_id, r.interval AS r_interval, r.ease_factor AS r_ease_factor,
                r.streak AS r_streak, r.next_review_at AS r_next_review_at,
                r.last_reviewed_at AS r_last_reviewed_at, r.history AS r_history
            FROM reviews r
            JOIN words w ON w.id = r.word_id
            WHERE w.language = ? AND r.next_review_at <= ?
            ORDER BY r.streak ASC, r.ease_factor ASC, r.interval ASC
            LIMIT ?
            "#,
        )
        .bind(language)
        .bind(Utc::now().to_rfc3339())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(DueReview {
                    word: Word {
                        id: row.try_get("w_id")?,
                        word: row.try_get("w_word")?,
                        language: row.try_get("w_language")?,
                        translation: row.try_get("w_translation")?,
                        examples: serde_json::from_str(&row.try_get::<String, _>("w_examples")?)?,
                        created_at: parse_datetime(&row.try_get::<String, _>("w_created_at")?)?,
                    },
                    review: Review {
                        word_id: row.try_get("r_word_id")?,
                        interval: row.try_get("r_interval")?,
                        ease_factor: row.try_get("r_ease_factor")?,
                        streak: row.try_get("r_streak")?,
                        next_review_at: parse_datetime(
                            &row.try_get::<String, _>("r_next_review_at")?,
                        )?,
                        last_reviewed_at: parse_optional_datetime(
                            row.try_get("r_last_reviewed_at")?,
                        )?,
                        history: serde_json::from_str(&row.try_get::<String, _>("r_history")?)?,
                    },
                })
            })
            .collect()
    }

    async fn log_review(&self, word_id: &str, grade: Grade) -> Result<Review> {
        let mut review = self
            .get_review(word_id)
            .await?
            .context("review not found")?;
        apply_grade(&mut review, grade);
        upsert_review_query(&review).execute(&self.pool).await?;
        Ok(review)
    }

    async fn undo_review(&self, word_id: &str) -> Result<Grade> {
        let mut review = self
            .get_review(word_id)
            .await?
            .context("review not found")?;
        let grade = undo_last(&mut review).context("no review history to undo")?;
        upsert_review_query(&review).execute(&self.pool).await?;
        Ok(grade)
    }
}

impl SqlxRepository {
    async fn get_word_by_id(&self, word_id: &str) -> Result<Option<Word>> {
        let row = sqlx::query("SELECT * FROM words WHERE id = ?")
            .bind(word_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(word_from_row).transpose()
    }

    async fn get_review(&self, word_id: &str) -> Result<Option<Review>> {
        let row = sqlx::query("SELECT * FROM reviews WHERE word_id = ?")
            .bind(word_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(review_from_row).transpose()
    }
}

fn upsert_review_query(
    review: &Review,
) -> sqlx::query::Query<'_, sqlx::Any, sqlx::any::AnyArguments<'_>> {
    sqlx::query(
        r#"
        INSERT INTO reviews
            (word_id, interval, ease_factor, streak, next_review_at, last_reviewed_at, history)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(word_id) DO UPDATE SET
            interval = excluded.interval,
            ease_factor = excluded.ease_factor,
            streak = excluded.streak,
            next_review_at = excluded.next_review_at,
            last_reviewed_at = excluded.last_reviewed_at,
            history = excluded.history
        "#,
    )
    .bind(&review.word_id)
    .bind(review.interval)
    .bind(review.ease_factor)
    .bind(review.streak)
    .bind(review.next_review_at.to_rfc3339())
    .bind(review.last_reviewed_at.map(|value| value.to_rfc3339()))
    .bind(serde_json::to_string(&review.history).expect("review history serializes"))
}

fn word_from_row(row: sqlx::any::AnyRow) -> Result<Word> {
    Ok(Word {
        id: row.try_get("id")?,
        word: row.try_get("word")?,
        language: row.try_get("language")?,
        translation: row.try_get("translation")?,
        examples: serde_json::from_str(&row.try_get::<String, _>("examples")?)?,
        created_at: parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    })
}

fn review_from_row(row: sqlx::any::AnyRow) -> Result<Review> {
    Ok(Review {
        word_id: row.try_get("word_id")?,
        interval: row.try_get("interval")?,
        ease_factor: row.try_get("ease_factor")?,
        streak: row.try_get("streak")?,
        next_review_at: parse_datetime(&row.try_get::<String, _>("next_review_at")?)?,
        last_reviewed_at: parse_optional_datetime(row.try_get("last_reviewed_at")?)?,
        history: serde_json::from_str(&row.try_get::<String, _>("history")?)?,
    })
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.as_deref().map(parse_datetime).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlite_repository_can_add_review_and_undo() {
        let repo = SqlxRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();
        let word = repo
            .insert_word_with_review(
                "test",
                "en",
                "/test/ 测试",
                &[Example {
                    sentence: "This is a test.".to_string(),
                    translation: "这是一个测试。".to_string(),
                }],
            )
            .await
            .unwrap();

        let due = repo.due_reviews("en", 10).await.unwrap();
        assert_eq!(due.len(), 1);
        repo.log_review(&word.id, Grade::Good).await.unwrap();
        let undone = repo.undo_review(&word.id).await.unwrap();
        assert_eq!(undone, Grade::Good);
    }
}
