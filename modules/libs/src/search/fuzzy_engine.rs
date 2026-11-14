use std::cmp::Reverse;

use super::engine::{SearchEngine, SearchLogic};
use super::result::{SearchField, SearchItem, fold};

/// How well a keyword sits in one field, best first. A keyword that only
/// appears as letters scattered over the text is not a match at all.
const WHOLE_FIELD: i64 = 16;
const WHOLE_WORD: i64 = 8;
const WORD_START: i64 = 6;
const INSIDE_WORD: i64 = 4;
const NEAR_MISS: i64 = 1;

/// A near miss may skip one letter of a word per this many keyword letters,
/// so short keywords, where a skipped letter is a different word, match as written.
const LETTERS_PER_SKIP: usize = 4;

#[derive(Default)]
pub struct FuzzySearchEngine;

impl FuzzySearchEngine {
    pub fn new() -> Self {
        Self
    }
}

impl SearchEngine for FuzzySearchEngine {
    fn search(
        &self,
        items: &[SearchItem],
        query: &str,
        num: usize,
        logic: SearchLogic,
    ) -> Vec<SearchItem> {
        let query = fold(query);
        let keywords: Vec<&str> = query.split_whitespace().collect();
        if keywords.is_empty() {
            return items.iter().take(num).cloned().collect();
        }

        let mut ranked: Vec<(&SearchItem, usize, i64)> = items
            .iter()
            .filter_map(|item| {
                let scores: Vec<i64> = keywords
                    .iter()
                    .filter_map(|kw| keyword_score(item, kw))
                    .collect();
                let enough = match logic {
                    SearchLogic::And => scores.len() == keywords.len(),
                    SearchLogic::Or => !scores.is_empty(),
                };
                enough.then(|| (item, scores.len(), scores.iter().sum()))
            })
            .collect();

        // Keywords matched come first: one strong match is a weaker cue than several.
        ranked.sort_by_key(|(_, matched, score)| (Reverse(*matched), Reverse(*score)));

        ranked
            .into_iter()
            .take(num)
            .map(|(item, _, _)| item.clone())
            .collect()
    }
}

/// Each field is matched on its own, so no match spans two fields.
fn keyword_score(item: &SearchItem, keyword: &str) -> Option<i64> {
    item.fields
        .iter()
        .filter_map(|SearchField { text, weight }| Some(match_quality(text, keyword)? * weight))
        .max()
}

fn match_quality(text: &str, keyword: &str) -> Option<i64> {
    if text == keyword {
        return Some(WHOLE_FIELD);
    }
    let written = text
        .match_indices(keyword)
        .map(|(start, _)| {
            let starts_word = !ends_in_letter(&text[..start]);
            let ends_word = !starts_with_letter(&text[start + keyword.len()..]);
            match (starts_word, ends_word) {
                (true, true) => WHOLE_WORD,
                (true, false) => WORD_START,
                _ => INSIDE_WORD,
            }
        })
        .max();
    written.or_else(|| {
        let keyword: Vec<char> = keyword.chars().collect();
        text.split(|c: char| !c.is_alphanumeric())
            .any(|word| is_near_miss(word, &keyword))
            .then_some(NEAR_MISS)
    })
}

fn ends_in_letter(text: &str) -> bool {
    text.chars().next_back().is_some_and(char::is_alphanumeric)
}

fn starts_with_letter(text: &str) -> bool {
    text.chars().next().is_some_and(char::is_alphanumeric)
}

