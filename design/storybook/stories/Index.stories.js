// The wiki index page: src/usecase/build/html/builtins/index.html, plus the
// `tag_links` component in macros.html. Markup duplicated deliberately — the
// templates are Tera, rendered by the Rust binary. Change one, change the other.

const tag = ({ title, count }) => `
  <li class="item">
    <a class="link" href="#"><span class="title">#${title}</span><span class="count">${count}</span></a>
  </li>`;

export default { title: 'Index' };

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

export const ListHeadAndPaging = {
  name: 'List head and paging',
  render: () => `
    <div class="index">
      <div class="links-block">
        <p class="list-head">
          <span class="view-nav">
            <span class="view active">updated</span>
            <a class="view" href="#">backlinks</a>
            <a class="view" href="#">titles</a>
          </span>
          <a class="all-link" href="#">all scraps &#8250;</a>
        </p>
      </div>
      <div class="paging-arrows">
        <a class="prev" href="#">&#8249; prev</a>
        <a class="next" href="#">next &#8250;</a>
      </div>
    </div>`,
};
