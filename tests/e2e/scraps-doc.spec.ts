import { test, expect } from '@playwright/test';

test('has title', async ({ page }) => {
  await page.goto('/');

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/Scraps Doc/);
});

test('search scraps', async ({ page }) => {
  await page.goto('/');

  // Fill the [id="search-input"] input.
  await page.locator('[id="search-input"]').fill('What is');

  // Press Enter.
  await page.keyboard.press('Enter');

  // Expect the search results to contain "What is Scraps?".
  const searchResults = await page.locator('[id="search-results"]').textContent();
  expect(searchResults).toContain('What is Scraps?');
});

test('fetch OGP data', async ({ page }) => {
  await page.goto('/scraps/autolink.html');

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
  expect(description).toContain('Scraps is a static site generator');

  // Verify image is loaded
  const image = ogpCard.locator('.ogp-image');
  const imageSrc = await image.getAttribute('src');
  expect(imageSrc).toContain('https://repository-images.githubusercontent.com/');
});