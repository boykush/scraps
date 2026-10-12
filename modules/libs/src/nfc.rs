//! Names (scrap keys, tags, types) are compared in Unicode NFC, so a name
//! matches however the filesystem or the editor composed it.

use std::borrow::Cow;

use unicode_normalization::{IsNormalized, UnicodeNormalization, is_nfc_quick};

/// `s` in NFC, borrowed when it already is: the build passes every name on
/// every page through here, and nearly all of them are.
pub fn nfc(s: &str) -> Cow<'_, str> {
    if s.is_ascii() || is_nfc_quick(s.chars()) == IsNormalized::Yes {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.nfc().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::kana("\u{30ab}\u{3099}\u{30a4}\u{30c8}\u{3099}", "\u{30ac}\u{30a4}\u{30c9}")]
    #[case::latin("cafe\u{301}", "caf\u{e9}")]
    #[case::hangul("\u{1112}\u{1161}\u{11ab}", "\u{d55c}")]
    fn it_composes(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(nfc(input), expected);
    }

    #[rstest]
    #[case::ascii("Test driven development")]
    #[case::composed("\u{30ac}\u{30a4}\u{30c9} caf\u{e9} \u{d55c}")]
    #[case::empty("")]
    fn it_borrows_what_is_already_nfc(#[case] input: &str) {
        assert!(matches!(nfc(input), Cow::Borrowed(s) if s == input));
    }

    #[rstest]
    #[case::full_width("\u{ff21}")]
    #[case::half_width_kana("\u{ff76}\u{ff9e}")]
    #[case::case("A")]
    fn it_leaves_compatibility_forms_and_case(#[case] input: &str) {
        assert_eq!(nfc(input), input);
    }
}
