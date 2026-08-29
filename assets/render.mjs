// Renders the brand PNGs from assets/logo.svg so the raster assets never drift
// from their vector source. Run with `mise run assets:build`.
//
// Playwright is borrowed from the e2e suite rather than added as a second
// dependency; it is the only headless browser the repo already installs.
import { chromium } from '../tests/e2e/node_modules/playwright/index.mjs';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ASSETS = dirname(fileURLToPath(import.meta.url));

const MARK = readFileSync(join(ASSETS, 'logo.svg'), 'utf8').replace(
  / width="640" height="640"/,
  ' width="100%" height="100%"',
);

const TAGLINE = 'The Wiki-link doc compiler for the LLM era.';
const SURFACE = '#2e3440';
const TEXT = '#eceff4';
const TEXT_MUTED = '#81a1c1';
const MONO = 'ui-monospace,SFMono-Regular,Menlo,Consolas,monospace';

const page = (w, h, body, bg) => `<!doctype html><html><head><meta charset="utf-8">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Rubik:wght@400;500;700&display=swap">
<style>html,body{margin:0;padding:0}body{width:${w}px;height:${h}px;background:${bg};overflow:hidden}</style>
</head><body>${body}</body></html>`;

const LOGO = page(640, 640, `<div style="width:640px;height:640px">${MARK}</div>`, 'transparent');

const SOCIAL = page(
  1280,
  640,
  `<div style="width:1280px;height:640px;box-sizing:border-box;background:${SURFACE};display:flex;flex-direction:column;align-items:center;justify-content:center;gap:40px">
  <div style="width:176px;height:176px">${MARK}</div>
  <div style="display:flex;flex-direction:column;align-items:center;gap:22px">
    <div style="font-family:'Rubik',sans-serif;font-weight:700;font-size:96px;line-height:1.25;letter-spacing:-0.03em;color:${TEXT}">Scraps</div>
    <div style="font-family:${MONO};font-size:30px;line-height:1.4;color:${TEXT_MUTED}">${TAGLINE}</div>
  </div>
</div>`,
  SURFACE,
);

// The `_opacity` twins predate the plated mark, which carries its own
// background; they are written identically so existing references keep working.
const JOBS = [
  { html: LOGO, width: 640, height: 640, omitBackground: true, out: ['logo.png', 'logo_opacity.png'] },
  {
    html: SOCIAL,
    width: 1280,
    height: 640,
    omitBackground: false,
    out: ['social_preview.png', 'social_preview_opacity.png'],
  },
];

const browser = await chromium.launch();
try {
  for (const job of JOBS) {
    const tab = await browser.newPage({
      viewport: { width: job.width, height: job.height },
      deviceScaleFactor: 1,
    });
    await tab.setContent(job.html, { waitUntil: 'load' });
    await tab.evaluate(() => document.fonts.ready);
    const png = await tab.screenshot({ omitBackground: job.omitBackground });
    for (const name of job.out) writeFileSync(join(ASSETS, name), png);
    console.log(`${job.width}x${job.height} -> ${job.out.join(', ')}`);
    await tab.close();
  }
} finally {
  await browser.close();
}
