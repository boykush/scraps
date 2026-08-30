// The scrap detail page assembled whole, mirroring
// src/usecase/build/html/builtins/scrap.html. The pieces are also available on
// their own under Chrome and Content; this exists because the spacing between
// them, and the linked-scraps list closing the page, only read in context.

const link = ({ title, ctx }) => `
  <li class="item">
    <a class="link" href="#">
      <span class="title">${title}</span>
      ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
    </a>
  </li>`;

const section = (label, scraps) =>
  scraps.length
    ? `<section class="connections">
        <p class="section-head">${label} &#183; ${scraps.length}</p>
        <ul class="scrap-links">${scraps.map(link).join('')}</ul>
      </section>`
    : '';

const page = ({ tags = [], ctx, title, date, content, backlinks = [], links = [] }) => `
  <div class="scrap">
    ${
      tags.length
        ? `<p class="tags">${tags
            .map(
              (t) =>
                `<a class="tag" href="#"><span class="syntax">#[[</span>${t}<span class="syntax">]]</span></a>`,
            )
            .join('')}</p>`
        : ''
    }
    ${ctx ? `<h3 class="context">${ctx}<span>&#47;</span></h3>` : ''}
    <h1 class="title">${title}</h1>
    ${date ? `<p class="commited-date">commited date: ${date}</p>` : ''}
    <div class="content">${content}</div>
  </div>
  ${section('backlinks', backlinks)}
  ${section('links', links)}`;

const LINKED = [
  { title: 'Wiki-link Notation', ctx: 'Reference' },
  { title: 'Lint Rules', ctx: 'Reference' },
  { title: 'CLI Overview', ctx: 'Reference' },
  { title: 'Normal Link', ctx: 'Reference/Wiki-link' },
  { title: 'Getting Started', ctx: 'Tutorial' },
  { title: 'What is Scraps?', ctx: 'Explanation' },
];

const FULL_CONTENT = `
  <p>A heading reference points at a section rather than a whole scrap. Write
  <code>[[Page#Heading]]</code> and the compiler resolves it against the
  target's heading list, emitting a fragment link in HTML and a structured ref
  in <a href="#">JSON</a>.</p>

  <h2>Lint</h2>

  <p>A reference whose heading no longer exists is a
  <code>broken_heading_ref</code> error, not a warning: the link is typed, so a
  dangling one is a type error.</p>

  <pre><code>$ scraps lint --rule broken_heading_ref
Reference/Wiki-link/Heading Reference.md:14
  broken_heading_ref  [[Configuration#output-dir]]</code></pre>

  <blockquote><p>Tags and scraps live in separate namespaces, and a tag is
  never an implicit fallback for an unresolved link.</p></blockquote>

  <h2>Resolution</h2>

  <ul>
    <li>Short form searches by title</li>
    <li>Ambiguity is an error, Java-import style</li>
    <li>Depth is bounded at three</li>
  </ul>

  <table>
    <thead><tr><th>Form</th><th>Resolves to</th></tr></thead>
    <tbody>
      <tr><td><code>[[Title]]</code></td><td>a scrap by title</td></tr>
      <tr><td><code>[[Ctx/Title]]</code></td><td>a scrap in a namespace</td></tr>
      <tr><td><code>[[Title#Heading]]</code></td><td>a section of one</td></tr>
    </tbody>
  </table>`;

export default {
  title: 'Scrap/Page',
  parameters: { docs: { description: { component: 'A compiled scrap, end to end.' } } },
};

export const Full = {
  render: () =>
    page({
      tags: ['notation', 'reference'],
      ctx: 'Reference/Wiki-link',
      title: 'Heading Reference',
      date: '2026-08-12',
      content: FULL_CONTENT,
      backlinks: LINKED,
      links: LINKED.slice(0, 2),
    }),
};

export const WithoutContext = {
  name: 'Without ctx',
  render: () =>
    page({
      title: 'Getting Started',
      date: '2026-08-02',
      content: '<p>Install, create a wiki, compile it, then query the same source from a shell.</p>',
      backlinks: LINKED.slice(0, 3),
    }),
};

export const WithoutGitMetadata = {
  name: 'Without git metadata',
  parameters: {
    docs: {
      description: {
        story: 'Built without --git, so there is no committed date under the title.',
      },
    },
  },
  render: () =>
    page({
      ctx: 'Reference',
      title: 'Configuration',
      content: '<p><code>.scraps.toml</code> declares a Scraps wiki: the directory containing this file is the wiki root.</p>',
      backlinks: LINKED.slice(0, 2),
    }),
};

export const WithoutLinks = {
  name: 'A dead end',
  parameters: {
    docs: {
      description: {
        story: 'A scrap that links nowhere. The lint rule dead_end exists to find these; the page still has to hold together without the closing list.',
      },
    },
  },
  render: () =>
    page({
      title: 'Orphan',
      date: '2026-07-30',
      content: '<p>Nothing below this paragraph, because the scrap has no outbound links.</p>',
    }),
};

export const LongTitle = {
  name: 'Long title',
  render: () =>
    page({
      ctx: 'Reference/Static Site',
      title: 'A Deliberately Long Scrap Title That Has To Wrap Across More Than One Line',
      date: '2026-08-20',
      content: '<p>The title above wraps rather than truncating, since a detail page has the room the index rows do not.</p>',
      backlinks: LINKED.slice(0, 4),
    }),
};
