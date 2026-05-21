//! Maps `rustc` error codes to the Foundations lesson that teaches the relevant
//! concept (prompt §5). The map is forward-looking — it returns lesson ids for
//! lessons not yet authored; the Sprint-4 renderer only links ones that exist.

/// The lesson id that teaches the concept behind `code`, if known.
pub fn concept_for_code(code: &str) -> Option<&'static str> {
    Some(match code {
        // Ownership / move semantics.
        "E0382" => "foundations-03-ownership-moves",
        // Borrow checker (aliasing, mutable+shared, use-after-move-while-borrowed).
        "E0499" | "E0502" | "E0505" | "E0506" => "foundations-04-borrows",
        // Lifetimes (informal in Foundations) live with borrows.
        "E0106" | "E0621" => "foundations-04-borrows",
        // Type mismatch — relates to the variables/types lesson.
        "E0308" => "foundations-02-variables",
        // Unresolved name / not found in scope — the very first lesson.
        "E0425" | "E0433" => "foundations-01-hello",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_for_e0382() {
        assert_eq!(
            concept_for_code("E0382"),
            Some("foundations-03-ownership-moves")
        );
    }

    #[test]
    fn test_concept_for_borrow_codes() {
        assert_eq!(concept_for_code("E0502"), Some("foundations-04-borrows"));
        assert_eq!(concept_for_code("E0106"), Some("foundations-04-borrows"));
    }

    #[test]
    fn test_concept_for_unknown() {
        assert_eq!(concept_for_code("E9999"), None);
    }
}
