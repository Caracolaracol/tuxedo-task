import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://caracolaracol.github.io',
  base: '/tuxedo-task',
  output: 'static',
  trailingSlash: 'always',
  build: { inlineStylesheets: 'never' },
});
