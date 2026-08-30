import { test, expect, type Page } from '@playwright/test';

/*
 * Lightweight browser smoke tests for a scraps-generated site, run against the
 * local `scraps serve`.
 *
 * The generated pages pull JS from third-party CDNs (jsdelivr for Fuse +
 * mermaid, cdnjs for highlight.js). Those requests intermittently *stall* on CI
 * runners, and since a navigation waits on its <script> resources (the classic
 * highlight.js tag even blocks DOMContentLoaded) a single stall hangs the whole
 * job. So we intercept every external request and answer it locally: the suite
 * becomes hermetic and never depends on CDN/runner network weather.
 *
 * We deliberately keep the set small — a serve+render smoke test and the search
 * wiring. Purely presentational / external-fetch checks (the old "CDN libraries
 * are loaded" and "fetch OGP data" cases) were dropped: the first was a
 * tautology once the libs are stubbed, and OGP card markup / the ESM-only Fuse
 * contract are better covered by Rust tests (e.g.
 * src/usecase/build/html/index_render.rs).
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

// Stub every external request so nothing reaches the network (and nothing can
// stall a navigation). Fuse needs a working stand-in; mermaid and highlight.js
// just need to exist so their import / inline `hljs.highlightAll()` don't error.
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
      return route.fulfill({ contentType: 'text/javascript', body: 'export default {};' });
    }
    if (url.includes('cdnjs.cloudflare.com') && url.includes('highlight')) {
      return route.fulfill({ contentType: 'text/javascript', body: 'window.hljs = { highlightAll() {} };' });
    }
    // Fonts, theme CSS, anything else external: empty 200 so nothing blocks.
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

test('sort views are always generated', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.view-nav .view.active')).toHaveText('updated');

  await page.goto('/backlinks/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.view-nav .view.active')).toHaveText('backlinks');

  await page.goto('/titles/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.jump-bar .jump').first()).toBeVisible();
  await expect(page.locator('section.title-group').first()).toBeVisible();
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
