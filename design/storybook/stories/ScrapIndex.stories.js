// The all-scraps index page: src/usecase/build/html/builtins/scraps_index.html
// and the `scrap_index` component in macros.html. One page, grouped by the
// title's initial with a jump bar — the title index and the dense listing
// merged. Markup duplicated deliberately — the templates are Tera, rendered
// by the Rust binary. Change one, change the other.

const entry = ({ title, ctx, count }) => `
  <li class="item">
    <a class="entry" href="#">
      <span class="title">${title}</span>
      ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
      <span class="count">${count}</span>
    </a>
  </li>`;

const group = ({ label, scraps }) => `
  <section class="title-group">
    <h2 class="group-head">
      <span class="label">${label}</span>
      <span class="count">${scraps.length}</span>
    </h2>
    <ul class="scrap-index">${scraps.map(entry).join('')}</ul>
  </section>`;

export default { title: 'Scrap index' };

export const Head = {
  render: () => `
    <div class="scraps-index">
      <p class="list-head">
        <span class="view-name">all scraps</span>
        <span class="stats">712 scraps</span>
      </p>
      <p class="jump-bar">
        <a class="jump" href="#">あ</a>
        <a class="jump" href="#">か</a>
        <a class="jump" href="#">に</a>
        <a class="jump" href="#">A</a>
        <a class="jump" href="#">C</a>
        <a class="jump" href="#">P</a>
        <a class="jump" href="#">漢字</a>
        <a class="jump" href="#">#</a>
      </p>
    </div>`,
};

export const Groups = {
  parameters: {
    docs: {
      description: {
        story:
          'Grouped by the title\'s initial — gojuon rows, A–Z, 漢字, #. Multi-column inside a group, unpaginated. The ctx sits beside the title because `Pod` and `Pod (Kubernetes)` are otherwise indistinguishable.',
      },
    },
  },
  render: () =>
    `<div class="scraps-index">${[
      {
        label: 'A',
        scraps: [
          { title: 'ABAC', count: 4 },
          { title: 'ADR', count: 9 },
          { title: 'Ambient Mesh', ctx: 'Istio', count: 5 },
          { title: 'Annotation', ctx: 'Kubernetes', count: 6 },
          { title: 'Argo CD', count: 11 },
          { title: 'A Deliberately Long Scrap Title That Has To Be Truncated', ctx: 'Reference', count: 2 },
        ],
      },
      {
        label: 'P',
        scraps: [
          { title: 'Pod', ctx: 'Kubernetes', count: 12 },
          { title: 'Pod', count: 4 },
        ],
      },
      {
        label: 'や',
        scraps: [{ title: 'ユビキタス言語', count: 11 }],
      },
      {
        label: '漢字',
        scraps: [
          { title: '境界づけられたコンテキスト', count: 10 },
          { title: '認知負荷', count: 9 },
        ],
      },
    ]
      .map(group)
      .join('')}</div>`,
};
