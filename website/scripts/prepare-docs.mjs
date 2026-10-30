import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { buildManual } from './manual.mjs';

const [readme, start, notes] = await Promise.all([
  readFile(new URL('../../README.md', import.meta.url), 'utf8'),
  readFile(new URL('../src/content/start.md', import.meta.url), 'utf8'),
  readFile(new URL('../src/content/reference-notes.md', import.meta.url), 'utf8'),
]);
const target = new URL('../src/generated/manual.md', import.meta.url);
await mkdir(new URL('.', target), { recursive: true });
await writeFile(target, buildManual(readme, start, notes));
