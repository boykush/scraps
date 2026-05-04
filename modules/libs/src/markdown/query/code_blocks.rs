use comrak::{
    nodes::{NodeCodeBlock, NodeValue},
    parse_document, Arena,
};

use super::common::options;

/// A fenced code block extracted from a markdown document.
///
/// `lang` is the first whitespace-delimited token of the info string (the
/// conventional language tag), or `None` when the fence has no info string.
/// `content` is the raw block body as authored, with the trailing newline
/// preserved by comrak. `line` is the 1-based source line of the opening
/// fence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub lang: Option<String>,
    pub content: String,
    pub line: usize,
}

/// Extract fenced code blocks from a markdown document, in occurrence order.
///
/// Indented (4-space) code blocks are intentionally skipped: only fenced
/// blocks (``` ``` ``` or ``` ~~~ ```) carry an explicit language tag, and
/// they're the construct agents care about for code introspection.
pub fn code_blocks(text: &str) -> Vec<CodeBlock> {
    if text.is_empty() {
        return Vec::new();
    }
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);

    let mut out = Vec::new();
    for n in root.descendants() {
        let NodeValue::CodeBlock(cb) = &n.data().value else {
            continue;
        };
        let NodeCodeBlock {
            fenced,
            info,
            literal,
            ..
        } = cb.as_ref();
        if !*fenced {
            continue;
        }
        let lang = info
            .split_whitespace()
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let line = n.data().sourcepos.start.line;
        out.push(CodeBlock {
            lang,
            content: literal.clone(),
            line,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_code_blocks_basic_with_lang() {
        let input = "```rust\nlet x = 1;\n```\n";
        let res = code_blocks(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].lang.as_deref(), Some("rust"));
        assert_eq!(res[0].content, "let x = 1;\n");
        assert_eq!(res[0].line, 1);
    }

    #[test]
    fn it_code_blocks_no_lang() {
        let input = "```\nplain\n```\n";
        let res = code_blocks(input);
        assert_eq!(res.len(), 1);
        assert!(res[0].lang.is_none());
        assert_eq!(res[0].content, "plain\n");
    }

    #[test]
    fn it_code_blocks_skips_indented() {
        // 4-space indented code block — not fenced, so excluded.
        let input = "para\n\n    indented code\n";
        assert!(code_blocks(input).is_empty());
    }

    #[test]
    fn it_code_blocks_multiple_with_line_numbers() {
        let input = "intro\n\n```rust\nlet x = 1;\n```\n\nmid\n\n```sh\ncargo build\n```\n";
        let res = code_blocks(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].lang.as_deref(), Some("rust"));
        assert_eq!(res[0].line, 3);
        assert_eq!(res[1].lang.as_deref(), Some("sh"));
        assert_eq!(res[1].line, 9);
    }

    #[test]
    fn it_code_blocks_info_string_takes_first_token() {
        // GFM: only the first whitespace-delimited token is the lang.
        let input = "```rust ignore\nlet x = 1;\n```\n";
        let res = code_blocks(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].lang.as_deref(), Some("rust"));
    }

    #[test]
    fn it_code_blocks_tilde_fence() {
        let input = "~~~python\nprint(1)\n~~~\n";
        let res = code_blocks(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].lang.as_deref(), Some("python"));
        assert_eq!(res[0].content, "print(1)\n");
    }

    #[test]
    fn it_code_blocks_empty_input() {
        assert!(code_blocks("").is_empty());
    }

    #[test]
    fn it_code_blocks_preserves_inner_whitespace() {
        let input = "```\n  indented inside\n\nblank line above\n```\n";
        let res = code_blocks(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].content, "  indented inside\n\nblank line above\n");
    }
}
