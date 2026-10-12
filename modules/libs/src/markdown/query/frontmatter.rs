use std::borrow::Cow;

use comrak::{Arena, nodes::NodeValue, parse_document};
use serde_json::{Map, Number, Value};
use yaml_rust2::yaml::{Yaml, YamlLoader};

use super::common::options;

const DELIMITER: &str = "---";
const BOM: char = '\u{feff}';

/// The YAML frontmatter block of a scrap, as opaque structured data.
///
/// No key is given a meaning here — this is passthrough, so the caller sees
/// exactly what the author wrote. A block that is absent, empty, unclosed
/// or malformed reads as `None`, so a retrofit wiki still builds.
pub fn frontmatter(text: &str) -> Option<Value> {
    block(text).map(|block| block.value)
}

/// The text after the frontmatter block and the blank lines that set it
/// apart: what a scrap says, as opposed to what is declared about it. Text
/// without a recognised block is returned whole, so a block `frontmatter`
/// does not read stays content.
pub fn body(text: &str) -> &str {
    block(text).map_or(text, |block| skip_blank_lines(&text[block.len..]))
}

fn skip_blank_lines(mut text: &str) -> &str {
    while let Some(end) = text.find('\n') {
        if !text[..end].trim().is_empty() {
            break;
        }
        text = &text[end + 1..];
    }
    text
}

/// `text` with the frontmatter block blanked out rather than cut, so the
/// lines and byte offsets a parse reports are still those of the file.
pub(super) fn mask(text: &str) -> Cow<'_, str> {
    read(text).1
}

/// The block's data and the masked text, from one recognition.
pub(super) fn read(text: &str) -> (Option<Value>, Cow<'_, str>) {
    match block(text) {
        Some(block) => (Some(block.value), Cow::Owned(blank(text, block.len))),
        None => (None, Cow::Borrowed(text)),
    }
}

struct Block {
    value: Value,
    /// Bytes the block takes at the start of the text, delimiters included.
    len: usize,
}

fn block(text: &str) -> Option<Block> {
    // Most scraps have no block; this spares them a parse per query.
    let unmarked = text.strip_prefix(BOM).unwrap_or(text);
    if !unmarked.starts_with(DELIMITER) {
        return None;
    }
    let arena = Arena::new();
    let mut opts = options();
    opts.extension.front_matter_delimiter = Some(DELIMITER.to_string());
    let root = parse_document(&arena, text, &opts);

    let raw = root
        .descendants()
        .find_map(|node| match &node.data().value {
            NodeValue::FrontMatter(raw) => Some(raw.clone()),
            _ => None,
        })?;

    let docs = YamlLoader::load_from_str(strip_delimiters(&raw)?).ok()?;
    let value = to_json(docs.into_iter().next()?)?;
    // comrak hands the block back verbatim, minus a leading BOM.
    let len = text.len() - unmarked.len() + raw.len();
    text.is_char_boundary(len).then_some(Block { value, len })
}

fn blank(text: &str, len: usize) -> String {
    let (block, body) = text.split_at(len);
    let mut out = String::with_capacity(text.len());
    for c in block.chars() {
        match c {
            '\n' | '\r' => out.push(c),
            _ => out.extend(std::iter::repeat_n(' ', c.len_utf8())),
        }
    }
    out.push_str(body);
    out
}

fn strip_delimiters(block: &str) -> Option<&str> {
    let opened = block.trim_start().strip_prefix(DELIMITER)?;
    let closing = opened.rfind(DELIMITER)?;
    Some(&opened[..closing])
}

fn to_json(yaml: Yaml) -> Option<Value> {
    let value = match yaml {
        Yaml::Null => Value::Null,
        Yaml::Boolean(b) => Value::Bool(b),
        Yaml::Integer(i) => Value::Number(i.into()),
        // A float YAML cannot express in JSON (inf, nan) stays as its literal.
        Yaml::Real(r) => r
            .parse::<f64>()
            .ok()
            .and_then(Number::from_f64)
            .map_or(Value::String(r), Value::Number),
        Yaml::String(s) => Value::String(s),
        Yaml::Array(items) => Value::Array(items.into_iter().filter_map(to_json).collect()),
        Yaml::Hash(entries) => {
            let mut object = Map::new();
            for (key, value) in entries {
                object.insert(key_to_string(key)?, to_json(value)?);
            }
            Value::Object(object)
        }
        Yaml::Alias(_) | Yaml::BadValue => return None,
    };
    Some(value)
}

