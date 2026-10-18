// The list page of a scope (the whole wiki or one tag):
// src/usecase/build/html/builtins/index.html with the `facet_bar` component,
// plus the `tag_links` component in macros.html. Markup duplicated
// deliberately — the templates are Tera, rendered by the Rust binary.

const tag = ({ title, count }) => `
  <li class="item">
    <a class="link" href="#"><span class="title">#${title}</span><span class="count">${count}</span></a>
  </li>`;

export default { title: 'Index' };

export const Tags = {
  parameters: {
    docs: {
      description: {
        story: 'The tag_links component, used by the tags index page.',
      },
    },
  },
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

const sortFacet = (current) => `
  <div class="facets">
    <p class="facet facet-sort">
      <span class="label">sort</span>
      ${['updated', 'backlinks', 'title']
        .map((sort) => `<a class="option" href="#"${sort === current ? ' aria-current="true"' : ''}>${sort}</a>`)
        .join('')}
    </p>
  </div>`;

export const ListHeadAndFacets = {
  name: 'List head and facets',
  render: () => `
    <div class="index" data-sort="updated">
      <p class="list-head">
        <span class="view-name"><span class="syntax">#[[</span>Security<span class="syntax">]]</span></span>
        <span class="stats">170 of 764 scraps</span>
      </p>
      ${sortFacet('updated')}
      <button type="button" class="more">show more</button>
    </div>`,
};

export const TitleOrder = {
  name: 'Title order',
  parameters: {
    docs: {
      description: {
        story:
          'Sorted by title, the list gains a jump bar and a heading per initial: gojuon rows, A–Z, 漢字, #. Other sorts hide both.',
      },
    },
  },
  render: () => `
    <div class="index" data-sort="title">
      ${sortFacet('title')}
      <p class="jump-bar">${['あ', 'A', 'P', '漢字']
        .map((label) => `<a class="jump" href="#">${label}</a>`)
        .join('')}</p>
      <ul class="scrap-rows">
        <li class="group-head">A</li>
        <li class="item"><a class="row" href="#"><span class="body"><span class="head"><span class="title">Argo CD</span><span class="type">Tool</span></span></span></a></li>
        <li class="group-head">P</li>
        <li class="item"><a class="row" href="#"><span class="body"><span class="head"><span class="title">Pod</span><span class="ctx">Kubernetes</span></span></span></a></li>
      </ul>
    </div>`,
};
