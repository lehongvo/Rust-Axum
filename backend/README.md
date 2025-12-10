# Rust Axum E-commerce API

Backend demo using Axum + SeaORM + PostgreSQL với API key + JWT + rate limit.

## Cấu trúc
- Mã nguồn + Docker Compose nằm trong thư mục này
- `migrations/`: schema SQL (users, products, orders, historys)

## Chạy local (không Docker)
1) Cài Rust + Postgres. Tạo DB:
   ```bash
   createdb app
   ```
2) Tạo file `.env` (xem biến cần thiết, có sẵn mẫu trong `env.example`):
   ```
   DATABASE_URL=postgres://app:app@localhost:5432/app   # nếu chạy backend trên host
   # Nếu chạy backend bên trong docker-compose, dùng: postgres://app:app@db:5432/app
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
- `POST /orders` – tạo đơn `{ "user_id": "...", "product_id": "...", "quantity": 1 }` (API key + JWT, ghi lại history)

## Database
Schema: users -> orders -> products, cùng bảng historys (log mỗi order). Migrations ở `migrations` và được thực thi khi app khởi động.

## Makefile
- `make install|build|lint|format|test|run`
- `make docker-up` / `make docker-down`