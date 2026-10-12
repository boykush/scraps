// Copies each library's browser build into the crate, where include_bytes!
// picks it up. Run with `mise run jslibs:build`.
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, '../src/usecase/build/js/builtins');

// Every file is emitted as .js: static hosts do not all serve .mjs with a
// JavaScript MIME type, and a module script refuses anything else.
const FILES = [
  ['@highlightjs/cdn-assets/highlight.min.js', 'highlight.min.js'],
  ['@highlightjs/cdn-assets/LICENSE', 'highlight.LICENSE'],
  ['fuse.js/dist/fuse.min.mjs', 'fuse.min.js'],
  ['fuse.js/LICENSE', 'fuse.LICENSE'],
  ['mermaid/dist/mermaid.min.js', 'mermaid.min.js'],
  ['mermaid/LICENSE', 'mermaid.LICENSE'],
];

mkdirSync(OUT, { recursive: true });
for (const [from, to] of FILES) {
  copyFileSync(join(HERE, 'node_modules', from), join(OUT, to));
  console.log(`✔︎ ${to}`);
}
