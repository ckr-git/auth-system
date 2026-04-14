.PHONY: dev dev-backend dev-frontend build test infra infra-down

infra:
	docker compose up -d

infra-down:
	docker compose down

dev-backend:
	cd backend && cargo run

dev-frontend:
	cd frontend && npm run dev

build-backend:
	cd backend && cargo build

build-frontend:
	cd frontend && npm run build

test-backend:
	cd backend && cargo test

test-frontend:
	cd frontend && npm test
