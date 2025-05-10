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
  const searchResults = await page.locator('[id="search-results"]').innerHTML();
  expect(searchResults).toContain('What is Scraps?');
});