//! `rusty-scheduler` — the SM-2-lite spaced-repetition scheduler for Rusty.
//!
//! Portability contract: this crate must stay OS-portable. No clock or filesystem
//! coupling baked in — time is passed in by the caller so the scheduler stays a
//! pure function of its inputs.

use serde::{Deserialize, Serialize};

/// State of a single concept in the spaced repetition system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewState {
    /// The SM-2 ease factor (defaults to 2.5).
    pub ease: f32,
    /// The current interval in lessons.
    pub interval_lessons: u32,
    /// The lesson index at which this review is next due.
    pub due_at_lesson: u64,
}

impl Default for ReviewState {
    fn default() -> Self {
        Self {
            ease: 2.5,
            interval_lessons: 0,
            due_at_lesson: 0, // 0 implies it is immediately due
        }
    }
}

/// Applies the SM-2 algorithm based on the user's quality of response.
///
/// `quality` is clamped between 0 and 5.
/// - 0: Complete blackout.
/// - 1: Incorrect response; the correct one remembered.
/// - 2: Incorrect response; where the correct one seemed easy to recall.
/// - 3: Correct response recalled with serious difficulty.
/// - 4: Correct response after a hesitation.
/// - 5: Perfect response.
///
/// `current_lesson_index` is the number of lessons completed by the learner.
pub fn grade_review(state: &ReviewState, quality: u8, current_lesson_index: u64) -> ReviewState {
    let quality = quality.clamp(0, 5);

    let mut ease = state.ease;
    let mut interval_lessons = state.interval_lessons;

    if quality < 3 {
        // User failed to recall the concept.
        interval_lessons = 1;
        // Ease is not decreased significantly on failure according to strict SM-2,
        // but typically it is. Here we'll stick to standard SM-2 ease logic.
    } else {
        // User recalled the concept successfully.
        if interval_lessons == 0 {
            interval_lessons = 1;
        } else if interval_lessons == 1 {
            interval_lessons = 6;
        } else {
            interval_lessons = (interval_lessons as f32 * ease).round() as u32;
        }
    }

    // Update ease factor
    let q = quality as f32;
    ease += 0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02);
    if ease < 1.3 {
        ease = 1.3;
    }

    let due_at_lesson = current_lesson_index + (interval_lessons as u64);

    ReviewState {
        ease,
        interval_lessons,
        due_at_lesson,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_review_perfect() {
        let state = ReviewState::default();
        let next = grade_review(&state, 5, 1);
        assert_eq!(next.interval_lessons, 1);
        assert_eq!(next.ease, 2.6); // 2.5 + (0.1 - 0) = 2.6
        assert_eq!(next.due_at_lesson, 2);
    }

    #[test]
    fn test_second_review_perfect() {
        let state = ReviewState {
            ease: 2.6,
            interval_lessons: 1,
            due_at_lesson: 2,
        };
        let next = grade_review(&state, 5, 2);
        assert_eq!(next.interval_lessons, 6);
        assert!((next.ease - 2.7).abs() < 0.001);
        assert_eq!(next.due_at_lesson, 8);
    }

    #[test]
    fn test_failure_resets_interval() {
        let state = ReviewState {
            ease: 2.7,
            interval_lessons: 6,
            due_at_lesson: 8,
        };
        // 0 quality means blackout
        let next = grade_review(&state, 0, 8);
        assert_eq!(next.interval_lessons, 1);
        // Ease should decrease by 0.8
        // ease + (0.1 - (5 - 0)*(0.08 + 5*0.02)) = ease + (0.1 - 5*0.18) = ease + (0.1 - 0.9) = ease - 0.8
        assert!((next.ease - 1.9).abs() < 0.001);
        assert_eq!(next.due_at_lesson, 9);
    }

    #[test]
    fn test_ease_floor() {
        let state = ReviewState {
            ease: 1.3, // minimum ease
            interval_lessons: 10,
            due_at_lesson: 12,
        };
        let next = grade_review(&state, 0, 12);
        assert_eq!(next.interval_lessons, 1);
        assert_eq!(next.ease, 1.3); // shouldn't go below 1.3
    }
}
