// Mirrors the index and scrap page chrome in
// src/usecase/build/html/builtins/{index,scrap,tag,scraps_index}.html.
// Duplicated deliberately; see Listings.stories.js.

export default { title: 'Chrome' };

export const Search = {
  parameters: {
    docs: {
      description: {
        story:
          'Results are positioned by doSearch() with an inline `top`, the first at -16px, which the list margin compensates for.',
      },
    },
  },
  render: () => `
    <div class="index">
      <div class="search-block">
        <input type="text" id="search-input" placeholder="Search by title..." />
        <ul id="search-results">
          <li style="top: -16px"><a href="#">Wiki-link Notation</a></li>
          <li style="top: 14px"><a href="#">Configuration</a></li>
          <li style="top: 44px"><a href="#">Lint Rules</a></li>
        </ul>
      </div>
    </div>
    <div style="height: 120px"></div>`,
};

export const ListHeadAndPaging = {
  name: 'List head and paging',
  render: () => `
    <div class="index">
      <div class="links-block">
        <p class="list-head">
          <span class="sort-key">sorted by committed date</span>
          <a class="all-link" href="#">all scraps &#8250;</a>
        </p>
      </div>
      <div class="paging-arrows">
        <a class="prev" href="#">&#8249; prev</a>
        <a class="next" href="#">next &#8250;</a>
      </div>
    </div>`,
};

export const ScrapHeader = {
  name: 'Scrap header',
  render: () => `
    <div class="scrap">
      <h3 class="context">Reference/Wiki-link<span>&#47;</span></h3>
      <h1 class="title">Heading Reference</h1>
      <p class="commited-date">commited date: 2026-08-12</p>
    </div>`,
};

export const TagHeader = {
  name: 'Tag header',
  render: () => `<div class="tag"><h1 class="title">Security</h1></div>`,
};

export const ScrapsIndexHead = {
  name: 'Scraps index head',
  render: () => `
    <div class="scraps-index">
      <p class="list-head">
        <span class="count">712 scraps</span>
        <span class="legend">title order &#183; backlinks</span>
      </p>
    </div>`,
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
  render: () => `
    <div class="scrap"><div class="content">
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
      </div>
    </div></div>`,
};
