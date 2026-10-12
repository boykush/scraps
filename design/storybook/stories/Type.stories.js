// Where a scrap's type shows: the label in scrap.html and the `type_filter`
// component in macros.html, used by index.html and scraps_index.html. Markup
// duplicated deliberately: change one, change the other.
// A type carries no #[[ ]] marks and never sits in the tag line.

const TYPES = [
  { name: 'Reference', count: 29 },
  { name: 'How-to', count: 2 },
  { name: 'Tutorial', count: 2 },
  { name: 'Explanation', count: 1 },
];

const option = (label, count, active) => {
  const inner = `${label}${count != null ? `<span class="count">${count}</span>` : ''}`;
  return active
    ? `<span class="option active">${inner}</span>`
    : `<a class="option" href="#">${inner}</a>`;
};

const filter = (activeName) => `
  <p class="type-filter">
    <span class="label">type</span>
    ${option('all', null, !activeName)}
    ${TYPES.map((t) => option(t.name, t.count, t.name === activeName)).join('')}
  </p>`;

const listHead = (stats) => `
  <p class="list-head">
    <span class="view-name">recently updated</span>
    <span class="stats">${stats}</span>
  </p>`;

export default {
  title: 'Type',
  parameters: {
    docs: {
      description: {
        component:
          'The frontmatter `type` of a scrap: at most one per scrap. It narrows a list view the way a label narrows an issue list.',
      },
    },
  },
};

export const Filter = {
  name: 'Filter, nothing chosen',
  render: () => `<div class="index"><div class="links-block">${listHead('34 scraps &#183; 11 tags')}${filter()}</div></div>`,
};

export const FilterActive = {
  name: 'Filter, one type chosen',
  parameters: {
    docs: {
      description: {
        story: 'The same view narrowed to one type. The stats say how many of the scraps are left; `all` goes back.',
      },
    },
  },
  render: () => `<div class="index"><div class="links-block">${listHead('2 of 34 scraps')}${filter('How-to')}</div></div>`,
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
