// Two variants of the stylesheet the build emits, derived by flatten-scheme.py
// and copied in by `mise run design:build`. Stories are markup only: every
// colour, size and spacing they show comes from the real theme.
//
// The variants exist because a `light-dark()` inside a custom property is not
// re-resolved when `color-scheme` changes at runtime, so a toggle cannot drive
// the shipped single stylesheet. Swapping the sheet can.
const SHEET_ID = 'scraps-theme';

function applyScheme(scheme) {
  let link = document.getElementById(SHEET_ID);
  if (!link) {
    link = document.createElement('link');
    link.id = SHEET_ID;
    link.rel = 'stylesheet';
    document.head.append(link);
  }
  const href = `./main.${scheme}.css`;
  if (link.getAttribute('href') !== href) link.setAttribute('href', href);
  document.documentElement.style.colorScheme = `only ${scheme}`;
}

/** @type { import('@storybook/html-vite').Preview } */
const preview = {
  globalTypes: {
    scheme: {
      description: 'Colour scheme',
      toolbar: {
        title: 'Scheme',
        icon: 'contrast',
        items: [
          { value: 'light', title: 'Light' },
          { value: 'dark', title: 'Dark' },
        ],
        dynamicTitle: true,
      },
    },
  },
  initialGlobals: { scheme: 'light' },
  decorators: [
    (story, context) => {
      applyScheme(context.globals.scheme ?? 'light');
      const wrap = document.createElement('div');
      wrap.style.cssText =
        'background: var(--color-surface); color: var(--color-text);' +
        'font-family: var(--font-family-sans); padding: var(--space-5);';
      const node = story();
      wrap.append(
        typeof node === 'string' ? document.createRange().createContextualFragment(node) : node,
      );
      return wrap;
    },
  ],
  parameters: {
    layout: 'fullscreen',
    controls: { expanded: true },
  },
};

export default preview;
