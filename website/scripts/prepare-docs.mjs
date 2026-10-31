import { mkdir, writeFile } from 'node:fs/promises';
import { buildManual, loadGuides } from './manual.mjs';

const target = new URL('../src/generated/manual.md', import.meta.url);
await mkdir(new URL('.', target), { recursive: true });
await writeFile(target, buildManual(await loadGuides()));
