// Parts of the scrap detail page: src/usecase/build/html/builtins/scrap.html,
// the `scrap_links` and `ogp_card` components in macros.html, and the block
// elements main.css styles directly. The whole page is under Scrap / Page.

const wrap = (inner) => `<div class="scrap"><div class="content">${inner}</div></div>`;

const link = ({ title, ctx }) => `
  <li class="item">
    <a class="link" href="#">
      <span class="title">${title}</span>
      ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
    </a>
  </li>`;

export default { title: 'Scrap' };

export const Header = {
  render: () => `
    <div class="scrap">
      <h3 class="context">Reference/Wiki-link<span>&#47;</span></h3>
      <h1 class="title">Heading Reference</h1>
      <p class="commited-date">commited date: 2026-08-12</p>
    </div>`,
};

export const Typography = {
  render: () =>
    wrap(`
      <h2>Second level</h2>
      <h3>Third level</h3>
      <p>Body copy at the base size, long enough to wrap and show the line height,
      carrying <strong>bold</strong>, <em>italic</em>, <code>inline code</code> and
      an <a href="#">internal link</a>, which is underlined because colour alone
      does not separate it from the text around it.</p>`),
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

export const OgpCard = {
  name: 'OGP card',
  parameters: {
    docs: {
      description: {
        story: 'A bare URL in a scrap becomes this card, filled in client side after the page loads.',
      },
    },
  },
  render: () =>
    wrap(`
      <div class="ogp-card">
        <a class="ogp-card-link" href="#">
          <img class="ogp-image" alt="" src="data:image/svg+xml;utf8,${encodeURIComponent(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 120"><rect width="240" height="120" fill="%235e81ac"/></svg>',
          )}" />
          <div class="ogp-content">
            <div class="ogp-title">Nord Theme</div>
            <div class="ogp-description">An arctic, north-bluish clean and elegant colour palette.</div>
          </div>
        </a>
      </div>`),
};

export const LinkedScraps = {
  name: 'Linked scraps',
  parameters: {
    docs: {
      description: {
        story: 'Closes every scrap page and every tag page: title and ctx only, no summary or counts.',
      },
    },
  },
  render: () =>
    `<ul class="scrap-links">${[
      { title: 'Wiki-link Notation', ctx: 'Reference' },
      { title: 'Lint Rules', ctx: 'Reference' },
      { title: 'CLI Overview', ctx: 'Reference' },
      { title: 'Normal Link', ctx: 'Reference/Wiki-link' },
      { title: 'Getting Started', ctx: 'Tutorial' },
      { title: 'What is Scraps?', ctx: 'Explanation' },
    ]
      .map(link)
      .join('')}</ul>`,
};
