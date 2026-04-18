# API Reference

接口列表以 `backend/src/presentation/router/mod.rs` 为准，仅记录当前代码中真实存在的路由。

## Authentication

除特别说明外，标记为 `Bearer` 的接口需要在 `Authorization: Bearer <token>` 中携带登录令牌。

## Routes

<!-- AUTO-GENERATED:BEGIN backend/src/presentation/router/mod.rs -->
### Health

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/api/health` | No | 健康检查，返回服务状态。 |

### Subjects

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/subjects/register` | No | 注册主体账号。 |
| `GET` | `/api/subjects/me` | Bearer | 获取当前登录用户信息。 |

### Credentials / TOTP

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/credentials/totp/setup` | Bearer | 生成 TOTP 绑定信息。 |
| `POST` | `/api/credentials/totp/confirm` | Bearer | 提交验证码并启用 TOTP。 |
| `POST` | `/api/credentials/totp/verify` | Bearer | 校验 TOTP 验证码。 |
| `GET` | `/api/credentials/status` | Bearer | 查询当前凭证状态。 |

### Credentials / Passkey

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/credentials/passkey/register-begin` | Bearer | 发起 Passkey 注册。 |
| `POST` | `/api/credentials/passkey/register-complete` | Bearer | 完成 Passkey 注册。 |
| `POST` | `/api/credentials/passkey/authenticate-begin` | Bearer | 发起已登录状态的 Passkey 验证。 |
| `POST` | `/api/credentials/passkey/authenticate-complete` | Bearer | 完成已登录状态的 Passkey 验证。 |

### Auth

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/auth/{subject_type}/login` | No | 按主体类型登录；`{subject_type}` 为 `member`、`staff` 或 `admin`。 |
| `POST` | `/api/auth/mfa/verify` | No | 使用 `mfa_token` 和验证码完成 MFA 登录。 |
| `POST` | `/api/auth/logout` | Bearer | 登出当前会话。 |
| `POST` | `/api/auth/passkey/begin` | No | 发起 Passkey 登录。 |
| `POST` | `/api/auth/passkey/complete` | No | 完成 Passkey 登录。 |

### Sessions

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/api/sessions` | Bearer | 列出当前用户活跃会话。 |
| `DELETE` | `/api/sessions/{session_id}` | Bearer | 撤销指定会话；`{session_id}` 为会话 ID。 |
<!-- AUTO-GENERATED:END backend/src/presentation/router/mod.rs -->
