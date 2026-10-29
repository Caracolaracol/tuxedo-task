import { test, expect } from '@playwright/test';

const command = 'cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked';

test('loads at the Pages subpath with working assets, metadata and attribution', async ({ page, request }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => {
    if (response.url().startsWith('http://127.0.0.1:4321/') && response.status() >= 400) errors.push(response.url());
  });
  await page.goto('./');
  await expect(page).toHaveTitle('Tuxedo Task — Less noise. More done.');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Less noise.More done.✳');
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

test('preview tabs work with pointer and keyboard', async ({ page }) => {
  await page.goto('./');
  const subtasks = page.getByRole('tab', { name: '01 / Subtasks' });
  const projects = page.getByRole('tab', { name: '02 / Projects' });
  const text = page.getByRole('tab', { name: '03 / Plain text' });
  await projects.click();
  await expect(projects).toHaveAttribute('aria-selected', 'true');
  await expect(page.locator('#panel-projects')).toBeVisible();
  await expect(page.locator('#panel-subtasks')).toBeHidden();
  await projects.press('ArrowRight');
  await expect(text).toBeFocused();
  await expect(page.locator('#panel-text')).toBeVisible();
  await text.press('Home');
  await expect(subtasks).toBeFocused();
  await expect(page.locator('#panel-subtasks')).toBeVisible();
  await subtasks.press('End');
  await expect(text).toBeFocused();
  await text.press('ArrowRight');
  await expect(subtasks).toBeFocused();
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
    await expect(page.getByRole('tablist')).toBeHidden();
    await page.locator('summary').filter({ hasText: 'Can I keep using my todo.txt files?' }).click();
    await expect(page.locator('details[open]')).toContainText('fork-specific extension');
  } finally {
    await context.close();
  }
});
