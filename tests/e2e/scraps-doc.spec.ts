import { test, expect } from '@playwright/test';

test('has metadata', async ({ page }, testInfo) => {
  await page.goto('http://localhost:1112/');
  await page.screenshot({
    path: testInfo.outputPath('home.png'),
    fullPage: true,
  });

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/Scraps Doc/);
});
