import StyleDictionary from 'style-dictionary';

const OUT = '../src/usecase/build/css/builtins/_tokens.css';
const DOC = 'dist/index.html';

/// Semantic colours pair a light and a dark value under one role. CSS
/// light-dark() keeps that in a single stylesheet, which is why the built-in
/// css/variables format cannot be used here.
const MODES = ['light', 'dark'];

function cssName(path) {
  const [head, ...rest] = path;
  if (head === 'color' && rest[0] === 'nord') return `--nord${rest[1]}`;
  if (head === 'color' && rest[0] === 'ext') return `--ext-${rest.slice(1).join('-')}`;
  if (head === 'color') return `--color-${rest.filter((p) => !MODES.includes(p)).join('-')}`;
  return `--${path.join('-')}`;
}

/// Generic families must stay unquoted to keep their keyword meaning;
/// everything else is a family name and is quoted.
const GENERIC_FAMILIES = new Set([
  'serif', 'sans-serif', 'monospace', 'cursive', 'fantasy', 'system-ui',
  'ui-serif', 'ui-sans-serif', 'ui-monospace', 'ui-rounded', 'math', 'emoji',
]);

function groupOf(path, name) {
  if (name.startsWith('--nord') || name.startsWith('--ext-')) return 'primitive';
  if (path[0] === 'color') return 'color';
  return path.slice(0, -1).join('-');
}

function literal(token) {
  const raw = token.original?.$value ?? token.$value;
  if (typeof raw === 'string' && raw.startsWith('{')) {
    return `var(${cssName(raw.slice(1, -1).split('.'))})`;
  }
  const value = token.$value;
  if (Array.isArray(value)) {
    return value.map((f) => (GENERIC_FAMILIES.has(f) ? f : `"${f}"`)).join(', ');
  }
  if (value && typeof value === 'object' && 'unit' in value) {
    return `${value.value}${value.unit}`;
  }
  return String(value);
}

function walk(node, path, out) {
  for (const [key, child] of Object.entries(node)) {
    if (key.startsWith('$') || typeof child !== 'object') continue;
    if ('$value' in child) out.push([[...path, key], child]);
    else walk(child, [...path, key], out);
  }
}

StyleDictionary.registerFormat({
  name: 'scraps/css-custom-properties',
  format: ({ dictionary }) => {
    const flat = [];
    walk(dictionary.tokens, [], flat);

    const lines = [
      '    /* Generated from tokens/*.tokens.json — do not edit. `mise run tokens:build` */',
      '',
    ];
    const emitted = new Set();
    let group = null;

    for (const [path, token] of flat) {
      const name = cssName(path);
      if (emitted.has(name)) continue;

      const nextGroup = groupOf(path, name);
      if (group !== null && nextGroup !== group) lines.push('');
      group = nextGroup;

      const mode = path[path.length - 1];
      if (MODES.includes(mode)) {
        const pair = MODES.map((m) => {
          const sibling = flat.find(
            ([p]) => cssName(p) === name && p[p.length - 1] === m,
          );
          return literal(sibling[1]);
        });
        lines.push(`    ${name}: light-dark(${pair.join(', ')});`);
      } else {
        lines.push(`    ${name}: ${literal(token)};`);
      }
      emitted.add(name);
    }

    return lines.join('\n') + '\n';
  },
});

