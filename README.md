# Multi-Subject Auth System

多主体认证系统，支持 Member / Staff / Admin 三种角色的注册、登录、TOTP 双因素认证、Passkey (WebAuthn) 和会话管理。

## 技术栈

| 层 | 技术 |
|---|------|
| 后端 | Rust + Axum + SQLx |
| 前端 | React 19 + TypeScript + Vite + Ant Design |
| 数据库 | PostgreSQL 16 |
| 缓存 | Redis 7 (会话 + MFA token) |
| 认证 | JWT + TOTP + WebAuthn |

## 架构

```
backend/src/
  domain/         # 领域模型、仓储接口、错误定义
  application/    # DTO、Service 层（业务逻辑）
  infrastructure/ # 仓储实现(PostgreSQL)、密码哈希、JWT
  presentation/   # Handler、Router、中间件(Claims 提取)
```

采用分层架构，handler → service → repository，依赖方向单向向内。

## 快速启动

### Windows GNU 环境补充（fresh machine 必需）

如果你在 Windows 上使用 Rust GNU toolchain（例如 `x86_64-pc-windows-gnu`），由于 `webauthn-rs` 会通过 OpenSSL 依赖链参与编译，新的机器需要先补齐 GNU 侧依赖，否则后端构建会在 `openssl-sys` / `pkg-config` 阶段失败。

推荐通过 MSYS2 安装：

```bash
pacman -Syu
pacman -Su
pacman -S mingw-w64-ucrt-x86_64-openssl mingw-w64-ucrt-x86_64-pkgconf
```

本地运行后端前，确保构建环境能解析到 MSYS2 的 OpenSSL / pkg-config（例如把 `E:/msys64/ucrt64/bin` 放进 PATH，并让 `OPENSSL_DIR` / `OPENSSL_LIB_DIR` / `OPENSSL_INCLUDE_DIR` 指向对应目录）。

```bash
# 1. 确保 Docker Desktop / Docker daemon 已启动

# 2. 启动基础设施
docker compose up -d

# 3. 配置环境变量
cp backend/.env.example backend/.env

# 4. 在第一个终端启动后端 (自动执行数据库迁移；依赖 PostgreSQL 和 Redis 已可连接)
cd backend && cargo run
```

```bash
# 5. 在第二个终端启动前端
cd frontend && npm install && npm run dev
```

- 前端默认地址：`http://localhost:5173`
- 后端默认地址：`http://localhost:3000`
- 后端必须单独启动；前端测试 / `npm test` 不会自动启动后端
- Playwright 只会根据 `frontend/playwright.config.ts` 自动启动前端 dev server

## 前端页面

| 路由 | 说明 |
|------|------|
| `/member/login` | Member 登录 / 注册 / Passkey 登录 |
| `/staff/login` | Community Staff 登录 / 注册 / Passkey 登录 |
| `/admin/login` | Platform Staff 登录 / 注册 / Passkey 登录 |
| `/dashboard` | 当前登录用户资料与凭证状态、TOTP 设置 |
| `/sessions` | 当前用户会话列表与会话撤销 |

根路径 `/` 会自动跳转到 `/member/login`。

## API 端点

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | /api/health | - | 健康检查 |
| POST | /api/subjects/register | - | 注册 |
| GET | /api/subjects/me | Bearer | 当前用户信息 |
| POST | /api/auth/{subject_type}/login | - | 登录 (subject_type: member/staff/admin) |
| POST | /api/auth/mfa/verify | - | MFA 验证 |
| POST | /api/auth/logout | Bearer | 登出 |
| GET | /api/credentials/status | Bearer | 凭证状态 |
| POST | /api/credentials/totp/setup | Bearer | 生成 TOTP 绑定信息 |
| POST | /api/credentials/totp/confirm | Bearer | 确认并启用 TOTP |
| POST | /api/credentials/totp/verify | Bearer | 验证 TOTP |
| POST | /api/credentials/passkey/register-begin | Bearer | 发起 Passkey 注册 |
| POST | /api/credentials/passkey/register-complete | Bearer | 完成 Passkey 注册 |
| POST | /api/credentials/passkey/authenticate-begin | Bearer | 发起已登录状态 Passkey 验证 |
| POST | /api/credentials/passkey/authenticate-complete | Bearer | 完成已登录状态 Passkey 验证 |
| POST | /api/auth/passkey/begin | - | 发起 Passkey 登录 |
| POST | /api/auth/passkey/complete | - | 完成 Passkey 登录 |
| GET | /api/sessions | Bearer | 会话列表 |
| DELETE | /api/sessions/{session_id} | Bearer | 撤销会话 |

## 环境变量

| 变量 | 必填 | 说明 | 示例 |
|------|------|------|------|
| `DATABASE_URL` | 是 | PostgreSQL 连接串 | `postgres://postgres:postgres@localhost:5432/auth_system` |
| `REDIS_URL` | 是 | Redis 连接串 | `redis://127.0.0.1:6379` |
| `JWT_SECRET` | 是 | JWT 签名密钥 | `change-me-to-a-random-secret` |
| `RUST_LOG` | 否 | Rust 日志级别过滤 | `auth_system=debug,tower_http=debug` |
| `WEBAUTHN_RP_ID` | 是 | WebAuthn relying party ID | `localhost` |
| `WEBAUTHN_RP_ORIGIN` | 是 | WebAuthn relying party origin | `http://localhost:5173` |

## 常用命令

| 命令 | 说明 |
|------|------|
| `docker compose up -d` | 启动 PostgreSQL 和 Redis |
| `docker compose down` | 停止基础设施 |
| `cd backend && cargo run` | 启动后端 |
| `cd backend && cargo build` | 构建后端 |
| `cd backend && cargo test` | 运行后端测试 |
| `cd frontend && npm test` | 运行前端 Playwright 测试 |
| `cd frontend && npm run dev` | 启动前端开发服务器 |
| `cd frontend && npm run build` | 构建前端 |
| `cd frontend && npm run lint` | 运行前端 ESLint |

## 更多文档

- [环境变量说明](docs/ENV.md)
- [API 参考](docs/API.md)
- [开发贡献指南](docs/CONTRIBUTING.md)
- [运行手册](docs/RUNBOOK.md)

## 设计决策

- **多主体分离**: 同一用户名可在不同角色下注册，通过 `subject_type` 区分
- **JWT + Redis 双层会话**: JWT 无状态验证 + Redis 支持即时撤销
- **TOTP 双因素**: 登录时检测是否启用 TOTP，启用则返回 `mfa_token` 要求二次验证
- **密码安全**: Argon2 哈希，token 存储使用 SHA-256 哈希

## Token 账单（近半年）

![Token 账单 1](docs/images/token-screenshot-1.png)

![Token 账单 2](docs/images/token-screenshot-2.png)
