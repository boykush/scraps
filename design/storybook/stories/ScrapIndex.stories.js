// The full scrap index page: src/usecase/build/html/builtins/scraps_index.html
// and the `scrap_index` component in macros.html.

const entry = ({ title, ctx, count }) => `
  <li class="item">
    <a class="entry" href="#">
      <span class="title">${title}</span>
      ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
      <span class="count">${count}</span>
    </a>
  </li>`;

export default { title: 'Scrap index' };

export const Head = {
  render: () => `
    <div class="scraps-index">
      <p class="list-head">
        <span class="count">712 scraps</span>
        <span class="legend">title order &#183; backlinks</span>
      </p>
    </div>`,
};

export const Entries = {
  parameters: {
    docs: {
      description: {
        story:
          'Multi-column and unpaginated. The ctx sits beside the title because `Pod` and `Pod (Kubernetes)` are otherwise indistinguishable.',
      },
    },
  },
  render: () =>
    `<ul class="scrap-index">${[
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
    ]
      .map(entry)
      .join('')}</ul>`,
};
