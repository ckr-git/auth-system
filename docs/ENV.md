# Environment Variables

后端运行依赖以下环境变量，来源是 `backend/.env.example`。前端当前没有独立的 `.env.example`。

<!-- AUTO-GENERATED:BEGIN backend/.env.example -->
| Variable | Required | Description | Example |
|----------|----------|-------------|---------|
| `DATABASE_URL` | Yes | PostgreSQL 连接字符串，后端启动时用于建立数据库连接并执行迁移。 | `postgres://postgres:postgres@localhost:5432/auth_system` |
| `REDIS_URL` | Yes | Redis 连接字符串，用于会话和 MFA token 存储。 | `redis://127.0.0.1:6379` |
| `JWT_SECRET` | Yes | JWT 签名密钥。 | `change-me-to-a-random-secret` |
| `RUST_LOG` | No | Rust 日志过滤配置；未设置时后端会回退到默认调试过滤。 | `auth_system=debug,tower_http=debug` |
| `WEBAUTHN_RP_ID` | Yes | WebAuthn relying party ID。 | `localhost` |
| `WEBAUTHN_RP_ORIGIN` | Yes | WebAuthn relying party origin，需与前端访问地址一致。 | `http://localhost:5173` |
<!-- AUTO-GENERATED:END backend/.env.example -->

## Usage

1. 复制示例文件：`cp backend/.env.example backend/.env`
2. 至少修改 `JWT_SECRET` 为随机强密钥。
3. 确保 `WEBAUTHN_RP_ORIGIN` 与你实际访问前端的地址一致，否则 Passkey 相关流程会失败。
