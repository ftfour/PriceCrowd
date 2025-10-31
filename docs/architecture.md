# Архитектура системы PriceCrowd 2.0

Этот документ описывает ключевые компоненты, связи между ними и основные потоки данных системы.

## Обзор
- Назначение: сбор и сравнение цен товаров по магазинам, базовые рейтинги и аналитика.
- Стек:
  - Frontend: Vue 3 + Vite + Tailwind (`frontend/`)
  - Backend: Rust + Axum (`backend/`)
  - База данных: MongoDB (в Docker через `docker-compose.yml`)
  - Интеграция: Telegram Bot (polling и/или webhook)
  - Статика: каталог загрузок (`/uploads`)

## Общая схема
```mermaid
flowchart LR
  subgraph Client
    U[Browser (Vue SPA)]
  end

  subgraph Server
    A[Axum API (Rust)]
    FS[(Uploads dir)]
  end

  DB[(MongoDB)]
  TG>Telegram API]

  U -- REST/JSON --> A
  A -- CRUD/Queries --> DB
  A -- Serve static --> FS
  TG -- webhook --> A
  A -- long polling --> TG
```

## Компоненты
- Frontend (Vue SPA)
  - Роутинг: `frontend/src/router/index.ts` (публичные и админ-маршруты).
  - Аутентификация: хранение JWT в `localStorage`, реактивное состояние в `frontend/src/auth.ts`.
  - Взаимодействие с API: базовый URL из `VITE_API_URL` (`frontend/src/api.ts`).

- Backend (Axum)
  - Точка входа: `backend/src/main.rs` — инициализация логгера, чтение env, сборка роутов, запуск HTTP.
  - Маршрутизация: `backend/src/routes.rs` — публичные и админские endpoints, CORS, статика `/uploads`.
  - Состояние приложения: `backend/src/state.rs` — коннект к MongoDB, коллекции, `jwt_secret`, сидирование админа.
  - Модели/DTO: `backend/src/models.rs` — `Product`, `Store`, `Category`, `StoreItem`, `StoreActivity`, `User`, и др.
  - Обработчики: `backend/src/handlers/*.rs` — CRUD, логин/JWT, загрузки файлов, инсайты и т.д.
  - Интеграция Telegram: `backend/src/telegram/` — webhook, long polling, линковка аккаунтов по коду.

- База данных (MongoDB)
  - Коллекции: `products`, `stores`, `categories`, `store_items`, `store_activities`, `users`, `settings`, `telegram_links`.
  - Схемы и связи описаны в `docs/database.md` и отражены в `backend/src/models.rs`.

- Файловое хранилище
  - Каталог загрузок задаётся env `UPLOADS_DIR` (по умолчанию `uploads`), раздаётся как `/uploads/*`.

## Маршруты и зоны доступа
- Публичные
  - `GET /healthz`
  - `POST /auth/login` — получение JWT
  - Чтение каталогов и карточек: `/products`, `/categories`, `/stores`, инсайты и активности
  - `POST /telegram/webhook` — если включён webhook-режим
  - Раздача статики `/uploads/*`
- Админ (middleware `require_admin` по JWT/role)
  - Управление справочниками: `POST/PUT/DELETE /products|stores|categories`
  - Привязка категорий к товарам
  - Управление номенклатурой магазина и ценами: `/stores/:id/products`
  - Загрузка файлов: `POST /upload`
  - Пользователи: `GET/POST/PUT/DELETE /users`
  - Настройки Telegram: `GET/PUT /settings/telegram`, статус `GET /settings/telegram/status`

Сборка роутов — `backend/src/routes.rs`.

## Аутентификация и авторизация
- Пароли пользователей хэшируются Argon2 (см. `auth.rs`).
- Логин возвращает JWT с payload: `sub`, `role`, `exp`.
- Админ-гвард декодирует и валидирует JWT; доступ разрешён только для `role=admin`.
- Frontend хранит `token/username/role` в `localStorage`, проверяет `exp` при старте.

## Интеграция с Telegram
- Режимы:
  - Long polling: фоновая задача (`spawn_poller`) опрашивает `getUpdates` при `enabled=true` и наличии `token`.
  - Webhook: `POST /telegram/webhook` принимает апдейты при `webhook_enabled=true`.
- Линковка аккаунтов: пользователь посылает боту код (`/link ABC123`), бекенд отмечает `users.telegram_id` и помечает код использованным (`telegram_links`).
- Статус/логи бота доступны администратору (`/settings/telegram/status`).

## Конфигурация окружения
- Backend (`backend/.env`):
  - `MONGODB_URI` — URI MongoDB (по умолчанию `mongodb://localhost:27017`)
  - `DATABASE_NAME` — имя БД (по умолчанию `pricecrowd`)
  - `JWT_SECRET` — секрет для подписи JWT
  - `ADMIN_USERNAME`, `ADMIN_PASSWORD` — при наличии выполняется сид админа
  - `PORT` — порт HTTP (по умолчанию `8080`)
  - `UPLOADS_DIR` — каталог загрузок (по умолчанию `uploads`)
- Frontend (`frontend/.env`):
  - `VITE_API_URL` — базовый URL API
- Docker: `docker-compose.yml` поднимает MongoDB и Mongo Express (UI).

## Потоки данных (ключевые сценарии)
- Аутентификация:
  1) `POST /auth/login` -> проверка пароля (Argon2) -> выдача JWT -> FE сохраняет `token`.
- Управление товарами/магазинами/категориями (админ):
  - CRUD через соответствующие endpoints с заголовком `Authorization: Bearer <JWT>`.
- Цены и активности:
  - `store_items` хранит цену товара в магазине; изменения пишутся в `store_activities`.
  - Инсайты по товару/магазину — агрегирующие запросы (см. `handlers/insights.rs`).
- Загрузка файлов:
  - `POST /upload` (multipart) -> сохранение в `UPLOADS_DIR` -> клиент получает `url` для использования в `image_url`.

## Нефункциональные аспекты
- Безопасность: JWT/role-based доступ к административным операциям; CORS открыт для любого origin (можно сузить при деплое).
- Масштабирование: Stateless-бекенд (кроме локального `Uploads dir`), основное состояние в MongoDB; Telegram long polling масштабировать 1 экземпляром на токен.
- Логирование: `tracing_subscriber` c фильтром по env; простые логи по Telegram в памяти (ограничены `MAX_LOGS`).

## Развёртывание и разработка
- Локально:
  - `docker compose up -d` — MongoDB + Mongo Express
  - Backend: `cd backend && cp .env.example .env && cargo run`
  - Frontend: `cd frontend && cp .env.example .env && npm i && npm run dev`
- Прод: рекомендовано раздавать статику CDN/облачным сториджем, ограничить CORS, хранить секреты вне репозитория, настроить резервное копирование MongoDB и индексы.

## Связанные документы
- БД и модели: `docs/database.md`
- Введение/презентация: `docs/presentation.md`
