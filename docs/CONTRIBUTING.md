# Contributing

## Development prerequisites

- Rust toolchain（后端）
- Node.js / npm（前端）
- Docker Desktop 或可用的 Docker Compose 环境（PostgreSQL / Redis）
- Windows GNU toolchain 用户若构建后端，需要额外准备 OpenSSL / pkg-config 依赖，详见根目录 `README.md`

## Local development setup

1. 启动基础设施：`docker compose up -d`
2. 复制后端环境变量：`cp backend/.env.example backend/.env`
3. 启动后端：`cd backend && cargo run`
4. 启动前端：`cd frontend && npm install && npm run dev`

## Available commands

<!-- AUTO-GENERATED:BEGIN commands -->
### Makefile

| Command | Description |
|---------|-------------|
| `make infra` | 启动 PostgreSQL 和 Redis。 |
| `make infra-down` | 停止基础设施。 |
| `make dev-backend` | 启动后端开发服务。 |
| `make dev-frontend` | 启动前端开发服务。 |
| `make build-backend` | 构建后端。 |
| `make build-frontend` | 构建前端。 |
| `make test-backend` | 运行后端测试。 |
| `make test-frontend` | 运行前端 Playwright 测试。 |

### Frontend scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | 启动 Vite 开发服务器。 |
| `npm run build` | 执行 TypeScript 构建并产出前端静态资源。 |
| `npm run lint` | 运行 ESLint。 |
| `npm test` | 运行前端 Playwright 测试。 |
| `npm run preview` | 本地预览构建产物。 |
<!-- AUTO-GENERATED:END commands -->

## Testing notes

- 后端可通过 `cd backend && cargo test` 运行测试。
- 前端可通过 `cd frontend && npm test` 运行 Playwright 测试。

## Pull request checklist

- 确认文档与代码一致，尤其是端口、路由、环境变量和脚本。
- 若修改 API 路由，更新 `docs/API.md`。
- 若修改环境变量，更新 `docs/ENV.md`。
- 若修改开发流程或命令，更新本文件和必要的运行文档。
