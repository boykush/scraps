import { test, expect, type Page } from '@playwright/test';

/*
 * Lightweight browser smoke tests for a scraps-generated site, run against the
 * local `scraps serve`.
 *
 * The site serves its own scripts, but the pages still link web fonts from
 * third-party hosts. Those requests intermittently *stall* on CI runners, and a
 * navigation waits on its stylesheets, so a single stall hangs the whole job.
 * So we answer every external request locally: the suite becomes hermetic and
 * never depends on runner network weather.
 */
async function mockExternalRequests(page: Page) {
  await page.route('**/*', async (route) => {
    const url = route.request().url();
    if (url.includes('127.0.0.1') || url.includes('localhost')) {
      return route.continue();
    }
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

  // The home is a designed listing — the README renders at /about/ instead.
  await expect(page.locator('div.index .links-block')).toBeVisible();
  await expect(page.locator('.readme-block')).toHaveCount(0);
});

test('readme renders at its own about page', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await page.locator('.sidebar .brand-block a.about').click();

  await expect(page).toHaveURL(/\/about\/$/);
  const about_content = await page.locator('div.about .content').textContent();
  expect(about_content).toContain('What is Scraps?');
});

test('sort views are always generated', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.sidebar nav.views .item.active')).toHaveText(/updated/);

  await page.goto('/backlinks/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.sidebar nav.views .item.active')).toHaveText(/backlinks/);

  await page.goto('/all-scraps/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.sidebar nav.views .item.active')).toHaveText(/all scraps/);
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

test('diagrams and code are rendered by the libraries the site serves', async ({ page }) => {
  await page.goto('/scraps/Reference/Static%20Site/Pipeline.html', { waitUntil: 'domcontentloaded' });

  await expect(page.locator('code.mermaid svg')).toBeVisible();

  await page.goto('/scraps/Reference/Markdown/Mermaid.html', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('pre code.hljs').first()).toBeVisible();
});

test('pages are served at their names as written', async ({ page }) => {
  await page.goto('/all-scraps/', { waitUntil: 'domcontentloaded' });

  // A title with a space and a `?`: the link is percent-encoded, and the dev
  // server has to decode it back to the file on disk.
  await page.locator('.scrap-index a.entry', { hasText: 'What is Scraps?' }).click();
  await expect(page).toHaveURL(/\/scraps\/Explanation\/What%20is%20Scraps%3F\.html$/);
  await expect(page.locator('h1.title')).toHaveText('What is Scraps?');

  // A type's views live under its name, and a tag's page under its own.
  await page.locator('p.type a.name').click();
  await expect(page).toHaveURL(/\/types\/Explanation\/$/);
  await expect(page.locator('.type-filter .option.active')).toContainText('Explanation');

  await page.goto('/all-tags/', { waitUntil: 'domcontentloaded' });
  await page.locator('.tag-links a.link').first().click();
  await expect(page).toHaveURL(/\/tags\/.+\.html$/);
  await expect(page.locator('section.connections .scrap-links')).toBeVisible();
});
