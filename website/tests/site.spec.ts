import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

const command = 'brew install caracolaracol/tap/tuxedotask';

test('loads at the Pages subpath with working assets, metadata and attribution', async ({ page, request }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => {
    if (response.url().startsWith('http://127.0.0.1:4321/') && response.status() >= 400) errors.push(response.url());
  });
  await page.goto('./');
  await expect(page).toHaveTitle('Tuxedo Task — for justice. with your tasks done');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('for justice. with your tasks done');
  await expect(page.locator('#closing-title')).toHaveText('sharp enough to prune crime');
  await expect(page.getByRole('navigation').getByRole('link', { name: 'Features', exact: true })).toHaveAttribute('href', '#features');
  await expect(page.locator('#features .eyebrow')).toHaveText('01 / Features');
  await expect(page.locator('#features-title')).toHaveText('Features');
  await expect(page.locator('#features h3')).toHaveText(['Subtasks', 'Project grouping', 'Family editing', 'Mouse support', 'Persistent filters', 'Layout customization']);
  await expect(page.locator('.hero-description')).toHaveText('A rose, a mask, and a mission: bring order to your todo.txt. Break down your tasks, finish the job, and make a dramatic exit.');
  await expect(page.getByRole('link', { name: /View full size/i })).toHaveCount(0);
  await expect(page.getByRole('link', { name: 'Full installation guide' })).toHaveAttribute('href', '/tuxedo-task/docs/#installation');
  await expect(page.getByRole('link', { name: 'All the shortcuts' })).toHaveAttribute('href', '/tuxedo-task/docs/#keybindings');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://caracolaracol.github.io/tuxedo-task/');
  await expect(page.locator('#install-command')).toHaveText(command);
  await expect(page.locator('.install-top')).toContainText('Install with Homebrew');
  await expect(page.locator('.installation')).toContainText('No Rust or Cargo required.');
  await expect(page.locator('.install-notes')).toContainText('tuxedotask');
  await expect(page.locator('footer')).toContainText('independent community fork');
  await expect(page.locator('footer a[href="https://github.com/webstonehq/tuxedo"]')).toBeVisible();
  for (const image of await page.locator('img').all()) {
    await image.scrollIntoViewIfNeeded();
    await expect.poll(() => image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0)).toBeTruthy();
    expect(await image.getAttribute('src')).toMatch(/^\/tuxedo-task\//);
  }
  const socialImage = await request.get('social-card.png');
  expect(socialImage.ok()).toBeTruthy();
  expect(socialImage.headers()['content-type']).toContain('image/png');
  expect(errors).toEqual([]);
});

test('four project principles are centered without a slogan or separator lines', async ({ page }) => {
  await page.goto('./');
  const principles = page.locator('.principles');
  await expect(principles.locator('li')).toHaveText(['Plain-text forever', 'Local-first', 'Room to grow', 'MIT licensed']);
  await expect(page.getByText('Small tool. Clear intentions.', { exact: true })).toHaveCount(0);
  for (const width of [320, 390, 768, 1440, 2560]) {
    await page.setViewportSize({ width, height: 1000 });
    const metrics = await principles.evaluate(element => {
      const style = getComputedStyle(element);
      const items = [...element.querySelectorAll('li')].map(item => item.getBoundingClientRect());
      return { borderTop: style.borderTopWidth, borderBottom: style.borderBottomWidth,
        center: (Math.min(...items.map(item => item.left)) + Math.max(...items.map(item => item.right))) / 2,
        separators: [...element.querySelectorAll('li')].some(item => {
          const css = getComputedStyle(item);
          return parseFloat(css.borderLeftWidth) > 0 || parseFloat(css.borderRightWidth) > 0;
        }) };
    });
    expect(metrics.borderTop).toBe('0px');
    expect(metrics.borderBottom).toBe('0px');
    expect(metrics.separators).toBe(false);
    expect(Math.abs(metrics.center - width / 2), `principles centered at ${width}px`).toBeLessThan(1);
  }
});

