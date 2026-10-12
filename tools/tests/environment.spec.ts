import { test, expect } from '@playwright/test';

test('cloud Chromium renders an original sprite and supports browser runtime APIs', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await expect(page.locator('body')).toHaveAttribute('data-ready', 'true');
  const health = JSON.parse(await page.locator('#health').innerText());
  expect(health.renderer.toLowerCase()).toContain('webgl');
  expect(health.originalSprite.width).toBeGreaterThan(0);
  expect(health.originalSprite.height).toBeGreaterThan(0);
  expect(health.webAssembly).toBe(true);
  expect(health.webAudio).toBe(true);
  expect(health.indexedDB).toBe(true);
  expect(errors).toEqual([]);
  await page.screenshot({ path: 'test-results/environment.png' });
});
