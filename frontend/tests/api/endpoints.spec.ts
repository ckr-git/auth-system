import { test, expect } from '@playwright/test';

const BASE = process.env.API_URL || 'http://localhost:3000';
const timestamp = Date.now();

test.describe('API Integration Tests @smoke', () => {
  let token: string;
  const username = `e2e_api_${timestamp}`;
  const password = 'TestPass123';

  test.describe.configure({ mode: 'serial' });

  test('GET /api/health returns ok', async ({ request }) => {
    const res = await request.get(`${BASE}/api/health`);
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.status).toBe('ok');
  });

  test('POST /api/subjects/register creates user', async ({ request }) => {
    const res = await request.post(`${BASE}/api/subjects/register`, {
      data: { username, display_name: 'API Test', subject_type: 'member', password },
    });
    expect(res.status()).toBe(201);
    const body = await res.json();
    expect(body.success).toBe(true);
    expect(body.data.username).toBe(username);
  });

  test('POST /api/subjects/register duplicate returns 409', async ({ request }) => {
    const res = await request.post(`${BASE}/api/subjects/register`, {
      data: { username, display_name: 'Dup', subject_type: 'member', password },
    });
    expect(res.status()).toBe(409);
  });

  test('POST /api/auth/member/login returns token', async ({ request }) => {
    const res = await request.post(`${BASE}/api/auth/member/login`, {
      data: { username, password },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.success).toBe(true);
    expect(body.data.token).toBeTruthy();
    token = body.data.token;
  });

  test('POST /api/auth/member/login wrong password returns 401', async ({ request }) => {
    const res = await request.post(`${BASE}/api/auth/member/login`, {
      data: { username, password: 'wrong' },
    });
    expect(res.status()).toBe(401);
  });

  test('POST /api/auth/invalid/login returns 400', async ({ request }) => {
    const res = await request.post(`${BASE}/api/auth/invalid/login`, {
      data: { username: 'x', password: 'x' },
    });
    expect(res.status()).toBe(400);
  });

  test('GET /api/subjects/me returns profile', async ({ request }) => {
    const res = await request.get(`${BASE}/api/subjects/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.data.username).toBe(username);
  });

  test('GET /api/subjects/me without token returns 401', async ({ request }) => {
    const res = await request.get(`${BASE}/api/subjects/me`);
    expect(res.status()).toBe(401);
  });

  test('GET /api/credentials/status returns credential info', async ({ request }) => {
    const res = await request.get(`${BASE}/api/credentials/status`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.data.has_password).toBe(true);
    expect(body.data.has_totp).toBe(false);
  });

  test('GET /api/sessions returns session list', async ({ request }) => {
    const res = await request.get(`${BASE}/api/sessions`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.data.length).toBeGreaterThan(0);
  });

  test('POST /api/auth/logout invalidates session', async ({ request }) => {
    const res = await request.post(`${BASE}/api/auth/logout`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(res.status()).toBe(200);

    const meRes = await request.get(`${BASE}/api/subjects/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(meRes.status()).toBe(401);
  });
});
