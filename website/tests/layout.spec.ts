import { test, expect, type Page } from '@playwright/test';

async function landingSizes(page: Page) {
  return page.evaluate(() => {
    const element = (selector: string) => document.querySelector(selector)!;
    const width = (selector: string) => element(selector).getBoundingClientRect().width;
    return {
      font: parseFloat(getComputedStyle(element('.hero-description')).fontSize),
      hero: width('.hero-preview img'),
      screenshot: width('.workflow .real-preview img'),
      logo: width('.site-header .brand img'),
      icon: width('.principles .ui-icon'),
      key: width('[data-shortcut="n"] kbd'),
      space: parseFloat(getComputedStyle(element('.feature-card')).paddingTop),
    };
  });
}

test('large screens expand the whole landing without a fixed width or page zoom', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('./');
  const baseline = await landingSizes(page);
  for (const width of [1440, 1920, 2560, 3840]) {
    await page.setViewportSize({ width, height: Math.round(width * 9 / 16) });
    const sizes = await landingSizes(page);
    for (const key of ['font', 'hero', 'screenshot', 'logo', 'icon', 'key', 'space'] as const) {
      expect(sizes[key] / baseline[key], `${key} scales at ${width}px`).toBeCloseTo(width / 1440, 1);
    }
    const layout = await page.locator('.site-shell').evaluate(shell => ({
      width: shell.getBoundingClientRect().width,
      maxWidth: getComputedStyle(shell).maxWidth,
      zoom: getComputedStyle(document.body).zoom,
      overflow: document.documentElement.scrollWidth > innerWidth,
    }));
    expect(layout.width).toBeCloseTo(width, 0);
    expect(layout.maxWidth).toBe('none');
    expect(layout.zoom).toBe('1');
    expect(layout.overflow).toBe(false);
  }
});

test('wide documentation grows with the viewport and keeps its sticky index usable', async ({ page }) => {
  await page.goto('docs/');
  for (const [width, height] of [[1440, 900], [1920, 1080], [2560, 1440], [3840, 2160], [3440, 900]]) {
    await page.setViewportSize({ width, height });
    const layout = await page.evaluate(() => {
      const article = document.querySelector<HTMLElement>('.docs-article')!;
      const sidebar = document.querySelector<HTMLElement>('.docs-sidebar')!;
      const shell = document.querySelector<HTMLElement>('.site-shell')!;
      return {
        font: parseFloat(getComputedStyle(document.querySelector('.docs-prose')!).fontSize),
        maxWidth: getComputedStyle(article).maxWidth,
        right: article.getBoundingClientRect().right,
        contentRight: shell.getBoundingClientRect().right - parseFloat(getComputedStyle(shell).paddingRight),
        sidebarHeight: sidebar.getBoundingClientRect().height,
        sidebarTop: parseFloat(getComputedStyle(sidebar).top),
        overflow: document.documentElement.scrollWidth > innerWidth,
      };
    });
    expect(layout.font).toBeCloseTo(15 * width / 1440, 1);
    expect(layout.maxWidth).toBe('none');
    expect(layout.right).toBeCloseTo(layout.contentRight, 0);
    expect(layout.sidebarHeight).toBeLessThanOrEqual(height - 2 * layout.sidebarTop + 1);
    expect(layout.overflow).toBe(false);
  }
});
