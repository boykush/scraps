/** @type { import('@storybook/html-vite').StorybookConfig } */
export default {
  stories: ['../stories/**/*.stories.js'],
  addons: ['@storybook/addon-a11y'],
  staticDirs: ['../static'],
  framework: { name: '@storybook/html-vite', options: {} },
  // This runs in CI for an OSS repo; nothing here needs reporting upstream.
  core: { disableTelemetry: true },
};
