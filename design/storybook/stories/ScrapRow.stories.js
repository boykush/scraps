// Mirrors the `scrap_rows` component in
// src/usecase/build/html/builtins/macros.html. The markup is duplicated
// deliberately: the templates are Tera, rendered by the Rust binary, so there
// is no way to mount them here. Change one, change the other.

const row = ({ title, ctx, summary, refs = 2, backlinks = 3, thumbnail }) => `
  <li class="item">
    <a class="row" href="#">
      ${thumbnail ? `<img class="thumbnail" alt="" src="${thumbnail}" />` : ''}
      <span class="body">
        <span class="head">
          <span class="title">${title}</span>
          ${ctx ? `<span class="ctx">${ctx}</span>` : ''}
        </span>
        ${summary ? `<span class="summary">${summary}</span>` : ''}
      </span>
      <span class="graph">
        <span class="metric">${refs} ref${refs === 1 ? '' : 's'}</span>
        <span class="metric">${backlinks} backlink${backlinks === 1 ? '' : 's'}</span>
      </span>
    </a>
  </li>`;

const list = (rows) => `<ul class="scrap-rows">${rows.map(row).join('')}</ul>`;

const THUMB =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40"><rect width="40" height="40" rx="4" fill="%235e81ac"/></svg>',
  );

export default {
  title: 'Index/Scrap row',
  parameters: {
    docs: {
      description: {
        component:
          'One entry in the wiki index. Carries the ctx beside the title where it disambiguates, and the outbound and inbound link counts on the right.',
      },
    },
  },
};

export const WithNamespace = {
  render: () =>
    list([
      {
        title: 'Wiki-link Notation',
        ctx: 'Reference',
        summary: 'Wiki-link notation gives Markdown a typed surface: each reference resolves, lints and emits.',
        refs: 9,
        backlinks: 4,
      },
    ]),
};

export const WithoutNamespace = {
  render: () =>
    list([
      {
        title: 'Getting Started',
        summary: 'Install, create a wiki, compile it, then query the same source from a shell.',
        refs: 6,
        backlinks: 3,
      },
    ]),
};

export const WithoutSummary = {
  render: () => list([{ title: 'Pod', ctx: 'Kubernetes', refs: 1, backlinks: 1 }]),
};

export const WithThumbnail = {
  render: () =>
    list([
      {
        title: 'Static Site',
        ctx: 'Reference',
        summary: 'The static site is one of the emit targets Scraps compiles to.',
        thumbnail: THUMB,
      },
    ]),
};

export const LongTitleAndSummary = {
  render: () =>
    list([
      {
        title: 'A Deliberately Long Scrap Title That Has To Be Truncated Somewhere',
        ctx: 'Reference/Static Site',
        summary:
          'A summary long enough that it has to be cut rather than allowed to wrap, because the row keeps its rhythm and the title above it is already competing for the same width.',
        refs: 12,
        backlinks: 8,
      },
    ]),
};

export const AsAList = {
  name: 'A full list',
  render: () =>
    list([
      { title: 'Alias', ctx: 'Reference/Wiki-link', summary: 'Display shows custom display text while linking to Title.', refs: 1, backlinks: 2 },
      { title: 'Section Embed', ctx: 'Reference/Wiki-link', summary: 'Title#Heading embeds a single section from another scrap.', refs: 2, backlinks: 2 },
      { title: 'Tag', ctx: 'Reference/Wiki-link', summary: '#tag marks a tag. Tags and scraps live in separate namespaces.', refs: 1, backlinks: 1 },
      { title: 'Configuration', ctx: 'Reference', summary: '.scraps.toml declares a Scraps wiki: the directory containing this file is the wiki root.', refs: 4, backlinks: 9 },
    ]),
};