/// WCAG relative luminance, so the generated page can state the contrast a
/// role actually achieves rather than asserting it passes.
function luminance(hex) {
  const raw = hex.replace('#', '');
  const channels = [0, 2, 4]
    .map((i) => parseInt(raw.slice(i, i + 2), 16) / 255)
    .map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(a, b) {
  const [la, lb] = [luminance(a), luminance(b)];
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

function swatch(name) {
  return `  <div style="border: var(--border-hairline) solid var(--color-rule); border-radius: var(--radius-md); overflow: hidden;">
    <div style="height: 48px; background: var(${name});"></div>
    <div style="padding: var(--space-2) var(--space-3); border-top: var(--border-hairline) solid var(--color-rule); font-family: var(--font-family-mono); font-size: var(--font-size-xs); color: var(--color-text-muted);">${name}</div>
  </div>`;
}

function grid(inner, min = 190) {
  return `<div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(${min}px, 1fr)); gap: var(--space-3);">
${inner}
</div>`;
}

StyleDictionary.registerFormat({
  name: 'scraps/tokens-doc',
  format: ({ dictionary }) => {
    const flat = [];
    walk(dictionary.tokens, [], flat);

    const byName = new Map();
    for (const [path, token] of flat) {
      const name = cssName(path);
      const mode = path[path.length - 1];
      if (!byName.has(name)) byName.set(name, { path, modes: {}, token });
      if (MODES.includes(mode)) byName.get(name).modes[mode] = token.$value;
    }

    const semantic = [...byName.entries()].filter(([n, e]) => n.startsWith('--color-') && Object.keys(e.modes).length);
    const primitives = [...byName.keys()].filter((n) => n.startsWith('--nord') || n.startsWith('--ext-'));
    const scale = (prefix) => [...byName.entries()].filter(([n]) => n.startsWith(prefix));

    const surface = byName.get('--color-surface').modes;
    const raised = byName.get('--color-surface-raised').modes;
    const rows = semantic.map(([name, entry]) => {
      // Syntax roles are read on the code ground, so quote them against it —
      // the same pairing tokens/check-contrast.py gates.
      const ground = name.startsWith('--color-syntax-') ? raised : surface;
      const ratio = (mode) => {
        if (name === '--color-surface') return '&#8212;';
        return `${contrast(entry.modes[mode], ground[mode]).toFixed(2)}:1`;
      };
      return `<tr><td><code>${name}</code></td><td><code>${entry.modes.light}</code></td><td><code>${entry.modes.dark}</code></td><td>${ratio('light')}</td><td>${ratio('dark')}</td></tr>`;
    });

    const typeRows = scale('--font-size-').map(([name, e]) =>
      `  <div style="display: flex; align-items: baseline; gap: var(--space-4);"><span style="width: 140px; flex-shrink: 0; font-family: var(--font-family-mono); font-size: var(--font-size-xs); color: var(--color-text-muted);">${name}</span><span style="font-size: var(${name}); line-height: var(--font-line-height-tight);">Typed source</span></div>`);

    const spaceBars = scale('--space-').map(([name]) =>
      `  <div style="width: var(${name}); height: 30px; background: var(--color-accent); border-radius: var(--radius-sm);" title="${name}"></div>`);

    const radiusBoxes = scale('--radius-').map(([name]) =>
      `  <div style="width: 62px; height: 44px; background: var(--color-surface-raised); border: var(--border-hairline) solid var(--color-rule); border-radius: var(${name});" title="${name}"></div>`);

    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<meta name="robots" content="noindex">
<title>Design Tokens — Scraps</title>
<!-- The theme's own stylesheet, one level up on the deployed site, so this
     page shows what the tokens actually produce rather than a copy of it. -->
<link rel="stylesheet" href="../main.css">
<link href="https://fonts.cdnfonts.com/css/rubik" rel="stylesheet">
</head>
<body>
<header><h1>Design Tokens</h1></header>
<main>
<div class="scrap">
<div class="content">

<p>The design tokens Scraps compiles its theme from. Generated from
<code>tokens/*.tokens.json</code> by <code>mise run tokens:build</code>; edit the
token files, not this page.</p>

<p>Swatches reference the live CSS custom properties, so they follow whichever
colour scheme you are reading in. The table states resolved values and the
contrast each role reaches against its surface, which
<code>tokens/check-contrast.py</code> gates at WCAG AA.</p>

<h2>Semantic colours</h2>

${grid(semantic.map(([n]) => swatch(n)).join('\n'))}

<table>
<thead><tr><th>Role</th><th>Light</th><th>Dark</th><th>Contrast light</th><th>Contrast dark</th></tr></thead>
<tbody>
${rows.join('\n')}
</tbody>
</table>

<h2>Primitives</h2>

<p>Raw palette values. Nothing outside the semantic layer should reference these.</p>

${grid(primitives.map(swatch).join('\n'), 150)}

<h2>Type scale</h2>

<div style="display: flex; flex-direction: column; gap: var(--space-2);">
${typeRows.join('\n')}
</div>

<h2>Space</h2>

<div style="display: flex; align-items: flex-end; gap: var(--space-3);">
${spaceBars.join('\n')}
</div>

<h2>Radius</h2>

<div style="display: flex; gap: var(--space-3);">
${radiusBoxes.join('\n')}
</div>

</div>
</div>
</main>
</body>
</html>
`;
  },
});

export default {
  source: ['*.tokens.json'],
  platforms: {
    css: {
      // The format computes its own names; this only keeps Style Dictionary's
      // internal names unique so genuine collisions still get reported.
      transforms: ['name/kebab'],
      files: [
        { destination: OUT, format: 'scraps/css-custom-properties' },
        { destination: DOC, format: 'scraps/tokens-doc' },
      ],
    },
  },
};
