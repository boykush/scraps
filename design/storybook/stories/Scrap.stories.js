// Parts of the scrap detail page: src/usecase/build/html/builtins/scrap.html,
// the `scrap_links` and `link_card` components in macros.html, and the block
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
      <p class="tags">
        <a class="tag" href="#"><span class="syntax">#[[</span>notation<span class="syntax">]]</span></a>
      </p>
      <h3 class="context">Reference/Wiki-link<span>&#47;</span></h3>
      <h1 class="title">Heading Reference</h1>
      <p class="commited-date">committed 2026-08-12</p>
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

// Storybook renders against main.css alone, so the hljs-* spans are written by
// hand here to stand in for what highlight.js emits at runtime.
export const Code = {
  parameters: {
    docs: {
      description: {
        story:
          'Code blocks wear surface-raised with a hairline, so the block keeps an edge on the dark ground where the page is nord0. The five syntax roles are gated against that ground, not the page.',
      },
    },
  },
  render: () =>
    wrap(`<pre><code class="hljs language-rust"><span class="hljs-comment">// The summary is a scrap's first non-empty line, truncated.</span>
<span class="hljs-keyword">pub</span> <span class="hljs-keyword">fn</span> <span class="hljs-title function_">from_md_text</span>(md_text: &amp;<span class="hljs-type">str</span>, max_chars: <span class="hljs-type">usize</span>) -&gt; <span class="hljs-type">Option</span>&lt;<span class="hljs-title class_">Summary</span>&gt; {
    <span class="hljs-keyword">let</span> <span class="hljs-keyword">mut</span> lines = md_text.<span class="hljs-title function_ invoke__">lines</span>().<span class="hljs-title function_ invoke__">map</span>(str::trim);
    <span class="hljs-keyword">let</span> first = lines.<span class="hljs-title function_ invoke__">find</span>(|l| !l.<span class="hljs-title function_ invoke__">starts_with</span>(<span class="hljs-string">"#"</span>))?;
    <span class="hljs-title function_ invoke__">Some</span>(<span class="hljs-title class_">Summary</span>(<span class="hljs-title function_ invoke__">truncate</span>(first, max_chars.<span class="hljs-title function_ invoke__">min</span>(<span class="hljs-number">120</span>))))
}</code></pre>`),
};

export const Diff = {
  parameters: {
    docs: {
      description: {
        story: 'A fenced diff. Added and removed lines carry their own roles so the pairing survives on both grounds.',
      },
    },
  },
  render: () =>
    wrap(`<pre><code class="hljs language-diff"><span class="hljs-meta">--- a/tokens/semantic.tokens.json</span>
<span class="hljs-meta">+++ b/tokens/semantic.tokens.json</span>
<span class="hljs-deletion">-      "$value": "{color.nord.8}"</span>
<span class="hljs-addition">+      "$value": "{color.ext.frost-deep}"</span>
</code></pre>`),
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

export const LinkCard = {
  name: 'Link card',
  parameters: {
    docs: {
      description: {
        story: 'A bare URL in a scrap becomes this static card at build time: host and URL, nothing fetched client side.',
      },
    },
  },
  render: () =>
    wrap(`
      <a class="link-card" href="#">
        <svg class="icon" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="M6.5 9.5 L9.5 6.5"></path><path d="M7.5 4.5 L9 3 A2.1 2.1 0 0 1 12 6 L10.5 7.5"></path><path d="M8.5 11.5 L7 13 A2.1 2.1 0 0 1 4 10 L5.5 8.5"></path></svg>
        <span class="body">
          <span class="host">www.nordtheme.com</span>
          <span class="url">https://www.nordtheme.com/docs/colors-and-palettes</span>
        </span>
      </a>`),
};

export const Connections = {
  name: 'Connections',
  parameters: {
    docs: {
      description: {
        story: 'Labelled backlinks / links sections close every scrap page: title and ctx only, no summary.',
      },
    },
  },
  render: () => `
    <section class="connections">
      <p class="section-head">backlinks &#183; 6</p>
      <ul class="scrap-links">${[
        { title: 'Wiki-link Notation', ctx: 'Reference' },
        { title: 'Lint Rules', ctx: 'Reference' },
        { title: 'CLI Overview', ctx: 'Reference' },
        { title: 'Normal Link', ctx: 'Reference/Wiki-link' },
        { title: 'Getting Started', ctx: 'Tutorial' },
        { title: 'What is Scraps?', ctx: 'Explanation' },
      ]
        .map(link)
        .join('')}</ul>
    </section>`,
};
