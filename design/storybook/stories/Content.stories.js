// The block elements a compiled scrap can contain, styled by main.css itself
// rather than by any component template.

const wrap = (inner) => `<div class="scrap"><div class="content">${inner}</div></div>`;

export default { title: 'Content' };

export const Typography = {
  render: () =>
    wrap(`
      <h2>Second level</h2>
      <h3>Third level</h3>
      <p>Body copy at the base size, long enough to wrap and show the line height,
      carrying <strong>bold</strong>, <em>italic</em>, <code>inline code</code> and
      an <a href="#">internal link</a>.</p>`),
};

export const Lists = {
  render: () =>
    wrap(`
      <ul><li>Unordered item</li><li>Item with a nested list<ul><li>Nested item</li></ul></li></ul>
      <ol><li>First step</li><li>Second step</li></ol>`),
};

export const Quote = {
  render: () =>
    wrap(
      '<blockquote><p>A typed reference resolves, lints and emits. A dangling one is an error, not a warning.</p></blockquote>',
    ),
};

export const Code = {
  render: () =>
    wrap(`<pre><code>pub fn from_md_text(md_text: &amp;str, max_chars: usize) -&gt; Option&lt;Summary&gt; {
    let mut lines = md_text.lines().map(str::trim);
    let first = lines.find(|l| !l.is_empty())?;
    Some(Summary(truncate(first, max_chars)))
}</code></pre>`),
};

export const Table = {
  render: () =>
    wrap(`<table>
      <thead><tr><th>Role</th><th>Light</th><th>Dark</th></tr></thead>
      <tbody>
        <tr><td>Surface</td><td>nord6</td><td>nord0</td></tr>
        <tr><td>Text</td><td>nord3</td><td>nord6</td></tr>
        <tr><td>Accent</td><td>ext.frost-deep</td><td>nord8</td></tr>
      </tbody>
    </table>`),
};
