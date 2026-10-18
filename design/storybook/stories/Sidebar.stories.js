// The persistent shell sidebar: src/usecase/build/html/builtins/base.html and
// the aside.sidebar block in main.css. Markup duplicated deliberately — the
// templates are Tera, rendered by the Rust binary. Change one, change the other.
//
// The real sidebar is position: sticky with height: 100vh; stories pin a fixed
// height inline so the frame stays readable.

const scope = ({ name, count, depth = 0, tag = true, current, zero }) => `
  <a class="item${depth ? ' nested' : ''}${zero ? ' zero' : ''}"${zero ? '' : ' href="#"'}${current ? ' aria-current="page"' : ''} style="--depth: ${depth}">${
    tag
      ? `<span class="tag-name"><span class="syntax">#[[</span>${name}<span class="syntax">]]</span></span>`
      : `<span class="scope-name">${name}</span>`
  }<span class="count">${zero ? 0 : count}</span></a>`;

const sidebar = ({ activeView, current = 'all', narrowed = false, results = [] }) => `
  <aside class="sidebar" style="height: 640px; position: static;">
    <div class="brand-block">
      <a class="brand" href="#">
        <span class="mark">[[&nbsp;]]</span>
        <span class="name">Scraps Doc</span>
      </a>
      ${activeView === 'about' ? '<span class="about active">about</span>' : '<a class="about" href="#">about</a>'}
    </div>
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
    <nav class="scopes" aria-label="Scopes">
      ${scope({ name: 'all scraps', count: 764, tag: false, current: current === 'all' })}
      <p class="nav-label">tags</p>
      ${[
        { name: 'Security', count: 170 },
        { name: 'Authentication', count: 40, depth: 1 },
        { name: 'Cryptography', count: 20, depth: 1 },
        { name: 'Programming', count: 87 },
        { name: 'Agile', count: 67, zero: narrowed },
        { name: 'Cloud Native', count: 63 },
      ]
        .map((entry) => scope({ ...entry, current: current === entry.name }))
        .join('')}
    </nav>
    <p class="version">scraps v2.0.0</p>
  </aside>`;

export default {
  title: 'Shell/Sidebar',
  parameters: {
    docs: {
      description: {
        component:
          'The persistent sidebar: brand, search, and the scopes a list can show — the whole wiki, then every tag as a tree. No file tree and no graph pane — ctx stays a row prefix, never navigation.',
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

export const TagScopeNarrowed = {
  name: 'Tag scope, narrowed to a type',
  parameters: {
    docs: {
      description: {
        story: 'On a list narrowed to a type, every count is that type\'s, and a tag holding none of it cannot be picked.',
      },
    },
  },
  render: () => sidebar({ activeView: 'updated', current: 'Security', narrowed: true }),
};
