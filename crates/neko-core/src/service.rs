use anyhow::{Context, Result};

use crate::{
    llm::{EnrichmentOutcome, WordEnricher},
    models::{AddWordResult, Grade},
    repository::WordRepository,
};

pub async fn add_word<R, E>(
    repo: &R,
    enricher: &E,
    raw_word: &str,
    tag: &str,
) -> Result<AddWordResult>
where
    R: WordRepository,
    E: WordEnricher,
{
    let input_word = normalize_word(raw_word);
    if let Some(existing) = repo.find_word(&input_word, tag).await? {
        let word = repo.reset_review_for_word(&existing.id).await?;
        return Ok(AddWordResult::Duplicate { word });
    }

    let (base_word, translation, examples) = match enricher.enrich_word(&input_word).await? {
        EnrichmentOutcome::Valid {
            word,
            translation,
            examples,
        } => (normalize_word(&word), translation, examples),
        EnrichmentOutcome::Invalid { reason } => {
            return Ok(AddWordResult::Invalid {
                input: input_word,
                reason,
            });
        }
        EnrichmentOutcome::Skip { reason } => {
            return Ok(AddWordResult::Skipped {
                input: input_word,
                reason,
            });
        }
    };
    if let Some(existing) = repo.find_word(&base_word, tag).await? {
        let word = repo.reset_review_for_word(&existing.id).await?;
        return Ok(AddWordResult::Duplicate { word });
    }

    let inserted = repo
        .insert_word_with_review(&base_word, tag, &translation, &examples)
        .await?;
    if let Some(word) = inserted {
        return Ok(AddWordResult::Added { word });
    }

    let existing = repo
        .find_word(&base_word, tag)
        .await?
        .context("word conflicted during insert but was not found")?;
    let word = repo.reset_review_for_word(&existing.id).await?;
    Ok(AddWordResult::Duplicate { word })
}

pub async fn log_review<R: WordRepository>(repo: &R, word_id: &str, grade: Grade) -> Result<()> {
    repo.log_review(word_id, grade).await?;
    Ok(())
}

fn normalize_word(word: &str) -> String {
    word.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        llm::{EnrichmentOutcome, WordEnricher},
        models::Example,
        repository::SqliteRepository,
    };
    use anyhow::Result;
    use async_trait::async_trait;

    struct FakeEnricher;

    #[async_trait]
    impl WordEnricher for FakeEnricher {
        async fn enrich_word(&self, word: &str) -> Result<EnrichmentOutcome> {
            if word == "asdfgh" {
                return Ok(EnrichmentOutcome::Invalid {
                    reason: "not a recognizable word or phrase".to_string(),
                });
            }
            if word.len() >= 3
                && word
                    .chars()
                    .all(|character| matches!(character, '-' | '*' | '_' | '`'))
            {
                return Ok(EnrichmentOutcome::Skip {
                    reason: "formatting separator".to_string(),
                });
            }
            let lemma = match word {
                "children" => "child",
                "#cat" | "# cat" => "cat",
                "## go to" => "go to",
                "- apple" => "apple",
                _ => word,
            };
            Ok(EnrichmentOutcome::Valid {
                word: lemma.to_string(),
                translation: "/x/ 测试".to_string(),
                examples: vec![Example {
                    sentence: format!("{lemma} example"),
                    translation: "例句".to_string(),
                }],
            })
        }
    }

    #[tokio::test]
    async fn duplicate_raw_word_is_not_inserted_again() {
        let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();

        let first = add_word(&repo, &FakeEnricher, "Test", "en").await.unwrap();
        let second = add_word(&repo, &FakeEnricher, "test", "en").await.unwrap();

        let AddWordResult::Added { word: first } = first else {
            panic!("expected an added word");
        };
        let AddWordResult::Duplicate { word: second } = second else {
            panic!("expected a duplicate word");
        };
        assert_eq!(first.id, second.id);
    }

    #[tokio::test]
    async fn duplicate_after_llm_lemma_is_not_inserted_again() {
        let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();

        let first = add_word(&repo, &FakeEnricher, "child", "en").await.unwrap();
        let second = add_word(&repo, &FakeEnricher, "children", "en")
            .await
            .unwrap();

        let AddWordResult::Added { word: first } = first else {
            panic!("expected an added word");
        };
        let AddWordResult::Duplicate { word: second } = second else {
            panic!("expected a duplicate word");
        };
        assert_eq!(first.id, second.id);
    }

    #[tokio::test]
    async fn invalid_input_is_not_inserted() {
        let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();

        let result = add_word(&repo, &FakeEnricher, "asdfgh", "en")
            .await
            .unwrap();

        assert!(matches!(result, AddWordResult::Invalid { .. }));
        assert!(repo.find_word("asdfgh", "en").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn pure_formatting_is_skipped_without_being_inserted() {
        let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();

        let result = add_word(&repo, &FakeEnricher, "```", "en").await.unwrap();

        assert!(matches!(result, AddWordResult::Skipped { .. }));
        assert!(repo.find_word("```", "en").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn formatting_prefixes_do_not_hide_valid_vocabulary() {
        for (input, expected) in [
            ("#cat", "cat"),
            ("# cat", "cat"),
            ("## go to", "go to"),
            ("- apple", "apple"),
        ] {
            let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
            repo.migrate().await.unwrap();
            let result = add_word(&repo, &FakeEnricher, input, "en").await.unwrap();

            let AddWordResult::Added { word } = result else {
                panic!("expected an added word for {input}");
            };
            assert_eq!(word.word, expected);
        }
    }
}
