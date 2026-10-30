import { test, expect } from '@playwright/test';

const docSections = ['Installation', 'Getting started', 'Usage', 'Tasks and subtasks', 'Keybindings', 'Mouse interaction', 'Natural-language add', 'Phone capture', 'Configuration', 'Themes', 'Fork additions', 'Troubleshooting', 'Command-line interface', 'Support and credits'];

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
  const installationCommands = await page.locator('#installation').evaluate(heading => {
    const blocks: string[] = [];
    let sibling = heading.nextElementSibling;
    while (sibling && sibling.tagName !== 'H2') {
      if (sibling.tagName === 'PRE') blocks.push(sibling.textContent?.trim() ?? '');
      sibling = sibling.nextElementSibling;
    }
    return blocks;
  });
  expect(installationCommands[0]).toBe('brew install caracolaracol/tap/tuxedotask');
  expect(installationCommands.join('\n')).not.toMatch(/tuxedo(?:task)? (?:--help|--version|update\b)/);
  await expect(page.locator('#your-first-task + blockquote')).toContainText('You do not need to create todo.txt yourself.');
  await expect(page.locator('#your-first-task + blockquote')).toContainText('any missing parent folders');
  await expect(page.locator('.docs-shortcuts a').nth(1)).toHaveAttribute('href', '#your-first-task');
  await expect(page.locator('.docs-prose')).toContainText('brew install caracolaracol/tap/tuxedotask');
  await expect(page.locator('.docs-prose')).toContainText('cargo install --git https://github.com/Caracolaracol/tuxedo-task --locked');
  await expect(page.locator('.docs-prose')).toContainText('Both command names are available');
  await expect(page.locator('#windows-installer-recommended')).toBeVisible();
  await expect(page.locator('.docs-prose a[href="https://github.com/Caracolaracol/tuxedo-task/releases/latest/download/tuxedo-task-windows-x86_64-setup.exe"]')).toBeVisible();
  await expect(page.locator('.docs-prose')).toContainText('without an administrator prompt');
  await expect(page.locator('.docs-prose')).toContainText('APPDATA');
  await expect(page.locator('.docs-prose')).toContainText('fork-specific extension');
  await expect(page.locator('.docs-prose')).toContainText('trusted networks only');
  await expect(page.getByRole('link', { name: /^Source reference on GitHub/ })).toHaveAttribute('href', 'https://github.com/Caracolaracol/tuxedo-task/blob/main/README.md');
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
