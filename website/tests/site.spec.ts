import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

const command = 'cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked';

test('loads at the Pages subpath with working assets, metadata and attribution', async ({ page, request }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => {
    if (response.url().startsWith('http://127.0.0.1:4321/') && response.status() >= 400) errors.push(response.url());
  });
  await page.goto('./');
  await expect(page).toHaveTitle('Tuxedo Task — for justice. with your tasks done');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('for justice. with your tasks done');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://caracolaracol.github.io/tuxedo-task/');
  await expect(page.locator('#install-command')).toHaveText(command);
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
  await expect(dividers).toHaveCount(5);
  for (const divider of await dividers.all()) {
    await expect(divider).toHaveAttribute('aria-hidden', 'true');
    await expect(divider.locator('img')).toHaveAttribute('src', '/tuxedo-task/third-party/rose.svg');
    await expect(divider.locator('img')).toHaveAttribute('alt', '');
  }
  await expect(page.locator('footer .art-credit')).toContainText('Twemoji contributors');
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
    await page.locator('summary').filter({ hasText: 'Can I keep using my todo.txt files?' }).click();
    await expect(page.locator('details[open]')).toContainText('fork-specific extension');
  } finally {
    await context.close();
  }
});
