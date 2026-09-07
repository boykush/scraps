use comrak::{nodes::NodeValue, parse_document, Arena};
use serde_json::{Map, Number, Value};
use yaml_rust2::yaml::{Yaml, YamlLoader};

use super::common::options;

const DELIMITER: &str = "---";

/// The YAML frontmatter block of a scrap, as opaque structured data.
///
/// Scraps assigns no meaning to any key — this is passthrough, so the caller
/// sees exactly what the author wrote. A block that is absent, empty, unclosed
/// or malformed reads as `None`, so a retrofit wiki still builds.
pub fn frontmatter(text: &str) -> Option<Value> {
    let arena = Arena::new();
    let mut opts = options();
    // Only this query recognises the block; the others still read it as
    // content, which is the behaviour the rest of the pipeline was built on.
    opts.extension.front_matter_delimiter = Some(DELIMITER.to_string());
    let root = parse_document(&arena, text, &opts);

    let block = root
        .descendants()
        .find_map(|node| match &node.data().value {
            NodeValue::FrontMatter(raw) => Some(raw.clone()),
            _ => None,
        })?;

    let docs = YamlLoader::load_from_str(strip_delimiters(&block)?).ok()?;
    to_json(docs.into_iter().next()?)
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
}
