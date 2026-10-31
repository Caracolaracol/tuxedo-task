import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { dirname, resolve, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildManual, loadGuides, markdownSection } from './manual.mjs';

const guides = await loadGuides();
const manual = buildManual(guides);
const repo = fileURLToPath(new URL('../../', import.meta.url));
const readme = await readFile(resolve(repo, 'README.md'), 'utf8');

// Match GitHub's simple heading IDs for the English guides, excluding code fences.
function headingIds(source) {
  let inFence = false;
  return source.split('\n').flatMap(line => {
    if (line.startsWith('```')) inFence = !inFence;
    if (inFence || !/^#{1,6} /.test(line)) return [];
    return [line.replace(/^#+ /, '').toLowerCase().replace(/[^\p{L}\p{N}_\- ]/gu, '').replaceAll(' ', '-')];
  });
}

async function assertExactPath(path) {
  let current = repo;
  for (const part of relative(repo, path).split(sep)) {
    assert.ok((await readdir(current)).includes(part), `Wrong path/case: ${relative(repo, path)}`);
    current = resolve(current, part);
  }
}

test('section extraction keeps subheadings and ignores headings inside code fences', () => {
  const source = '# Title\n\n## Guide\nKeep this.\n### Child\n```text\n## Not a heading\n```\n\n## Next\nSkip this.\n';
  const actual = markdownSection(source, 'Guide');
  assert.ok(actual.includes('### Child'));
  assert.ok(actual.includes('## Not a heading'));
  assert.ok(!actual.includes('Skip this.'));
});

test('missing guide files or required headings fail loudly', () => {
  assert.throws(() => markdownSection(manual, 'Missing section'), /Documentation source missing/);
  assert.throws(() => buildManual({ ...guides, 'configuration.md': undefined }), /configuration.md/);
  assert.throws(() => buildManual({ ...guides, 'configuration.md': guides['configuration.md'].replace('# Configuration', '# Removed') }), /Configuration/);
});

test('README is a concise TUI-first overview with retained identity and licensing', () => {
  assert.ok(readme.split('\n').length < 160, 'Keep detailed reference material in Docs/');
  const title = readme.split('\n')[0];
  assert.match(title, /^# <img src="src\/serve\/assets\/icon\.svg"[^>]+> Tuxedo Task$/);
  assert.ok(readme.indexOf('TuxedoMask.jpg') < readme.indexOf('## Features'));
  for (const feature of ['Persona', 'recurrence', 'subtasks', 'command palette', 'Markdown notes', 'Mouse support', 'Phone capture', 'JSON', 'custom themes', 'saved searches', 'archive', 'undo']) {
    assert.ok(readme.includes(feature), feature);
  }
  assert.ok(readme.includes('## Quick start — TUI'));
  assert.ok(readme.includes('Docs/README.md'));
  assert.ok(readme.includes('independent community fork'));
  assert.ok(readme.includes('not covered by MIT'));
  assert.ok(readme.includes('website/THIRD_PARTY.md'));
  assert.doesNotMatch(readme, /## Configuration|## Command-line interface|## Keybindings/);
});

test('all local README and guide links exist with correct case and anchors', async () => {
  const docsFiles = (await readdir(resolve(repo, 'Docs'))).filter(name => name.endsWith('.md'));
  for (const file of ['README.md', ...docsFiles.map(name => `Docs/${name}`)]) {
    const source = await readFile(resolve(repo, file), 'utf8');
    for (const [, href] of source.matchAll(/\]\(([^)]+)\)/g)) {
      if (/^(?:https?:|mailto:)/.test(href)) continue;
      const [path, fragment] = href.split('#');
      const target = path ? resolve(repo, dirname(file), path) : resolve(repo, file);
      await assertExactPath(target);
      if (fragment && target.endsWith('.md')) {
        const ids = headingIds(await readFile(target, 'utf8'));
        assert.ok(ids.includes(fragment), `${file}: ${href}`);
      }
    }
  }
});

test('manual includes all application references and excludes marketing artwork', () => {
  for (const title of ['Getting started', 'Installation', 'Usage', 'Tasks and subtasks', 'Keybindings', 'Mouse interaction', 'Command-line interface', 'Natural-language add', 'Phone capture', 'Configuration', 'Themes', 'Fork additions', 'Troubleshooting', 'Support and credits']) {
    assert.ok(manual.includes(`## ${title}\n`), title);
  }
  assert.ok(manual.includes('cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked'));
  assert.ok(!manual.includes('TuxedoMask.jpg'));
  assert.ok(!manual.includes('screenshots/theme-'));
  assert.ok(!manual.includes('[Documentation index]'));
  assert.ok(manual.includes('[normal]\n\n# Navigation')); // Do not promote code comments to headings.
});

test('installation contains installation commands, not version or help checks', () => {
  const installation = markdownSection(manual, 'Installation');
  const firstBlock = /```sh\n([\s\S]*?)```/.exec(installation)?.[1].trim();
  assert.equal(firstBlock, 'brew install caracolaracol/tap/tuxedotask');
  assert.doesNotMatch(installation, /tuxedo(?:task)?\s+(?:--help|--version|update\b)/);
  assert.ok(installation.includes('cargo install --git'));
});

test('first task clearly explains automatic files, folders and first-run choices', () => {
  const gettingStarted = markdownSection(manual, 'Getting started');
  for (const text of ['You do not need to create `todo.txt` yourself', 'any missing parent folders', 'without clearing its', 'no need to run `touch`', 'Press `c` to create', 'current terminal folder', 'Press `s` to try a sample', 'tuxedo "$HOME/Documents/tasks/todo.txt"']) {
    assert.ok(gettingStarted.includes(text), text);
  }
});

test('installation comes first and optional CLI stays after the interactive guide', () => {
  const titles = [...manual.matchAll(/^## (.+)$/gm)].map(match => match[1]);
  assert.equal(titles[0], 'Installation');
  assert.ok(manual.indexOf('## Installation') < manual.indexOf('### Your first task'));
  assert.ok(titles.indexOf('Command-line interface') > titles.indexOf('Troubleshooting'));
  assert.equal(titles.at(-1), 'Support and credits');
  assert.doesNotMatch(markdownSection(manual, 'Usage'), /tuxedo (?:--version|--help|<command>)/);
  assert.doesNotMatch(markdownSection(manual, 'Phone capture'), /echo "/);
  const cli = markdownSection(manual, 'Command-line interface');
  assert.ok(cli.includes('tuxedo --version'));
  assert.ok(cli.includes('tuxedo --help'));
  assert.ok(cli.includes('### Capture from scripts'));
});

test('Persona reference documents Katakana, shortcuts, sharing limits and availability', () => {
  const people = markdownSection(manual, 'Persona assignments', 3);
  for (const text of ['ペルソナ', '::alias', 'Press `m`', 'Press `fm`', 'IPv6 stays literal', 'not an account system']) {
    assert.ok(people.includes(text), text);
  }
  assert.ok(manual.includes('`listpersona`'));
  assert.ok(manual.includes('`people` string array'));
});

test('generated links resolve inside the single-page guide or to GitHub assets', () => {
  assert.ok(manual.includes('](#installation)'));
  assert.ok(!manual.includes('](#install)'));
  assert.ok(manual.includes('](https://github.com/Caracolaracol/tuxedo-task/tree/main/Docs/themes)'));
  assert.ok(!/\]\([a-z-]+\.md(?:#|\))/.test(manual), 'No unresolved relative Markdown links');
  const ids = headingIds(manual);
  for (const [, id] of manual.matchAll(/\]\(#([^)]*)\)/g)) {
    assert.ok(ids.includes(id), `Missing website anchor: ${id}`);
  }
  assert.equal(manual, buildManual(guides));
});
