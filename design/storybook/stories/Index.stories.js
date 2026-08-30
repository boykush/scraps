// The wiki index page: src/usecase/build/html/builtins/index.html, plus the
// `tag_links` component in macros.html. Markup duplicated deliberately — the
// templates are Tera, rendered by the Rust binary. Change one, change the other.

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

export const ListHeadAndPaging = {
  name: 'List head and paging',
  render: () => `
    <div class="index">
      <div class="links-block">
        <p class="list-head">
          <span class="view-name">recently updated</span>
          <span class="stats">764 scraps &#183; 41 tags</span>
        </p>
      </div>
      <div class="paging-arrows">
        <a class="prev" href="#">&#8249; prev</a>
        <a class="next" href="#">next &#8250;</a>
      </div>
    </div>`,
};
