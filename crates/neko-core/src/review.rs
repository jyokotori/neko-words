use chrono::{Duration, Utc};

use crate::models::{Grade, Review, ReviewHistoryEntry};

pub fn initial_review(word_id: String) -> Review {
    Review {
        word_id,
        interval: 0,
        ease_factor: 2.5,
        streak: 0,
        next_review_at: Utc::now(),
        last_reviewed_at: None,
        history: Vec::new(),
    }
}

pub fn reset_review(review: &mut Review) {
    review.streak = 0;
    review.interval = 0;
    review.next_review_at = Utc::now();
    review.ease_factor = (review.ease_factor - 0.2).max(1.3);
}

pub fn apply_grade(review: &mut Review, grade: Grade) {
    let now = Utc::now();
    let quality = grade.quality();
    review.last_reviewed_at = Some(now);

    if quality < 3 {
        review.streak = 0;
        review.interval = 1;
        review.next_review_at = now + Duration::minutes(1);
    } else {
        review.interval = if review.streak == 0 {
            1
        } else if review.streak == 1 {
            6
        } else {
            (review.interval as f64 * review.ease_factor) as i64
        };
        review.streak += 1;

        let delta = 0.1 - (5 - quality) as f64 * (0.08 + (5 - quality) as f64 * 0.02);
        review.ease_factor = (review.ease_factor + delta).max(1.3);
        review.next_review_at = now + Duration::days(review.interval);
    }

    review.history.push(ReviewHistoryEntry {
        date: now,
        grade,
        interval: review.interval,
        ease: review.ease_factor,
    });
}

pub fn undo_last(review: &mut Review) -> Option<Grade> {
    let popped = review.history.pop()?;
    if let Some(prev) = review.history.last() {
        review.interval = prev.interval;
        review.ease_factor = prev.ease;
        review.last_reviewed_at = Some(prev.date);
        review.streak = review
            .history
            .iter()
            .filter(|entry| !matches!(entry.grade, Grade::Again | Grade::Hard))
            .count() as i64;
    } else {
        review.interval = 0;
        review.ease_factor = 2.5;
        review.streak = 0;
        review.last_reviewed_at = None;
    }
    review.next_review_at = Utc::now();
    Some(popped.grade)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_reviews_advance_streak_and_interval() {
        let mut review = initial_review("w1".to_string());

        apply_grade(&mut review, Grade::Good);
        assert_eq!(review.streak, 1);
        assert_eq!(review.interval, 1);
        assert!(review.next_review_at > Utc::now());

        apply_grade(&mut review, Grade::Easy);
        assert_eq!(review.streak, 2);
        assert_eq!(review.interval, 6);
        assert!(review.ease_factor > 2.5);
    }

    #[test]
    fn again_resets_to_short_retry() {
        let mut review = initial_review("w1".to_string());
        apply_grade(&mut review, Grade::Good);
        apply_grade(&mut review, Grade::Again);

        assert_eq!(review.streak, 0);
        assert_eq!(review.interval, 1);
        assert_eq!(review.history.len(), 2);
    }

    #[test]
    fn undo_restores_initial_state_after_one_log() {
        let mut review = initial_review("w1".to_string());
        apply_grade(&mut review, Grade::Good);

        let undone = undo_last(&mut review);

        assert_eq!(undone, Some(Grade::Good));
        assert_eq!(review.interval, 0);
        assert_eq!(review.ease_factor, 2.5);
        assert_eq!(review.streak, 0);
        assert!(review.history.is_empty());
    }
}
