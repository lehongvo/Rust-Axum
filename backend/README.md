# Rust Axum E-commerce API

Backend demo using Axum + SeaORM + PostgreSQL với API key + JWT + rate limit.

## Cấu trúc
- `backend/`: mã nguồn dịch vụ API
- `docker-compose.yml`: chạy app + Postgres nhanh bằng Docker

## Chạy local (không Docker)
1) Cài Rust + Postgres. Tạo DB:
   ```bash
   createdb app
   ```
2) Tạo file `backend/.env` (xem biến cần thiết):
   ```
   DATABASE_URL=postgres://app:app@localhost:5432/app
   APP_PORT=3000
   API_KEY=changeme
   JWT_SECRET=super-secret
   ADMIN_USER=admin
   ADMIN_PASS=admin123
   RATE_LIMIT_PER_MINUTE=60
   RUST_LOG=info
   ```
3) Chạy server:
   ```bash
   cd backend
   cargo run
   ```

## Chạy với Docker Compose
```bash
docker-compose up --build
```

Compose tạo Postgres (user/pass/db: `app`) và service `api` expose cổng `3000`.

## API đơn giản
- `GET /health` – không auth
- `POST /login` – nhận JWT (dùng `ADMIN_USER/ADMIN_PASS`)
- `GET /products` – yêu cầu `X-API-KEY` + `Authorization: Bearer <jwt>`
- `POST /products` – tạo sản phẩm `{ "name": "...", "price_cents": 1000 }` (API key + JWT)
- `POST /orders` – tạo đơn `{ "product_id": "...", "quantity": 1 }` (API key + JWT)

## Database
Migrations SQL ở `backend/migrations` và được thực thi khi app khởi động (SeaORM chạy file `0001_init.sql`).

## Makefile
- `make install|build|lint|format|test|run`
- `make docker-up` / `make docker-down`