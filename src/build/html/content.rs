pub fn insert(html: &str, content: &str) -> String {
    html.replace("{{ content }}", content)
}
