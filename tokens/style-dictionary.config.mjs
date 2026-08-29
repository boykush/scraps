import StyleDictionary from 'style-dictionary';

const OUT = '../src/usecase/build/css/builtins/_tokens.css';

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

export default {
  source: ['*.tokens.json'],
  platforms: {
    css: {
      // The format computes its own names; this only keeps Style Dictionary's
      // internal names unique so genuine collisions still get reported.
      transforms: ['name/kebab'],
      files: [{ destination: OUT, format: 'scraps/css-custom-properties' }],
    },
  },
};
