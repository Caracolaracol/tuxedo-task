import { copyFile, mkdir } from 'node:fs/promises';

// Keep the website's branding and real application capture in sync with Rust.
const assets = [
  ['../../src/serve/assets/icon.svg', '../public/brand.svg'],
  ['../../Docs/screenshots/subtasks.svg', '../public/screenshots/subtasks.svg'],
  ['../../Docs/screenshots/list.svg', '../public/screenshots/list.svg'],
];

for (const [source, destination] of assets) {
  const target = new URL(destination, import.meta.url);
  await mkdir(new URL('.', target), { recursive: true });
  await copyFile(new URL(source, import.meta.url), target);
}
