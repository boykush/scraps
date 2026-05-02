use comrak::{nodes::NodeValue, options::Extension, parse_document, Arena, Options};
use url::Url;

fn options() -> Options<'static> {
    Options {
        extension: Extension::default(),
        ..Options::default()
    }
}

pub fn head_image(text: &str) -> Option<Url> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    for node in root.descendants() {
        if let NodeValue::Image(node_link) = &node.data().value {
            return Url::parse(&node_link.url).ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::image_found(
        "![alt](https://example.com/image.png)",
        Some("https://example.com/image.png")
    )]
    #[case::no_image("# header1", None)]
    fn it_head_image(#[case] input: &str, #[case] expected_url: Option<&str>) {
        assert_eq!(
            head_image(input),
            expected_url.map(|u| Url::parse(u).unwrap())
        );
    }
}
