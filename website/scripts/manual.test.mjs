import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { buildManual, readmeSection } from './manual.mjs';

const [readme, start, notes] = await Promise.all([
  readFile(new URL('../../README.md', import.meta.url), 'utf8'),
  readFile(new URL('../src/content/start.md', import.meta.url), 'utf8'),
  readFile(new URL('../src/content/reference-notes.md', import.meta.url), 'utf8'),
]);

test('section extraction keeps subheadings and ignores headings inside code fences', () => {
  const source = '# Title\n\n## Guide\nKeep this.\n### Child\n```text\n## Not a heading\n```\n\n## Next\nSkip this.\n';
  const actual = readmeSection(source, 'Guide');
  assert.ok(actual.includes('### Child'));
  assert.ok(actual.includes('## Not a heading'));
  assert.ok(!actual.includes('Skip this.'));
});

test('missing source sections fail loudly instead of publishing incomplete docs', () => {
  assert.throws(() => readmeSection(readme, 'Missing section'), /Documentation source missing/);
  assert.throws(() => buildManual(readme.replace('## Configuration', '## Config removed'), start, notes), /Configuration/);
});

test('manual includes all application references and excludes marketing artwork', () => {
  const manual = buildManual(readme, start, notes);
  for (const title of ['Getting started', 'Installation', 'Usage', 'Tasks and subtasks', 'Keybindings', 'Mouse interaction', 'Command-line interface', 'Natural-language add', 'Phone capture', 'Configuration', 'Themes', 'Fork additions', 'Troubleshooting', 'Support and credits']) {
    assert.ok(manual.includes(`## ${title}\n`), title);
  }
  assert.ok(manual.includes('cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked'));
  assert.ok(manual.includes(readmeSection(readme, 'Command-line interface')));
  assert.ok(manual.includes(readmeSection(readme, 'Keybindings')));
  assert.ok(!manual.includes('TuxedoMask.jpg'));
  assert.ok(!manual.includes('docs/screenshots/theme-'));
});

test('installation contains installation commands, not version or help checks', () => {
  const installation = readmeSection(buildManual(readme, start, notes), 'Installation');
  const firstBlock = /```sh\n([\s\S]*?)```/.exec(installation)?.[1].trim();
  assert.equal(firstBlock, 'brew install caracolaracol/tap/tuxedotask');
  assert.doesNotMatch(installation, /tuxedo(?:task)?\s+(?:--help|--version|update\b)/);
  assert.ok(installation.includes('cargo install --git'));
});

test('first task clearly explains automatic files, folders and first-run choices', () => {
  const gettingStarted = readmeSection(buildManual(readme, start, notes), 'Getting started');
  assert.ok(gettingStarted.includes('You do not need to create `todo.txt` yourself'));
  assert.ok(gettingStarted.includes('any missing parent folders'));
  assert.ok(gettingStarted.includes('without clearing its'));
  assert.ok(gettingStarted.includes('no need to run `touch`'));
  assert.ok(gettingStarted.includes('Press `c` to create'));
  assert.ok(gettingStarted.includes('current terminal folder'));
  assert.ok(gettingStarted.includes('Press `s` to try a sample'));
  assert.ok(gettingStarted.includes('tuxedo "$HOME/Documents/tasks/todo.txt"'));
});

test('installation comes first and optional CLI stays after the interactive guide', () => {
  const manual = buildManual(readme, start, notes);
  const titles = [...manual.matchAll(/^## (.+)$/gm)].map(match => match[1]);
  assert.equal(titles[0], 'Installation');
  assert.ok(manual.indexOf('## Installation') < manual.indexOf('### Your first task'));
  assert.ok(titles.indexOf('Command-line interface') > titles.indexOf('Troubleshooting'));
  assert.equal(titles.at(-1), 'Support and credits');
  const usage = readmeSection(manual, 'Usage');
  assert.doesNotMatch(usage, /tuxedo (?:--version|--help|<command>)/);
  assert.doesNotMatch(readmeSection(manual, 'Phone capture'), /echo "/);
  const cli = readmeSection(manual, 'Command-line interface');
  assert.ok(cli.includes('tuxedo --version'));
  assert.ok(cli.includes('tuxedo --help'));
  assert.ok(cli.includes('### Capture from scripts'));
});

test('generated links are suitable for the documentation route', () => {
  const manual = buildManual(readme, start, notes);
  assert.ok(manual.includes('](#installation)'));
  assert.ok(!manual.includes('](#install)'));
  assert.ok(!manual.includes('](docs/themes)'));
  assert.ok(manual.includes('](https://github.com/Caracolaracol/tuxedo-task/tree/main/docs/themes)'));
  assert.equal(manual, buildManual(readme, start, notes));
});
