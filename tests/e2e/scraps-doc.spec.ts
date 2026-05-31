import { test, expect, type Page } from '@playwright/test';

/*
 * These E2E tests exercise the *wiring* of a scraps-generated site (home,
 * search box, code-highlight hook, OGP cards) against the local `scraps serve`.
 *
 * The generated pages pull their JS from third-party CDNs (jsdelivr for Fuse +
 * mermaid, cdnjs for highlight.js) and fetch OGP data through corsproxy.io.
 * Hanging the suite on those services made CI red for weeks: requests to them
 * intermittently *stall* on GitHub-hosted runners, and since a page waits on
 * its <script> resources (the classic highlight.js tag even blocks
 * DOMContentLoaded), a single stalled request makes a navigation hang until the
 * test timeout. With mermaid alone pulling 60+ transitive ESM modules from
 * jsdelivr per page, the odds of a run completing without one stall approached
 * zero, so every run hit the 15-minute job cap.
 *
 * Fix: intercept every external request and fulfill it locally, making the
 * suite hermetic and fast regardless of CDN/runner network weather. The real
 * "Fuse must load as ESM" contract is covered by the Rust regression test in
 * src/usecase/build/html/index_render.rs; here we assert that scraps emits
 * pages that correctly wire up search / highlight / OGP.
 */

// Minimal stand-in for Fuse.js (ESM). index.html does
// `new Fuse(data, { keys: ["title"] })` then `fuse.search(q)`, expecting
// `[{ item }, ...]`. A substring match over the configured keys is enough.
const FUSE_STUB = `
export default class Fuse {
  constructor(list = [], options = {}) {
    this._list = list;
    this._keys = (options.keys || []).map((k) => (typeof k === 'string' ? k : k.name));
  }
  search(query) {
    const q = String(query).toLowerCase().trim();
    if (!q) return [];
    return this._list
      .filter((item) => this._keys.some((key) => String(item?.[key] ?? '').toLowerCase().includes(q)))
      .map((item) => ({ item }));
  }
}
`;

// mermaid is imported but never invoked by the template, so any object works.
// A Proxy returning no-op functions keeps it safe if a call is ever added.
const MERMAID_STUB = `export default new Proxy({}, { get: () => () => {} });`;

// highlight.js is a classic <script> followed by an inline `hljs.highlightAll()`
// call, so the stub must expose those globals synchronously.
const HLJS_STUB = `window.hljs = { highlightAll() {}, highlightElement() {} };`;

// Canned OGP document returned in place of the corsproxy.io response for
// <https://github.com/boykush/scraps>.
const OGP_HTML = `<!doctype html><html><head>
<meta property="og:title" content="GitHub - boykush/scraps: The Wiki-link doc compiler for the LLM era" />
<meta property="og:description" content="The Wiki-link doc compiler for the LLM era - boykush/scraps" />
<meta property="og:image" content="https://repository-images.githubusercontent.com/000000000/scraps.png" />
</head><body></body></html>`;

// Route every external request to a local response so no test ever depends on
// (or stalls on) a third-party service. Requests served by `scraps serve` on
// 127.0.0.1 are left untouched.
async function mockExternalRequests(page: Page) {
  await page.route('**/*', async (route) => {
    const url = route.request().url();

    if (url.includes('127.0.0.1') || url.includes('localhost')) {
      return route.continue();
    }
    if (url.includes('cdn.jsdelivr.net/npm/fuse.js')) {
      return route.fulfill({ contentType: 'text/javascript', body: FUSE_STUB });
    }
    if (url.includes('cdn.jsdelivr.net/npm/mermaid')) {
      return route.fulfill({ contentType: 'text/javascript', body: MERMAID_STUB });
    }
    if (url.includes('cdnjs.cloudflare.com') && url.includes('highlight')) {
      return route.fulfill({ contentType: 'text/javascript', body: HLJS_STUB });
    }
    if (url.includes('corsproxy.io')) {
      return route.fulfill({ contentType: 'text/html', body: OGP_HTML });
    }
    // Fonts, theme CSS, transitive ESM deps, anything else external: succeed
    // empty so nothing reaches the network and nothing can block a load.
    return route.fulfill({ status: 200, contentType: 'text/css', body: '' });
  });
}

test.beforeEach(async ({ page }) => {
  await mockExternalRequests(page);
});

test('get home', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/Scraps Doc/);

  const readme_content = await page.locator('[class="readme-block"]').textContent();
  expect(readme_content).toContain('What is Scraps?');
});

test('search scraps', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });

  // The search handler is wired up by a module script after the index is
  // fetched; wait for it before driving the input.
  await page.waitForFunction(() => typeof (window as any).search === 'function');

  // Fill the [id="search-input"] input.
  await page.locator('[id="search-input"]').fill('What is');

  // Press Enter.
  await page.keyboard.press('Enter');

  // Expect the search results to contain "What is Scraps?" (auto-retries).
  await expect(page.locator('[id="search-results"]')).toContainText('What is Scraps?');
});

test('CDN libraries are loaded', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });

  // highlight.js (classic script) and Fuse.js (module script) are wired up
  // asynchronously, so poll instead of reading once.
  await expect
    .poll(() => page.evaluate(() => typeof (window as any).hljs !== 'undefined'))
    .toBe(true);
  await expect
    .poll(() => page.evaluate(() => typeof (window as any).Fuse !== 'undefined'))
    .toBe(true);
});

test('fetch OGP data', async ({ page }) => {
  // Autolink.md intentionally hosts a live `<https://github.com/boykush/scraps>`
  // autolink so this test has a stable OGP card to assert against.
  // Keep that autolink in `docs/Reference/Markdown/Autolink.md` if you edit it.
  await page.goto('/scraps/reference/markdown/autolink.html', { waitUntil: 'domcontentloaded' });

  // Wait for OGP card to be present
  const ogpCard = page.locator('.ogp-card').first();
  await expect(ogpCard).toBeVisible();

  // Wait for OGP data to be loaded (max 5 seconds)
  await expect(async () => {
    const titleText = await ogpCard.locator('.ogp-title').textContent();
    expect(titleText).not.toBeNull();
    expect(titleText).not.toBe('Loading...');
  }).toPass({
    timeout: 5000,
  });

  // Verify GitHub repository OGP data
  const title = await ogpCard.locator('.ogp-title').textContent();
  const description = await ogpCard.locator('.ogp-description').textContent();

  expect(title).toContain('GitHub - boykush/scraps');
  expect(description).toContain('The Wiki-link doc compiler for the LLM era');

  // Verify image is loaded
  const image = ogpCard.locator('.ogp-image');
  const imageSrc = await image.getAttribute('src');
  expect(imageSrc).toContain('https://repository-images.githubusercontent.com/');
});
