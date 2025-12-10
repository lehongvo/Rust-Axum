-- Reset tables for dev bootstrap
DROP TABLE IF EXISTS historys;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS users;

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Products table stores catalog items
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Orders table references products and users
CREATE TABLE orders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity INT NOT NULL CHECK (quantity > 0),
    total_cents BIGINT NOT NULL CHECK (total_cents >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Historys table references orders
CREATE TABLE historys (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed sample data
INSERT INTO users (id, email) VALUES
    ('11111111-1111-1111-1111-111111111111', 'user1@example.com'),
    ('22222222-2222-2222-2222-222222222222', 'user2@example.com');

INSERT INTO products (id, name, price_cents) VALUES
    ('33333333-3333-3333-3333-333333333333', 'Sample Product A', 1000),
    ('44444444-4444-4444-4444-444444444444', 'Sample Product B', 2000);

INSERT INTO orders (id, user_id, product_id, quantity, total_cents) VALUES
    ('55555555-5555-5555-5555-555555555555', '11111111-1111-1111-1111-111111111111', '33333333-3333-3333-3333-333333333333', 1, 1000);

INSERT INTO historys (id, order_id, action) VALUES
    ('66666666-6666-6666-6666-666666666666', '55555555-5555-5555-5555-555555555555', 'created');
