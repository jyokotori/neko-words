use anyhow::Result;

use crate::{
    llm::WordEnricher,
    models::{AddWordResult, Grade},
    repository::WordRepository,
};

pub async fn add_word<R, E>(
    repo: &R,
    enricher: &E,
    raw_word: &str,
    language: &str,
) -> Result<AddWordResult>
where
    R: WordRepository,
    E: WordEnricher,
{
    let input_word = normalize_word(raw_word);
    if let Some(existing) = repo.find_word(&input_word, language).await? {
        let word = repo.reset_review_for_word(&existing.id).await?;
        return Ok(AddWordResult {
            word,
            duplicate: true,
        });
    }

    let enriched = enricher.enrich_word(&input_word, language).await?;
    let base_word = normalize_word(&enriched.word);
    if let Some(existing) = repo.find_word(&base_word, language).await? {
        let word = repo.reset_review_for_word(&existing.id).await?;
        return Ok(AddWordResult {
            word,
            duplicate: true,
        });
    }

    let inserted = repo
        .insert_word_with_review(
            &base_word,
            language,
            &enriched.translation,
            &enriched.examples,
        )
        .await?;
    Ok(AddWordResult {
        word: inserted,
        duplicate: false,
    })
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
        llm::{EnrichedWord, WordEnricher},
        models::Example,
        repository::SqliteRepository,
    };
    use anyhow::Result;
    use async_trait::async_trait;

    struct FakeEnricher;

    #[async_trait]
    impl WordEnricher for FakeEnricher {
        async fn enrich_word(&self, word: &str, _language: &str) -> Result<EnrichedWord> {
            let lemma = if word == "children" { "child" } else { word };
            Ok(EnrichedWord {
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

        assert!(!first.duplicate);
        assert!(second.duplicate);
        assert_eq!(first.word.id, second.word.id);
    }

    #[tokio::test]
    async fn duplicate_after_llm_lemma_is_not_inserted_again() {
        let repo = SqliteRepository::connect("sqlite::memory:").await.unwrap();
        repo.migrate().await.unwrap();

        let first = add_word(&repo, &FakeEnricher, "child", "en").await.unwrap();
        let second = add_word(&repo, &FakeEnricher, "children", "en")
            .await
            .unwrap();

        assert!(!first.duplicate);
        assert!(second.duplicate);
        assert_eq!(first.word.id, second.word.id);
    }
}
