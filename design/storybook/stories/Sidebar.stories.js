// The persistent shell sidebar: src/usecase/build/html/builtins/base.html and
// the aside.sidebar block in main.css. Markup duplicated deliberately — the
// templates are Tera, rendered by the Rust binary. Change one, change the other.
//
// The real sidebar is position: sticky with height: 100vh; stories pin a fixed
// height inline so the frame stays readable.

const viewItem = ({ label, count, active, href = '#' }) =>
  active
    ? `<span class="item active">${label}${count != null ? `<span class="count">${count}</span>` : ''}</span>`
    : `<a class="item" href="${href}">${label}${count != null ? `<span class="count">${count}</span>` : ''}</a>`;

const tagItem = ({ title, count }) => `
  <a class="item" href="#"><span class="tag-name"><span class="syntax">#[[</span>${title}<span class="syntax">]]</span></span><span class="count">${count}</span></a>`;

const sidebar = ({ activeView, results = [] }) => `
  <aside class="sidebar" style="height: 640px; position: static;">
    <a class="brand" href="#">
      <span class="mark">[[&nbsp;]]</span>
      <span class="name">Scraps Doc</span>
    </a>
    <div class="search">
      <input type="search" id="search-input" placeholder="Search by title..." autocomplete="off" />
      ${
        results.length
          ? `<ul id="search-results">${results
              .map((r) => `<li><a href="#">${r}</a></li>`)
              .join('')}</ul>`
          : '<ul id="search-results"></ul>'
      }
    </div>
    <nav class="views">
      <p class="nav-label">views</p>
      ${viewItem({ label: 'updated', active: activeView === 'updated' })}
      ${viewItem({ label: 'backlinks', active: activeView === 'backlinks' })}
      ${viewItem({ label: 'titles', active: activeView === 'titles' })}
      ${viewItem({ label: 'all scraps', count: 764, active: activeView === 'scraps' })}
    </nav>
    <nav class="tags">
      <p class="nav-label">tags</p>
      ${[
        { title: 'Security', count: 170 },
        { title: 'Programming', count: 87 },
        { title: 'Agile', count: 67 },
        { title: 'Cloud Native', count: 63 },
        { title: 'Documentation', count: 38 },
      ]
        .map(tagItem)
        .join('')}
      <a class="item more" href="#">+ 36 more</a>
    </nav>
    <p class="version">scraps v2.0.0</p>
  </aside>`;

export default {
  title: 'Shell/Sidebar',
  parameters: {
    docs: {
      description: {
        component:
          'The persistent sidebar: brand, search, sort views, tags. No file tree and no graph pane — ctx stays a row prefix, never navigation.',
      },
    },
  },
};

export const Default = {
  render: () => sidebar({ activeView: 'updated' }),
};

export const SearchOpen = {
  name: 'Search with results',
  render: () =>
    sidebar({
      activeView: 'updated',
      results: ['Wiki-link Notation', 'Configuration', 'Lint Rules'],
    }),
};

export const TitlesActive = {
  name: 'Titles view active',
  render: () => sidebar({ activeView: 'titles' }),
};