test('hero uses the real renderer output and opens it with the keyboard', async ({ page, request }) => {
  await page.goto('./');
  const hero = page.locator('.hero-preview');
  await expect(hero.locator('img')).toHaveAttribute('src', '/tuxedo-task/screenshots/subtasks.svg');
  await expect(hero).toContainText('Rendered by the application itself');
  await expect(page.locator('[data-terminal-demo]')).toHaveCount(0);
  await expect(page.getByRole('tablist')).toHaveCount(0);
  const image = await request.get('screenshots/subtasks.svg');
  expect(await image.body()).toEqual(await readFile(new URL('../../docs/screenshots/subtasks.svg', import.meta.url)));
  const fullSize = page.getByRole('link', { name: 'Open full-size application screenshot', exact: true });
  await fullSize.focus();
  await fullSize.press('Enter');
  await expect(page).toHaveURL('http://127.0.0.1:4321/tuxedo-task/screenshots/subtasks.svg');
});

test('rose dividers are local, decorative, and credited under their own license', async ({ page, request }) => {
  await page.goto('./');
  const dividers = page.locator('.rose-divider');
  await expect(dividers).toHaveCount(4);
  const border = page.locator('[data-rose-border]');
  await expect(border).toHaveCount(1);
  await expect(page.locator('.rose-divider, [data-rose-border]').first()).toHaveAttribute('data-rose-border', '');
  await expect(border).toHaveAttribute('aria-hidden', 'true');
  await expect(border.locator('svg')).toHaveAttribute('viewBox', '0 0 618.603 186');
  expect(await border.locator('path').count()).toBeGreaterThan(10);
  await expect(border.locator('img[src$="rose.svg"]')).toHaveCount(0);
  for (const divider of await dividers.all()) {
    await expect(divider).toHaveAttribute('aria-hidden', 'true');
    await expect(divider.locator('img')).toHaveAttribute('src', '/tuxedo-task/third-party/rose.svg');
    await expect(divider.locator('img')).toHaveAttribute('alt', '');
  }
  await expect(page.locator('footer .art-credit')).toContainText('Twemoji contributors');
  await expect(page.locator('footer .art-credit')).toContainText('johnny_automatic / Openclipart');
  await expect(page.locator('footer a[href="https://creativecommons.org/publicdomain/zero/1.0/"]')).toBeVisible();
  await expect(page.locator('footer .art-credit a').last()).toHaveAttribute('href', 'https://creativecommons.org/licenses/by/4.0/');
  const rose = await request.get('third-party/rose.svg');
  expect(rose.ok()).toBeTruthy();
  expect(await rose.body()).toEqual(await readFile(new URL('../public/third-party/rose.svg', import.meta.url)));
  const license = await request.get('third-party/LICENSE-Twemoji-graphics.txt');
  expect(license.ok()).toBeTruthy();
  expect(await license.text()).toContain('Attribution 4.0 International');
});

test('copies the exact fork installation command', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.goto('./#install');
  await page.getByRole('button', { name: 'Copy installation command', exact: true }).click();
  await expect(page.getByRole('status')).toHaveText('Copied. Your terminal is next.');
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(command);
});

test('clipboard failure selects the command and explains manual copying', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: async () => { throw new Error('Permission denied'); } } });
  });
  await page.goto('./#install');
  await page.getByRole('button', { name: 'Copy installation command', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Command selected.');
  expect(await page.evaluate(() => window.getSelection()?.toString())).toBe(command);
});

test('keyboard walkthrough includes lowercase n and supports manual keyboard selection', async ({ page }) => {
  await page.goto('./');
  const guide = page.locator('[data-shortcut-guide]');
  const addTask = guide.getByRole('button', { name: 'n: Add a new task', exact: true });
  await expect(addTask.locator('kbd')).toHaveText('N');
  await addTask.focus();
  await expect(guide).toHaveAttribute('data-playing', 'false');
  await addTask.press('Enter');
  await expect(addTask).toHaveAttribute('aria-pressed', 'true');
  await expect(guide.locator('[data-shortcut-note]')).toContainText('press n to open the add-task dialog');
  await expect(guide.getByRole('button', { name: 'Play animation', exact: true })).toBeVisible();
  const undo = guide.getByRole('button', { name: 'u: Undo the last change', exact: true });
  await undo.focus();
  await undo.press('Space');
  await expect(undo).toHaveAttribute('aria-pressed', 'true');
  await expect(guide.locator('[data-shortcut-counter]')).toHaveText('05 / 05');
  await expect(guide.locator('[aria-pressed="true"]')).toHaveCount(1);
});

