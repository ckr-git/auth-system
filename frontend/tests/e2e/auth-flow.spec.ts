import { test, expect } from '@playwright/test';

const timestamp = Date.now();

test.describe('Auth Flow @smoke', () => {
  const memberUser = `e2e_auth_member_${timestamp}`;
  const staffUser = `e2e_auth_staff_${timestamp}`;
  const adminUser = `e2e_auth_admin_${timestamp}`;
  const password = 'TestPass123';

  test.describe.configure({ mode: 'serial' });

  test('Member: register → dashboard → logout', async ({ page }) => {
    await page.goto('/member/login');
    await page.getByRole('tab', { name: 'Register' }).click();
    await page.getByTestId('register-input-username').fill(memberUser);
    await page.getByTestId('register-input-displayname').fill('E2E Member');
    await page.getByTestId('register-input-password').fill(password);
    await page.getByTestId('register-btn-submit').click();

    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByText(memberUser)).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
    await expect(page.getByText('Password:')).toBeVisible();

    await page.getByTestId('layout-btn-logout').click();
    await expect(page).toHaveURL('/member/login');
  });

  test('Staff: register → dashboard → logout', async ({ page }) => {
    await page.goto('/staff/login');
    await expect(page.getByRole('heading', { name: 'Community Staff Portal' })).toBeVisible();
    await page.getByRole('tab', { name: 'Register' }).click();
    await page.getByTestId('register-input-username').fill(staffUser);
    await page.getByTestId('register-input-displayname').fill('E2E Staff');
    await page.getByTestId('register-input-password').fill(password);
    await page.getByTestId('register-btn-submit').click();

    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByRole('banner').getByText('community_staff')).toBeVisible();

    await page.getByTestId('layout-btn-logout').click();
    await expect(page).toHaveURL('/member/login');
  });

  test('Admin: register → dashboard → logout', async ({ page }) => {
    await page.goto('/admin/login');
    await expect(page.getByRole('heading', { name: 'Platform Staff Portal' })).toBeVisible();
    await page.getByRole('tab', { name: 'Register' }).click();
    await page.getByTestId('register-input-username').fill(adminUser);
    await page.getByTestId('register-input-displayname').fill('E2E Admin');
    await page.getByTestId('register-input-password').fill(password);
    await page.getByTestId('register-btn-submit').click();

    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByRole('banner').getByText('platform_staff')).toBeVisible();

    await page.getByTestId('layout-btn-logout').click();
    await expect(page).toHaveURL('/member/login');
  });

  test('Login with existing member account', async ({ page }) => {
    await page.goto('/member/login');
    await page.getByTestId('login-input-username').fill(memberUser);
    await page.getByTestId('login-input-password').fill(password);
    await page.getByTestId('login-btn-submit').click();

    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByText(memberUser)).toBeVisible();
  });

  test('Login with wrong credentials shows error', async ({ page }) => {
    await page.goto('/member/login');
    await page.getByTestId('login-input-username').fill('nonexistent');
    await page.getByTestId('login-input-password').fill('wrongpass');
    await page.getByTestId('login-btn-submit').click();

    await expect(page.getByText('Invalid credentials')).toBeVisible();
  });

  test('Empty form shows validation errors', async ({ page }) => {
    await page.goto('/member/login');
    await page.getByTestId('login-btn-submit').click();

    await expect(page.getByText('Username is required')).toBeVisible();
    await expect(page.getByText('Password is required')).toBeVisible();
  });
});
