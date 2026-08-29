// Mirrors `scrap_index`, `scrap_links` and `tag_links` in
// src/usecase/build/html/builtins/macros.html. Duplicated deliberately: the
// templates are Tera, rendered by the Rust binary, so they cannot be mounted
// here. Change one, change the other.

const entry = ({ title, ctx, count }) => `
  <li class="item">
    <a class="entry" href="#">
      <span class="title">${title}</span>
      ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
      <span class="count">${count}</span>
    </a>
  </li>`;

const link = ({ title, ctx }) => `
  <li class="item">
    <a class="link" href="#">
      <span class="title">${title}</span>
      ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
    </a>
  </li>`;

const tag = ({ title, count }) => `
  <li class="item">
    <a class="link" href="#"><span class="title">#${title}</span><span class="count">${count}</span></a>
  </li>`;

const ENTRIES = [
  { title: 'ABAC', count: 4 },
  { title: 'ADR', count: 9 },
  { title: 'Ambient Mesh', ctx: 'Istio', count: 5 },
  { title: 'Annotation', ctx: 'Kubernetes', count: 6 },
  { title: 'Argo CD', count: 11 },
  { title: 'Claude Code', count: 18 },
  { title: 'Pod', ctx: 'Kubernetes', count: 12 },
  { title: 'Pod', count: 4 },
  { title: 'ユビキタス言語', count: 11 },
  { title: '境界づけられたコンテキスト', count: 10 },
  { title: 'A Deliberately Long Scrap Title That Has To Be Truncated', ctx: 'Reference', count: 2 },
  { title: '認知負荷', count: 9 },
];

export default { title: 'Listings' };

export const FullIndex = {
  name: 'Scrap index',
  parameters: {
    docs: {
      description: {
        story:
          'The whole wiki on one page, in title order, multi-column and unpaginated. The ctx sits beside the title because `Pod` and `Pod (Kubernetes)` are otherwise indistinguishable.',
      },
    },
  },
  render: () => `<ul class="scrap-index">${ENTRIES.map(entry).join('')}</ul>`,
};

export const LinkedScraps = {
  name: 'Linked scraps',
  parameters: {
    docs: {
      description: {
        story: 'Shown below a scrap and on tag pages: title and ctx only, no summary or counts.',
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

export const Tags = {
  render: () =>
    `<ul class="tag-links">${[
      { title: 'Security', count: 170 },
      { title: 'Programming', count: 87 },
      { title: 'Agile', count: 67 },
      { title: 'Cloud Native', count: 63 },
      { title: 'Security/Authentication', count: 40 },
      { title: 'Documentation', count: 38 },
    ]
      .map(tag)
      .join('')}<li class="item"><a class="link more" href="#">More&#8230;</a></li></ul>`,
};