/// The keyword's letters appear in order within a stretch of the word barely
/// longer than the keyword: a dropped letter, not a coincidence.
fn is_near_miss(word: &str, keyword: &[char]) -> bool {
    let skips = keyword.len() / LETTERS_PER_SKIP;
    if skips == 0 {
        return false;
    }
    let word: Vec<char> = word.chars().collect();
    (0..word.len()).any(|start| {
        let window = word[start..].iter().take(keyword.len() + skips);
        let mut rest = keyword.iter().peekable();
        for c in window {
            if rest.peek() == Some(&c) {
                rest.next();
            }
        }
        rest.peek().is_none()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn engine() -> FuzzySearchEngine {
        FuzzySearchEngine::new()
    }

    // ===========================================
    // OR logic tests (default)
    // ===========================================

    #[rstest]
    fn test_or_matches_any_keyword(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, None, &[]),
            SearchItem::new("Python Guide", None, None, &[]),
            SearchItem::new("Rust and Python", None, None, &[]),
        ];
        let results = engine.search(&items, "rust python", 100, SearchLogic::Or);

        assert_eq!(results.len(), 3);
    }

    #[rstest]
    fn test_or_partial_match(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Documentation", None, None, &[]),
            SearchItem::new("Sample", None, None, &[]),
        ];
        let results = engine.search(&items, "doc", 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Documentation");
    }

    #[rstest]
    fn test_or_no_match(engine: FuzzySearchEngine) {
        let items = vec![SearchItem::new("Test", None, None, &[])];
        let results = engine.search(&items, "xyzzyx", 100, SearchLogic::Or);

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_or_searches_the_description(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document A", None, Some("contains uniquekeyword here"), &[]),
            SearchItem::new("Document B", None, Some("no match"), &[]),
        ];
        let results = engine.search(&items, "uniquekeyword", 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Document A");
    }

    #[rstest]
    fn test_or_empty_query_returns_all(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("A", None, None, &[]),
            SearchItem::new("B", None, None, &[]),
        ];
        let results = engine.search(&items, "", 100, SearchLogic::Or);

        assert_eq!(results.len(), 2);
    }

    #[rstest]
    fn test_or_result_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Item {}", i), None, Some("keyword"), &[]))
            .collect();
        let results = engine.search(&items, "keyword", 5, SearchLogic::Or);

        assert_eq!(results.len(), 5);
    }

    // ===========================================
    // AND logic tests
    // ===========================================

    #[rstest]
    fn test_and_requires_all_keywords(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, None, &[]),
            SearchItem::new("Python Guide", None, None, &[]),
            SearchItem::new("Rust and Python", None, None, &[]),
        ];
        let results = engine.search(&items, "rust python", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Rust and Python");
    }

    #[rstest]
    fn test_and_exact_match(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Test Document", None, None, &[]),
            SearchItem::new("Another Test", None, None, &[]),
            SearchItem::new("Document Only", None, None, &[]),
        ];
        let results = engine.search(&items, "Test Document", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Test Document");
    }

    #[rstest]
    #[case::composed_query("caf\u{e9}", "cafe\u{301} menu")]
    #[case::decomposed_query("cafe\u{301}", "caf\u{e9} menu")]
    fn test_matches_across_normalization_forms(
        engine: FuzzySearchEngine,
        #[case] query: &str,
        #[case] body: &str,
    ) {
        let items = vec![
            SearchItem::new("note", None, Some(body), &[]),
            SearchItem::new("other", None, None, &[]),
        ];
        let results = engine.search(&items, query, 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "note");
    }

    #[rstest]
    fn test_and_keyword_order_independent(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Programming", None, None, &[]),
            SearchItem::new("Programming Rust", None, None, &[]),
            SearchItem::new("Python Language", None, None, &[]),
        ];
        let results = engine.search(&items, "Programming Rust", 100, SearchLogic::And);

        assert_eq!(results.len(), 2);
    }

    #[rstest]
    fn test_and_non_consecutive_keywords(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust is great for Programming", None, None, &[]),
            SearchItem::new("Python Language", None, None, &[]),
        ];
        let results = engine.search(&items, "Rust Programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
    }

    #[rstest]
    fn test_and_no_match_missing_keyword(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, None, &[]),
            SearchItem::new("Python Guide", None, None, &[]),
        ];
        let results = engine.search(&items, "rust java", 100, SearchLogic::And);

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_and_searches_the_description(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document", None, Some("rust programming"), &[]),
            SearchItem::new("Another", None, Some("rust only"), &[]),
        ];
        let results = engine.search(&items, "rust programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Document");
    }

    #[rstest]
    fn test_and_result_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Test Item {}", i), None, None, &[]))
            .collect();
        let results = engine.search(&items, "Test Item", 5, SearchLogic::And);

        assert_eq!(results.len(), 5);
    }

    // ===========================================
    // Ranking and noise tests
    // ===========================================

    fn keys(results: &[SearchItem]) -> Vec<&str> {
        results.iter().map(|r| r.key.as_str()).collect()
    }

    #[rstest]
    #[case::latin("scraps", "Security champions rapid response")]
    #[case::latin_across_a_long_key("wiki", "OWASP Security Champions/Promote knowledge sharing")]
    #[case::cjk("記憶", "記録と追憶")]
    fn test_scattered_letters_do_not_match(
        engine: FuzzySearchEngine,
        #[case] query: &str,
        #[case] key: &str,
    ) {
        let items = vec![SearchItem::new(key, None, None, &[])];

        assert!(
            engine
                .search(&items, query, 100, SearchLogic::Or)
                .is_empty()
        );
    }

    #[rstest]
    fn test_a_dropped_letter_still_matches_a_long_keyword(engine: FuzzySearchEngine) {
        let items = vec![SearchItem::new("Kubernetes", None, None, &[])];

        let results = engine.search(&items, "kubernets", 100, SearchLogic::Or);

        assert_eq!(keys(&results), ["Kubernetes"]);
    }

    #[rstest]
    #[case::across_words("abeta", SearchItem::new("alpha beta", None, None, &[]))]
    #[case::across_fields("erss", SearchItem::new("orders", None, Some("Sales"), &[]))]
    fn test_a_match_does_not_span_words_or_fields(
        engine: FuzzySearchEngine,
        #[case] query: &str,
        #[case] item: SearchItem,
    ) {
        assert!(
            engine
                .search(&[item], query, 100, SearchLogic::Or)
                .is_empty()
        );
    }

    #[rstest]
    fn test_tighter_matches_rank_higher(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Trusted sources", None, None, &[]),
            SearchItem::new("Rustacean", None, None, &[]),
            SearchItem::new("Rust Guide", None, None, &[]),
            SearchItem::new("Rust", None, None, &[]),
        ];

        let results = engine.search(&items, "rust", 100, SearchLogic::Or);

        assert_eq!(
            keys(&results),
            ["Rust", "Rust Guide", "Rustacean", "Trusted sources"]
        );
    }

    #[rstest]
    fn test_a_name_outranks_a_tag_and_a_tag_a_description(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("a", None, Some("Notes on rust tooling"), &[]),
            SearchItem::new("b", None, None, &["Rust tooling".to_string()]),
            SearchItem::new("c", Some("Rust tooling"), None, &[]),
        ];

        let results = engine.search(&items, "rust", 100, SearchLogic::Or);

        assert_eq!(keys(&results), ["c", "b", "a"]);
    }

    #[rstest]
    fn test_or_ranks_by_how_many_keywords_matched(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Documentation", None, None, &[]),
            SearchItem::new("notes", None, Some("db and ui"), &[]),
        ];

        let results = engine.search(&items, "documentation db ui", 100, SearchLogic::Or);

        assert_eq!(keys(&results), ["notes", "Documentation"]);
    }

    #[rstest]
    #[case::uppercase_query("Rust", "rust guide")]
    #[case::uppercase_text("rust", "RUST GUIDE")]
    #[case::acronym("PKM", "pkm notes")]
    fn test_case_is_ignored(engine: FuzzySearchEngine, #[case] query: &str, #[case] key: &str) {
        let items = vec![SearchItem::new(key, None, None, &[])];

        assert_eq!(engine.search(&items, query, 100, SearchLogic::Or).len(), 1);
    }

    #[rstest]
    fn test_cjk_matches_a_contiguous_run(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("間に隔たりがあり反復して復習", None, None, &[]),
            SearchItem::new("学習法", None, Some("間隔反復で覚える"), &[]),
        ];

        let results = engine.search(&items, "間隔反復", 100, SearchLogic::Or);

        assert_eq!(keys(&results), ["学習法"]);
    }

    #[rstest]
    fn test_relevant_scraps_outrank_unrelated_ones(engine: FuzzySearchEngine) {
        let owasp = |name: &str| {
            SearchItem::new(
                &format!("OWASP Security Champions/{name}"),
                None,
                Some("A principle of a security champions programme."),
                &["Security".to_string()],
            )
        };
        let items = vec![
            owasp("Promote knowledge sharing"),
            owasp("Reward responsibility"),
            owasp("Invest in your champions"),
            owasp("Trust your champions"),
            SearchItem::new("PKM", None, None, &[]),
            SearchItem::new("Spaced repetition", None, Some("間隔反復で記憶を保つ"), &[]),
            SearchItem::new("LLM Wiki", None, Some("An LLM-maintained wiki"), &[]),
            SearchItem::new("Scraps", None, None, &["PKM".to_string()]),
        ];

        let results = engine.search(
            &items,
            "Scraps LLM Wiki 記憶 想起 間隔反復 PKM 検索",
            100,
            SearchLogic::Or,
        );

        assert_eq!(
            keys(&results),
            ["Scraps", "LLM Wiki", "Spaced repetition", "PKM"]
        );
    }

    #[rstest]
    #[case(SearchLogic::Or)]
    #[case(SearchLogic::And)]
    fn test_a_blank_query_is_an_empty_query(engine: FuzzySearchEngine, #[case] logic: SearchLogic) {
        let items = vec![
            SearchItem::new("A", None, None, &[]),
            SearchItem::new("B", None, None, &[]),
        ];

        assert_eq!(engine.search(&items, "  ", 100, logic).len(), 2);
    }

    // ===========================================
    // Common behavior tests
    // ===========================================

    #[rstest]
    fn test_single_keyword_same_for_or_and_and(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, None, &[]),
            SearchItem::new("Python Guide", None, None, &[]),
        ];
        let or_results = engine.search(&items, "rust", 100, SearchLogic::Or);
        let and_results = engine.search(&items, "rust", 100, SearchLogic::And);

        assert_eq!(or_results.len(), and_results.len());
    }

    #[rstest]
    fn test_default_logic_is_or(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, None, &[]),
            SearchItem::new("Python Guide", None, None, &[]),
            SearchItem::new("Rust and Python", None, None, &[]),
        ];
        let default_results = engine.search(&items, "rust python", 100, SearchLogic::default());
        let or_results = engine.search(&items, "rust python", 100, SearchLogic::Or);

        assert_eq!(default_results.len(), or_results.len());
        assert_eq!(default_results.len(), 3);
    }

    #[rstest]
    fn test_empty_items_returns_empty(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = vec![];
        let or_results = engine.search(&items, "test", 100, SearchLogic::Or);
        let and_results = engine.search(&items, "test", 100, SearchLogic::And);

        assert_eq!(or_results.len(), 0);
        assert_eq!(and_results.len(), 0);
    }
}
