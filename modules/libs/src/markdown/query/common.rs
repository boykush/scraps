use comrak::{
    nodes::{AstNode, NodeValue},
    Options,
};

pub(super) fn options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.wikilinks_title_after_pipe = true;
    opts.extension.tasklist = true;
    opts.parse.relaxed_tasklist_matching = true;
    opts
}

pub(super) fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for d in node.descendants() {
        if let NodeValue::Text(t) = &d.data().value {
            s.push_str(t);
        }
    }
    s
}

pub(super) fn parse_wikilink_url(url: &str) -> (Vec<String>, String, Option<String>) {
    let (path, heading) = match url.split_once('#') {
        Some((p, h)) => (p.to_string(), Some(h.to_string())),
        None => (url.to_string(), None),
    };
    let mut parts: Vec<String> = path.split('/').map(|s| s.to_string()).collect();
    let title = parts.pop().unwrap_or_default();
    (parts, title, heading)
}

pub(super) fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

pub(super) fn byte_to_line(starts: &[usize], byte: usize) -> usize {
    starts.partition_point(|&s| s <= byte)
}

pub(super) fn line_col_to_byte(starts: &[usize], line: usize, col: usize) -> usize {
    let li = line.saturating_sub(1);
    let base = starts.get(li).copied().unwrap_or(0);
    base + col.saturating_sub(1)
}

pub(super) fn line_byte_offset(starts: &[usize], total_len: usize, line: usize) -> usize {
    if line == 0 {
        return 0;
    }
    starts.get(line - 1).copied().unwrap_or(total_len)
}

pub(super) fn code_byte_ranges<'a>(root: &'a AstNode<'a>, starts: &[usize]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for n in root.descendants() {
        let in_code = matches!(
            &n.data().value,
            NodeValue::CodeBlock(_) | NodeValue::Code(_)
        );
        if !in_code {
            continue;
        }
        let pos = n.data().sourcepos;
        let s = line_col_to_byte(starts, pos.start.line, pos.start.column);
        let e = line_col_to_byte(starts, pos.end.line, pos.end.column) + 1;
        ranges.push((s, e));
    }
    ranges.sort_by_key(|r| r.0);
    ranges
}

pub(super) fn in_code(ranges: &[(usize, usize)], byte: usize) -> bool {
    ranges.iter().any(|(s, e)| *s <= byte && byte < *e)
}
