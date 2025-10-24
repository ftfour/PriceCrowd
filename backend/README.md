# PriceCrowd Backend (Rust + Axum + MongoDB)

Provides CRUD API for products stored in MongoDB.

## Endpoints

- `GET /healthz` — health check
- `GET /products` — list products
- `POST /products` — create product
- `PUT /products/:id` — update product
- `DELETE /products/:id` — delete product

## Running locally

1. Create `.env` from example:
   - `cp .env.example .env` (or copy manually on Windows)
2. Start MongoDB (Docker recommended): see `docker-compose.yml` below.
3. Run backend:
   - `cargo run` (inside `backend`)

## Docker Compose (MongoDB)

A simple Compose file is included at the repo root to start Mongo and Mongo Express.

- MongoDB: `mongodb://localhost:27017`
- Mongo Express UI: http://localhost:8081 (optional)