fn key_to_string(key: Yaml) -> Option<String> {
    match key {
        Yaml::String(s) => Some(s),
        Yaml::Integer(i) => Some(i.to_string()),
        Yaml::Boolean(b) => Some(b.to_string()),
        Yaml::Real(r) => Some(r),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case::scalars(
        "---\nstatus: draft\nowner: alice\ndone: true\ncount: 3\nratio: 1.5\nempty:\n---\n\n# Body\n",
        json!({"status": "draft", "owner": "alice", "done": true, "count": 3, "ratio": 1.5, "empty": null})
    )]
    #[case::nested(
        "---\ntaxonomy:\n  - infra\n  - rollout\nauthor:\n  name: alice\n  team: sre\n---\n\n# Body\n",
        json!({"taxonomy": ["infra", "rollout"], "author": {"name": "alice", "team": "sre"}})
    )]
    #[case::sequence_root("---\n- one\n- two\n---\n\n# Body\n", json!(["one", "two"]))]
    #[case::no_body("---\nstatus: draft\n---\n", json!({"status": "draft"}))]
    fn it_parses_yaml_frontmatter(#[case] input: &str, #[case] expected: Value) {
        assert_eq!(frontmatter(input), Some(expected));
    }

    #[rstest]
    #[case::absent("# Body\n\nNo frontmatter here.\n")]
    #[case::empty_block("---\n---\n\n# Body\n")]
    #[case::not_at_start("# Body\n\n---\nstatus: draft\n---\n")]
    #[case::unclosed("---\nstatus: draft\n\n# Body\n")]
    #[case::malformed("---\nstatus: [unclosed\n---\n\n# Body\n")]
    #[case::thematic_break("Some text\n\n---\n\nMore text\n")]
    fn it_returns_none(#[case] input: &str) {
        assert_eq!(frontmatter(input), None);
    }

    #[rstest]
    fn it_does_not_read_a_fenced_block_as_frontmatter() {
        let input = "# Body\n\n```markdown\n---\nstatus: draft\n---\n```\n";
        assert_eq!(frontmatter(input), None);
    }

    #[rstest]
    #[case::block("---\nstatus: draft\n---\n\n# Body\n", "# Body\n")]
    #[case::no_blank_line("---\nstatus: draft\n---\n# Body\n", "# Body\n")]
    #[case::blank_lines(
        "---\nstatus: draft\n---\n\n\n# Body\n\n\ntext\n",
        "# Body\n\n\ntext\n"
    )]
    #[case::crlf("---\r\nstatus: draft\r\n---\r\n\r\n# Body\r\n", "# Body\r\n")]
    #[case::crlf_blank_lines("---\r\nk: v\r\n---\r\n\r\n\r\n# Body\r\n", "# Body\r\n")]
    #[case::indented_content("---\nk: v\n---\n \n\n    code\n", "    code\n")]
    #[case::bom("\u{feff}---\nstatus: draft\n---\n\n# Body\n", "# Body\n")]
    #[case::no_body("---\nstatus: draft\n---\n", "")]
    #[case::absent("# Body\n\ntext\n", "# Body\n\ntext\n")]
    #[case::empty_block("---\n---\n\n# Body\n", "---\n---\n\n# Body\n")]
    #[case::unclosed("---\nstatus: draft\n\n# Body\n", "---\nstatus: draft\n\n# Body\n")]
    #[case::malformed(
        "---\nstatus: [unclosed\n---\n\n# Body\n",
        "---\nstatus: [unclosed\n---\n\n# Body\n"
    )]
    fn it_returns_the_text_after_a_recognised_block(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(body(input), expected);
    }

    #[rstest]
    #[case::ascii(
        "---\nstatus: draft\n---\n\n# Body\n",
        "   \n             \n   \n\n# Body\n"
    )]
    #[case::multibyte("---\nk: あ\n---\n# Body\n", "   \n      \n   \n# Body\n")]
    #[case::crlf("---\r\nk: v\r\n---\r\n# Body\r\n", "   \r\n    \r\n   \r\n# Body\r\n")]
    fn it_masks_a_block_keeping_lines_and_offsets(#[case] input: &str, #[case] expected: &str) {
        let masked = mask(input);
        assert_eq!(masked, expected);
        assert_eq!(masked.len(), input.len());
    }

    #[rstest]
    fn it_masks_nothing_without_a_recognised_block() {
        let input = "---\nstatus: [unclosed\n---\n\n# Body\n";
        assert!(matches!(mask(input), Cow::Borrowed(text) if text == input));
    }
}
