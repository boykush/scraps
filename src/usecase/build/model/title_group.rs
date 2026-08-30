/// Which section of the title index a scrap belongs to, derived from the
/// first character of its title. Kanji get their own bucket because reading
/// them needs a dictionary, which the lean core deliberately does not carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TitleGroup {
    /// A gojuon row, labelled by its leading kana (あ か さ た な は ま や ら わ).
    Kana(char),
    /// An ASCII letter, uppercased.
    Latin(char),
    Kanji,
    Other,
}

impl TitleGroup {
    pub fn from_title(title: &str) -> TitleGroup {
        match title.chars().next() {
            Some(c) => Self::from_char(c),
            None => TitleGroup::Other,
        }
    }

    fn from_char(c: char) -> TitleGroup {
        let c = fold_width(c);
        let c = katakana_to_hiragana(c);
        if c.is_ascii_alphabetic() {
            return TitleGroup::Latin(c.to_ascii_uppercase());
        }
        if let Some(row) = gojuon_row(c) {
            return TitleGroup::Kana(row);
        }
        if is_cjk_ideograph(c) {
            return TitleGroup::Kanji;
        }
        TitleGroup::Other
    }

    pub fn label(&self) -> String {
        match self {
            TitleGroup::Kana(row) => row.to_string(),
            TitleGroup::Latin(letter) => letter.to_string(),
            TitleGroup::Kanji => "漢字".to_string(),
            TitleGroup::Other => "#".to_string(),
        }
    }
}

fn fold_width(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => {
            char::from_u32(c as u32 - 0xFF01 + 0x21).expect("offset stays in ASCII")
        }
        _ => c,
    }
}

fn katakana_to_hiragana(c: char) -> char {
    match c {
        '\u{30A1}'..='\u{30F6}' => {
            char::from_u32(c as u32 - 0x60).expect("offset stays in hiragana block")
        }
        _ => c,
    }
}

/// Rows fold voiced/semi-voiced and small kana onto their base row, so for
/// example デ lands on た行. The ranges follow Unicode hiragana ordering.
fn gojuon_row(c: char) -> Option<char> {
    match c {
        'ぁ'..='お' | 'ゔ' => Some('あ'),
        'か'..='ご' | 'ゕ' | 'ゖ' => Some('か'),
        'さ'..='ぞ' => Some('さ'),
        'た'..='ど' => Some('た'),
        'な'..='の' => Some('な'),
        'は'..='ぽ' => Some('は'),
        'ま'..='も' => Some('ま'),
        'ゃ'..='よ' => Some('や'),
        'ら'..='ろ' => Some('ら'),
        'ゎ'..='ん' => Some('わ'),
        _ => None,
    }
}

fn is_cjk_ideograph(c: char) -> bool {
    matches!(
        c,
        '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hiragana_maps_to_its_row() {
        assert_eq!(TitleGroup::from_title("あいうえお"), TitleGroup::Kana('あ'));
        assert_eq!(TitleGroup::from_title("すくらっぷ"), TitleGroup::Kana('さ'));
        assert_eq!(TitleGroup::from_title("わたし"), TitleGroup::Kana('わ'));
        assert_eq!(TitleGroup::from_title("んじゃめな"), TitleGroup::Kana('わ'));
    }

    #[test]
    fn voiced_and_small_kana_fold_onto_the_base_row() {
        assert_eq!(TitleGroup::from_title("がぎぐ"), TitleGroup::Kana('か'));
        assert_eq!(TitleGroup::from_title("ぱぴぷ"), TitleGroup::Kana('は'));
        assert_eq!(TitleGroup::from_title("っち"), TitleGroup::Kana('た'));
        assert_eq!(TitleGroup::from_title("ゃゅょ"), TitleGroup::Kana('や'));
    }

    #[test]
    fn katakana_folds_to_hiragana_rows() {
        assert_eq!(
            TitleGroup::from_title("デザイントークン"),
            TitleGroup::Kana('た')
        );
        assert_eq!(TitleGroup::from_title("クラウド"), TitleGroup::Kana('か'));
        assert_eq!(
            TitleGroup::from_title("ヴァイオリン"),
            TitleGroup::Kana('あ')
        );
    }

    #[test]
    fn latin_uppercases_and_folds_width() {
        assert_eq!(TitleGroup::from_title("scraps"), TitleGroup::Latin('S'));
        assert_eq!(TitleGroup::from_title("DTCG"), TitleGroup::Latin('D'));
        assert_eq!(
            TitleGroup::from_title("Ｋubernetes"),
            TitleGroup::Latin('K')
        );
    }

    #[test]
    fn kanji_gets_its_own_bucket() {
        assert_eq!(TitleGroup::from_title("良いコード"), TitleGroup::Kanji);
        assert_eq!(TitleGroup::from_title("設計"), TitleGroup::Kanji);
    }

    #[test]
    fn digits_symbols_and_empty_fall_back_to_other() {
        assert_eq!(TitleGroup::from_title("12 factor"), TitleGroup::Other);
        assert_eq!(TitleGroup::from_title("６つの帽子"), TitleGroup::Other);
        assert_eq!(TitleGroup::from_title("ーん"), TitleGroup::Other);
        assert_eq!(TitleGroup::from_title(""), TitleGroup::Other);
    }

    #[test]
    fn groups_order_kana_then_latin_then_kanji_then_other() {
        let mut groups = vec![
            TitleGroup::Other,
            TitleGroup::Latin('B'),
            TitleGroup::Kanji,
            TitleGroup::Kana('わ'),
            TitleGroup::Latin('A'),
            TitleGroup::Kana('あ'),
        ];
        groups.sort();
        assert_eq!(
            groups,
            vec![
                TitleGroup::Kana('あ'),
                TitleGroup::Kana('わ'),
                TitleGroup::Latin('A'),
                TitleGroup::Latin('B'),
                TitleGroup::Kanji,
                TitleGroup::Other,
            ]
        );
    }

    #[test]
    fn labels() {
        assert_eq!(TitleGroup::Kana('た').label(), "た");
        assert_eq!(TitleGroup::Latin('D').label(), "D");
        assert_eq!(TitleGroup::Kanji.label(), "漢字");
        assert_eq!(TitleGroup::Other.label(), "#");
    }
}
