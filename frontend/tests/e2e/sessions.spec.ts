import { test, expect } from '@playwright/test';

const timestamp = Date.now();
const username = `e2e_sessions_${timestamp}`;
const password = 'TestPass123';

function buildUsername(suffix: string) {
  return `${username}_${suffix}`.slice(0, 32);
}

test.describe('Session Management @smoke', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/member/login');
    await page.getByRole('tab', { name: 'Register' }).click();
    await page.getByTestId('register-input-username').fill(buildUsername(test.info().title.slice(0, 8)));
    await page.getByTestId('register-input-displayname').fill('Session Test');
    await page.getByTestId('register-input-password').fill(password);
    await page.getByTestId('register-btn-submit').click();
    await expect(page).toHaveURL('/dashboard');
  });

  test('Sessions page shows current session', async ({ page }) => {
    await page.getByRole('menuitem', { name: /Sessions/ }).click();
    await expect(page).toHaveURL('/sessions');
    await expect(page.getByRole('heading', { name: 'Active Sessions' })).toBeVisible();
    await expect(page.getByText('Current')).toBeVisible();
  });

  test('Current session revoke button is disabled', async ({ page }) => {
    await page.getByRole('menuitem', { name: /Sessions/ }).click();
    await expect(page).toHaveURL('/sessions');
    const revokeBtn = page.getByRole('button', { name: /Revoke/ });
    await expect(revokeBtn).toBeDisabled();
  });
});
