// Rule-based importance boost helpers for the enrichment pipeline

const DEADLINE_KEYWORDS: &[&str] = &[
    "today", "tomorrow", "urgent", "asap", "deadline", "due",
];

/// +0.2 if text contains any deadline/urgency keyword (case-insensitive).
fn has_deadline_keyword(text: &str) -> bool {
    let lower = text.to_lowercase();
    DEADLINE_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// +0.1 if text contains two or more consecutive capitalised words (person name heuristic).
fn has_person_name(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }
    for pair in words.windows(2) {
        let a = pair[0].trim_matches(|c: char| !c.is_alphabetic());
        let b = pair[1].trim_matches(|c: char| !c.is_alphabetic());
        if !a.is_empty() && !b.is_empty() {
            let a_cap = a.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            let b_cap = b.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            if a_cap && b_cap {
                return true;
            }
        }
    }
    false
}

/// +0.1 if text contains an http:// or https:// URL.
fn has_url(text: &str) -> bool {
    text.contains("http://") || text.contains("https://")
}

/// +0.05 if text is longer than `threshold` characters.
fn is_long_content(text: &str, threshold: usize) -> bool {
    text.len() > threshold
}

/// +0.05 if text has more than `min_sentences` sentence-ending punctuation marks.
fn has_multiple_sentences(text: &str, min_sentences: usize) -> bool {
    let count = text.chars().filter(|&c| c == '.' || c == '!' || c == '?').count();
    count >= min_sentences
}

/// Applies all rule-based boosts to an LLM-provided importance score.
/// Returns the final score capped at 1.0.
pub fn apply_importance_boosts(llm_score: f64, text: &str) -> f64 {
    let mut boost = 0.0_f64;

    if has_deadline_keyword(text) { boost += 0.2; }
    if has_person_name(text)      { boost += 0.1; }
    if has_url(text)              { boost += 0.1; }
    if is_long_content(text, 500) { boost += 0.05; }
    if has_multiple_sentences(text, 3) { boost += 0.05; }

    (llm_score + boost).min(1.0)
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_keyword_detected() {
        assert!(has_deadline_keyword("This is urgent, due today!"));
        assert!(!has_deadline_keyword("Just a regular note."));
    }

    #[test]
    fn person_name_detected() {
        assert!(has_person_name("Met John Smith at the conference."));
        assert!(!has_person_name("met someone at the conference."));
    }

    #[test]
    fn url_detected() {
        assert!(has_url("See https://example.com for details."));
        assert!(!has_url("No link here."));
    }

    #[test]
    fn boost_capped_at_one() {
        // All boosts fire: 0.5 + 0.2 + 0.1 + 0.1 + 0.05 + 0.05 = 1.0
        let text = "Urgent deadline today! Meet John Smith at https://example.com. \
                    This is a very long note with multiple sentences. It has three. \
                    And a fourth sentence here to be safe.";
        let result = apply_importance_boosts(0.5, text);
        assert!(result <= 1.0);
    }
}