test('keyboard animation advances, pauses completely, and resumes', async ({ page }) => {
  await page.goto('./');
  const guide = page.locator('[data-shortcut-guide]');
  await guide.scrollIntoViewIfNeeded();
  await expect(guide).toHaveAttribute('data-playing', 'true');
  const active = () => guide.locator('[aria-pressed="true"]').getAttribute('data-shortcut');
  const initial = await active();
  await expect.poll(active, { timeout: 5000 }).not.toBe(initial);
  await guide.getByRole('button', { name: 'Pause animation', exact: true }).click();
  await expect(guide).toHaveAttribute('data-playing', 'false');
  const paused = await active();
  await page.waitForTimeout(2800);
  expect(await active()).toBe(paused);
  await guide.getByRole('button', { name: 'Play animation', exact: true }).click();
  await expect(guide).toHaveAttribute('data-playing', 'true');
  await expect.poll(active, { timeout: 5000 }).not.toBe(paused);
  await page.getByRole('heading', { level: 1 }).scrollIntoViewIfNeeded();
  await expect(guide).toHaveAttribute('data-playing', 'false');
  const offscreen = await active();
  await page.waitForTimeout(2800);
  expect(await active()).toBe(offscreen);
  await guide.scrollIntoViewIfNeeded();
  await expect(guide).toHaveAttribute('data-playing', 'true');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect(guide).toHaveAttribute('data-playing', 'false');
  await expect(guide.getByRole('button', { name: 'Play animation', exact: true })).toBeVisible();
});

test('reduced motion disables autoplay while keeping every shortcut explorable', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('./');
  const guide = page.locator('[data-shortcut-guide]');
  await guide.scrollIntoViewIfNeeded();
  await expect(guide).toHaveAttribute('data-playing', 'false');
  await expect(guide.getByRole('button', { name: 'Play animation', exact: true })).toBeVisible();
  await page.waitForTimeout(2800);
  await expect(guide.locator('[data-shortcut="n"]')).toHaveAttribute('aria-pressed', 'true');
  expect(await guide.locator('kbd').first().evaluate(element => getComputedStyle(element).animationName)).toBe('none');
  await guide.getByRole('button', { name: 'x: Complete a task', exact: true }).click();
  await expect(guide.locator('[data-shortcut-counter]')).toHaveText('04 / 05');
  await expect(guide).toHaveAttribute('data-playing', 'false');
});

test('FAQ uses native keyboard-accessible disclosure', async ({ page }) => {
  await page.goto('./');
  const question = page.locator('summary').filter({ hasText: 'Will Homebrew install this fork?' });
  await question.focus();
  await question.press('Enter');
  await expect(page.locator('details[open]')).toContainText('brew install tuxedo installs the upstream');
});

test('does not overflow at mobile, tablet or desktop sizes', async ({ page }) => {
  await page.goto('./');
  for (const width of [320, 375, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), `layout at ${width}px`).toBeTruthy();
  }
});

test('content and navigation still work without JavaScript', async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  try {
    await page.goto('http://127.0.0.1:4321/tuxedo-task/');
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
    await expect(page.locator('#install-command')).toHaveText(command);
    await expect(page.getByRole('button', { name: 'Copy installation command', exact: true })).toBeHidden();
    await expect(page.locator('.hero-preview img')).toBeVisible();
    await expect(page.locator('[data-terminal-demo]')).toHaveCount(0);
    await expect(page.locator('[data-shortcut="n"]')).toBeDisabled();
    await expect(page.locator('[data-shortcut-controls]')).toBeHidden();
    await page.locator('summary').filter({ hasText: 'Can I keep using my todo.txt files?' }).click();
    await expect(page.locator('details[open]')).toContainText('fork-specific extension');
  } finally {
    await context.close();
  }
});
