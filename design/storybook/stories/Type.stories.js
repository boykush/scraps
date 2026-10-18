// Where a scrap's type shows: the label in scrap.html, the label in a
// `scrap_rows` row, and the type row of the `facet_bar` component in
// macros.html. Markup duplicated deliberately: change one, change the other.
// A type carries no #[[ ]] marks and never sits in the tag line.

const TYPES = [
  { name: 'Reference', count: 29 },
  { name: 'How-to', count: 2 },
  { name: 'Tutorial', count: 0 },
  { name: 'Explanation', count: 1 },
];

const option = (label, count, current) =>
  count === 0
    ? `<span class="option zero">${label}<span class="count">0</span></span>`
    : `<a class="option" href="#"${current ? ' aria-current="true"' : ''}>${label}${count != null ? `<span class="count">${count}</span>` : ''}</a>`;

const facet = (currentName) => `
  <div class="facets">
    <p class="facet facet-type">
      <span class="label">type</span>
      ${option('all', null, !currentName)}
      ${TYPES.map((t) => option(t.name, t.count, t.name === currentName)).join('')}
    </p>
  </div>`;

const listHead = (stats) => `
  <p class="list-head">
    <span class="view-name">all scraps</span>
    <span class="stats">${stats}</span>
  </p>`;

export default {
  title: 'Type',
  parameters: {
    docs: {
      description: {
        component:
          'The frontmatter `type` of a scrap: at most one per scrap. It narrows any list, through `?type=`, the way a label narrows an issue list.',
      },
    },
  },
};

export const Facet = {
  name: 'Facet, nothing chosen',
  parameters: {
    docs: {
      description: {
        story: 'Counts are within the listed scope. A type the scope holds none of is shown but cannot be picked.',
      },
    },
  },
  render: () => `<div class="index">${listHead('34 scraps')}${facet()}</div>`,
};

export const FacetActive = {
  name: 'Facet, one type chosen',
  parameters: {
    docs: {
      description: {
        story: 'The same list narrowed to one type. The stats say how many of the scraps are left; `all` goes back.',
      },
    },
  },
  render: () => `<div class="index">${listHead('2 of 34 scraps')}${facet('How-to')}</div>`,
};

export const ScrapLabel = {
  name: 'Label on a scrap',
  parameters: {
    docs: {
      description: {
        story: 'The type sits with the page metadata under the title, sharing a line with the committed date. Tags stay above the title with their syntax marks.',
      },
    },
  },
  render: () => `
    <div class="scrap">
      <p class="tags">
        <a class="tag" href="#"><span class="syntax">#[[</span>CLI<span class="syntax">]]</span></a>
      </p>
      <h3 class="context">Reference<span>&#47;</span></h3>
      <h1 class="title">CLI Overview</h1>
      <p class="type">type <a class="name" href="#">Reference</a></p>
      <p class="commited-date">committed 2026-08-12</p>
    </div>`,
};

export const ScrapLabelWithoutGitMetadata = {
  name: 'Label without git metadata',
  render: () => `
    <div class="scrap">
      <h1 class="title">Orders</h1>
      <p class="type">type <a class="name" href="#">BigQuery Table</a></p>
    </div>`,
};
