use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use sqlx::sqlite::{
    SqliteArguments, SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow,
};
use uuid::Uuid;

use super::{WordRepository, parse_datetime, parse_optional_datetime};
use crate::{
    models::{DueReview, Example, ExportData, Grade, Review, Word},
    review::{apply_grade, initial_review, reset_review, undo_last},
};

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)
            .with_context(|| format!("invalid sqlite url: {database_url}"))?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .with_context(|| format!("failed to connect database: {database_url}"))?;
        Ok(Self { pool })
    }

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

#[async_trait]
impl WordRepository for SqliteRepository {
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS words (
                id TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                tag TEXT NOT NULL,
                translation TEXT NOT NULL,
                examples TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(tag, word)
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

    async fn find_word(&self, word: &str, tag: &str) -> Result<Option<Word>> {
        let row = sqlx::query("SELECT * FROM words WHERE word = ? AND tag = ?")
            .bind(word)
            .bind(tag)
            .fetch_optional(&self.pool)
            .await?;
        row.map(word_from_row).transpose()
    }

    async fn insert_word_with_review(
        &self,
        word: &str,
        tag: &str,
        translation: &str,
        examples: &[Example],
    ) -> Result<Word> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let examples_json = serde_json::to_string(examples)?;
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO words (id, word, tag, translation, examples, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(word)
        .bind(tag)
        .bind(translation)
        .bind(examples_json)
        .bind(created_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        let review = initial_review(id.clone());
        upsert_review_query(&review).execute(&mut *tx).await?;
        tx.commit().await?;

        self.find_word(word, tag)
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

    async fn due_reviews(&self, tag: &str, limit: i64) -> Result<Vec<DueReview>> {
        let rows = sqlx::query(
            r#"
            SELECT
                w.id AS w_id, w.word AS w_word, w.tag AS w_tag,
                w.translation AS w_translation, w.examples AS w_examples, w.created_at AS w_created_at,
                r.word_id AS r_word_id, r.interval AS r_interval, r.ease_factor AS r_ease_factor,
                r.streak AS r_streak, r.next_review_at AS r_next_review_at,
                r.last_reviewed_at AS r_last_reviewed_at, r.history AS r_history
            FROM reviews r
            JOIN words w ON w.id = r.word_id
            WHERE w.tag = ? AND r.next_review_at <= ?
            ORDER BY r.streak ASC, r.ease_factor ASC, r.interval ASC
            LIMIT ?
            "#,
        )
        .bind(tag)
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
                        tag: row.try_get("w_tag")?,
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

    async fn export_all(&self) -> Result<ExportData> {
        let words = sqlx::query("SELECT * FROM words ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(word_from_row)
            .collect::<Result<Vec<_>>>()?;
        let reviews = sqlx::query("SELECT * FROM reviews")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(review_from_row)
            .collect::<Result<Vec<_>>>()?;
        Ok(ExportData {
            version: 1,
            words,
            reviews,
        })
    }

    async fn import_all(&self, data: &ExportData) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for word in &data.words {
            sqlx::query(
                r#"
                INSERT INTO words (id, word, tag, translation, examples, created_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    word = excluded.word,
                    tag = excluded.tag,
                    translation = excluded.translation,
                    examples = excluded.examples,
                    created_at = excluded.created_at
                "#,
            )
            .bind(&word.id)
            .bind(&word.word)
            .bind(&word.tag)
            .bind(&word.translation)
            .bind(serde_json::to_string(&word.examples)?)
            .bind(word.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }
        let with_review: HashSet<&str> = data.reviews.iter().map(|r| r.word_id.as_str()).collect();
        for review in &data.reviews {
            upsert_review_query(review).execute(&mut *tx).await?;
        }
        for word in &data.words {
            if !with_review.contains(word.id.as_str()) {
                let review = initial_review(word.id.clone());
                upsert_review_query(&review).execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}

fn upsert_review_query(
    review: &Review,
) -> sqlx::query::Query<'static, sqlx::Sqlite, SqliteArguments<'static>> {
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
    .bind(review.word_id.clone())
    .bind(review.interval)
    .bind(review.ease_factor)
    .bind(review.streak)
    .bind(review.next_review_at.to_rfc3339())
    .bind(review.last_reviewed_at.map(|value| value.to_rfc3339()))
    .bind(serde_json::to_string(&review.history).expect("review history serializes"))
}

fn word_from_row(row: SqliteRow) -> Result<Word> {
    Ok(Word {
        id: row.try_get("id")?,
        word: row.try_get("word")?,
        tag: row.try_get("tag")?,
        translation: row.try_get("translation")?,
        examples: serde_json::from_str(&row.try_get::<String, _>("examples")?)?,
        created_at: parse_datetime(&row.try_get::<String, _>("created_at")?)?,
    })
}

fn review_from_row(row: SqliteRow) -> Result<Review> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlite_repository_can_add_review_and_undo() {
        let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();
        let word = repo
            .insert_word_with_review(
                "test",
                "en",
                "/x/ 测试",
                &[Example {
                    sentence: "test sentence".to_string(),
                    translation: "测试句子".to_string(),
                }],
            )
            .await
            .unwrap();

        repo.log_review(&word.id, Grade::Good).await.unwrap();
        let undone = repo.undo_review(&word.id).await.unwrap();
        assert_eq!(undone, Grade::Good);
    }

    #[tokio::test]
    async fn export_then_import_round_trips() {
        let source = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        source.migrate().await.unwrap();
        source
            .insert_word_with_review(
                "alpha",
                "en",
                "/a/ 甲",
                &[Example {
                    sentence: "alpha sentence".to_string(),
                    translation: "甲句".to_string(),
                }],
            )
            .await
            .unwrap();
        let dump = source.export_all().await.unwrap();
        assert_eq!(dump.words.len(), 1);
        assert_eq!(dump.reviews.len(), 1);

        let target = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        target.migrate().await.unwrap();
        target.import_all(&dump).await.unwrap();
        let restored = target.export_all().await.unwrap();
        assert_eq!(restored.words, dump.words);
        assert_eq!(restored.reviews, dump.reviews);
    }

    #[tokio::test]
    async fn import_words_without_reviews_synthesizes_initial_review() {
        let source = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        source.migrate().await.unwrap();
        let word = source
            .insert_word_with_review("beta", "en", "/b/ 乙", &[])
            .await
            .unwrap();

        // Words-only payload (mirrors the legacy backup that lacked reviews).
        let dump = ExportData {
            version: 1,
            words: vec![word.clone()],
            reviews: vec![],
        };

        let target = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        target.migrate().await.unwrap();
        target.import_all(&dump).await.unwrap();

        let due = target.due_reviews("en", 10).await.unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].word.id, word.id);
    }
}
