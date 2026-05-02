use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena,
};

use super::common::options;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Done,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub status: TaskStatus,
    pub text: String,
    pub line: usize,
}

fn task_item_body<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for child in node.children() {
        if !matches!(child.data().value, NodeValue::Paragraph) {
            continue;
        }
        for d in child.descendants() {
            if let NodeValue::Text(t) = &d.data().value {
                s.push_str(t);
            }
        }
    }
    s
}

pub fn task_items(text: &str) -> Vec<TaskItem> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    let mut out = Vec::new();
    for node in root.descendants() {
        let NodeValue::TaskItem(item) = &node.data().value else {
            continue;
        };
        let status = match item.symbol {
            None | Some(' ') => TaskStatus::Open,
            Some('x') | Some('X') => TaskStatus::Done,
            Some('-') => TaskStatus::Deferred,
            _ => continue,
        };
        let body = task_item_body(node);
        let line = node.data().sourcepos.start.line;
        out.push(TaskItem {
            status,
            text: body,
            line,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn item(status: TaskStatus, text: &str, line: usize) -> TaskItem {
        TaskItem {
            status,
            text: text.to_string(),
            line,
        }
    }

    #[rstest]
    #[case::open("- [ ] open", vec![item(TaskStatus::Open, "open", 1)])]
    #[case::done_lower("- [x] done", vec![item(TaskStatus::Done, "done", 1)])]
    #[case::done_upper("- [X] DONE", vec![item(TaskStatus::Done, "DONE", 1)])]
    #[case::deferred("- [-] deferred", vec![item(TaskStatus::Deferred, "deferred", 1)])]
    fn it_task_items_status(#[case] input: &str, #[case] expected: Vec<TaskItem>) {
        assert_eq!(task_items(input), expected);
    }

    #[rstest]
    #[case::slash("- [/] other")]
    #[case::question("- [?] huh")]
    #[case::dot("- [.] dot")]
    fn it_task_items_unsupported_skipped(#[case] input: &str) {
        assert!(task_items(input).is_empty());
    }

    #[test]
    fn it_task_items_nested_list() {
        let input = "- [ ] top\n  - [x] sub\n  - [-] sub-deferred\n";
        let res = task_items(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].status, TaskStatus::Open);
        assert_eq!(res[0].text, "top");
        assert_eq!(res[1].status, TaskStatus::Done);
        assert_eq!(res[1].text, "sub");
        assert_eq!(res[2].status, TaskStatus::Deferred);
        assert_eq!(res[2].text, "sub-deferred");
    }

    #[test]
    fn it_task_items_in_blockquote() {
        let input = "> - [ ] quoted\n> - [x] also\n";
        let res = task_items(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].status, TaskStatus::Open);
        assert_eq!(res[1].status, TaskStatus::Done);
    }

    #[test]
    fn it_task_items_mixed_with_plain_bullets() {
        let input = "- plain bullet\n- [ ] task\n- another plain\n- [x] done\n";
        let res = task_items(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].text, "task");
        assert_eq!(res[1].text, "done");
    }

    #[test]
    fn it_task_items_inline_formatting_text_is_plain() {
        let input = "- [ ] read **the** book";
        let res = task_items(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "read the book");
    }

    #[test]
    fn it_task_items_with_wikilink_in_text() {
        let input = "- [ ] link to [[topic]] now";
        let res = task_items(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "link to topic now");
    }

    #[test]
    fn it_task_items_line_numbers_lf() {
        let input = "- [ ] first\n- [x] second\n- [-] third";
        let res = task_items(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].line, 1);
        assert_eq!(res[1].line, 2);
        assert_eq!(res[2].line, 3);
    }

    #[test]
    fn it_task_items_line_numbers_crlf() {
        let input = "- [ ] first\r\n- [x] second\r\n- [-] third";
        let res = task_items(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].line, 1);
        assert_eq!(res[1].line, 2);
        assert_eq!(res[2].line, 3);
    }

    #[test]
    fn it_task_items_composite_document() {
        let input = "\
# Title

- [ ] todo
  - [x] sub done
- regular bullet
- [-] deferred

> - [ ] quoted

```
- [ ] in-code
```
";
        let res = task_items(input);
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].status, TaskStatus::Open);
        assert_eq!(res[0].text, "todo");
        assert_eq!(res[1].status, TaskStatus::Done);
        assert_eq!(res[2].status, TaskStatus::Deferred);
        assert_eq!(res[3].status, TaskStatus::Open);
        assert_eq!(res[3].text, "quoted");
    }
}
