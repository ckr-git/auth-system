# Runbook

## Services and ports

| Service | Default Port | Source |
|---------|--------------|--------|
| Frontend (Vite) | `5173` | `frontend/package.json`, `backend/.env.example` |
| Backend (Axum) | `3000` | `backend/src/main.rs` |
| PostgreSQL | `5432` | `docker-compose.yml` |
| Redis | `6379` | `docker-compose.yml` |

## Startup order

1. 启动基础设施：`docker compose up -d`
2. 配置后端环境变量：`cp backend/.env.example backend/.env`
3. 启动后端：`cd backend && cargo run`
4. 启动前端：`cd frontend && npm install && npm run dev`

## Health check

- 后端健康检查：`GET /api/health`
- 默认地址：`http://localhost:3000/api/health`

## Minimal functional checks

启动完成后，至少检查以下页面与路径：

- `/member/login`
- `/staff/login`
- `/admin/login`
- `/dashboard`
- `/sessions`

根路径 `/` 会自动跳转到 `/member/login`。

## Common issues

### Backend fails at startup

优先检查：
- `backend/.env` 是否存在，且 `DATABASE_URL`、`JWT_SECRET` 等变量已配置。
- PostgreSQL 和 Redis 是否已启动。
- 数据库端口 `5432`、Redis 端口 `6379` 是否被占用。

### WebAuthn / Passkey flow fails

优先检查：
- `WEBAUTHN_RP_ORIGIN` 是否与实际前端访问地址一致。
- 当前是否通过 `http://localhost:5173` 访问前端。

### Windows GNU Rust build fails

若使用 Rust GNU toolchain，后端依赖 `webauthn-rs` 的 OpenSSL 链路；缺少 OpenSSL / pkg-config 时，可能在 `openssl-sys` 或 `pkg-config` 阶段失败。按根目录 `README.md` 中的 MSYS2 依赖说明补齐后再重试。

## Operational notes

- 后端启动时会自动执行 `./migrations` 下的数据库迁移。
- 后端 CORS 当前允许的前端来源是 `http://localhost:5173`。
