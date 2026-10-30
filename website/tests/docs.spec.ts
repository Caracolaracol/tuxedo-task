import { test, expect } from '@playwright/test';

const docSections = ['Getting started', 'Installation', 'Usage', 'Tasks and subtasks', 'Keybindings', 'Mouse interaction', 'Command-line interface', 'Natural-language add', 'Phone capture', 'Configuration', 'Themes', 'Fork additions', 'Troubleshooting', 'Support and credits'];

test('documentation is linked from the landing and contains the full application guide', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => {
    if (response.url().startsWith('http://127.0.0.1:4321/') && response.status() >= 400) errors.push(response.url());
  });
  await page.goto('./');
  await page.getByRole('navigation', { name: 'Main navigation' }).getByRole('link', { name: 'Documentation', exact: true }).click();
  await expect(page).toHaveURL('http://127.0.0.1:4321/tuxedo-task/docs/');
  await expect(page).toHaveTitle('Documentation — Tuxedo Task');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Documentation');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://caracolaracol.github.io/tuxedo-task/docs/');
  await expect(page.getByRole('navigation', { name: 'Main navigation' }).getByRole('link', { name: 'Documentation', exact: true })).toHaveAttribute('aria-current', 'page');
  await expect(page.locator('.docs-prose h2')).toHaveText(docSections);
  await expect(page.locator('.docs-prose')).toContainText('cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked');
  await expect(page.locator('.docs-prose')).toContainText('fork-specific extension');
  await expect(page.locator('.docs-prose')).toContainText('trusted networks only');
  expect(errors).toEqual([]);
});

test('table of contents, quick links and internal guide references resolve to unique targets', async ({ page }) => {
  await page.goto('docs/');
  const invalid = await page.locator('.docs-page a[href^="#"]').evaluateAll(links => links.flatMap(link => {
    const hash = link.getAttribute('href') ?? '';
    const id = decodeURIComponent(hash.slice(1));
    return document.querySelectorAll(`[id="${CSS.escape(id)}"]`).length === 1 ? [] : [hash];
  }));
  expect(invalid).toEqual([]);
  const duplicateIds = await page.locator('[id]').evaluateAll(elements => {
    const ids = elements.map(element => element.id);
    return ids.filter((id, index) => ids.indexOf(id) !== index);
  });
  expect(duplicateIds).toEqual([]);
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.getByRole('link', { name: 'Work with subtasks' }).click();
  await expect(page).toHaveURL(/\/docs\/#subtasks-nested-tasks$/);
  await expect(page.locator('#subtasks-nested-tasks')).toBeInViewport();
});

test('CLI tables, config examples and custom-theme reference are readable', async ({ page }) => {
  await page.goto('docs/');
  expect(await page.locator('.docs-prose table').count()).toBeGreaterThan(5);
  expect(await page.locator('.docs-prose pre').count()).toBeGreaterThan(8);
  await expect(page.locator('.docs-prose')).toContainText('mouse = true');
  await expect(page.locator('.docs-prose')).toContainText('TODO_FILE');
  await expect(page.locator('.docs-prose')).toContainText('Family editing');
  const themeReference = page.locator('.docs-prose summary').filter({ hasText: 'Theme file format and field reference' });
  await themeReference.click();
  await expect(page.locator('.docs-prose details[open]')).toContainText('status_fg');
  await expect(page.locator('.docs-prose a[href="https://github.com/Caracolaracol/tuxedo-task/tree/main/docs/themes"]')).toBeVisible();
});

test('documentation remains within the viewport at all supported widths', async ({ page }) => {
  await page.goto('docs/');
  for (const width of [320, 375, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), `docs layout at ${width}px`).toBeTruthy();
  }
});

test.describe('without JavaScript', () => {
  test.use({ javaScriptEnabled: false });
  test('guide navigation and native reference disclosures still work', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('docs/');
    await expect(page.getByRole('heading', { name: 'Documentation', exact: true })).toBeVisible();
    const mobileMenu = page.locator('.docs-mobile-nav');
    if (await mobileMenu.isVisible()) await mobileMenu.locator('summary').click();
    const contents = page.getByRole('navigation', { name: 'Documentation sections' });
    await expect(contents).toBeVisible();
    await contents.getByRole('link', { name: 'Installation', exact: true }).click();
    await expect(page).toHaveURL(/\/docs\/#installation$/);
    await expect(page.locator('#installation')).toBeInViewport();
    await page.getByRole('link', { name: 'Tuxedo Task home', exact: true }).click();
    await expect(page).toHaveURL('http://127.0.0.1:4321/tuxedo-task/');
  });
});
