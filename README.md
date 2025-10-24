# PriceCrowd 2.0 — запуск проекта

Проект состоит из фронтенда (Vue 3 + Vite + Tailwind) и бэкенда (Rust + Axum + MongoDB). Ниже — быстрый старт и настройки окружения.

## Требования

- Node.js 18+ (рекомендуется LTS)
- Rust (stable, cargo)
- Docker + Docker Compose (для MongoDB; можно использовать локальный MongoDB без Docker)

## Быстрый старт

1) Поднять базу данных (MongoDB)
- Команда: `docker compose up -d`
- По умолчанию MongoDB доступен на `mongodb://localhost:27017`
- Включён Mongo Express (UI): http://localhost:8081 (опционально)

2) Запустить бэкенд (API)
- Скопируйте пример переменных окружения:
  - Windows (PowerShell): `Copy-Item backend/.env.example backend/.env`
  - Linux/macOS: `cp backend/.env.example backend/.env`
- Запустите сервер:
  - `cd backend`
  - `cargo run`
- По умолчанию API слушает `http://localhost:8080`

3) Запустить фронтенд (SPA)
- Скопируйте пример переменных окружения:
  - Windows (PowerShell): `Copy-Item frontend/.env.example frontend/.env`
  - Linux/macOS: `cp frontend/.env.example frontend/.env`
- Проверьте `VITE_API_URL` (по умолчанию `http://localhost:8080`)
- Установите зависимости и запустите дев‑сервер:
  - `cd frontend`
  - `npm install`
  - `npm run dev`
- Откройте адрес, который выведет Vite (обычно `http://localhost:5173`).

## Конфигурация окружения

Backend (`backend/.env`):
- `MONGODB_URI` — URI MongoDB, по умолчанию `mongodb://localhost:27017`
- `DATABASE_NAME` — имя базы, по умолчанию `pricecrowd`
- `PORT` — порт API, по умолчанию `8080`
- `UPLOADS_DIR` — папка для загруженных файлов, по умолчанию `uploads`

Frontend (`frontend/.env`):
- `VITE_API_URL` — базовый URL API, по умолчанию `http://localhost:8080`

## Эндпоинты API

Здоровье:
- `GET /healthz`

Товары (Products):
- `GET /products` — список
- `GET /products/:id` — один товар
- `POST /products` — создать (JSON: `{title, price, desc, image_url?}`)
- `PUT /products/:id` — изменить (JSON: частичное `{title?, price?, desc?, image_url?}`)
- `DELETE /products/:id` — удалить

Магазины (Stores):
- `GET /stores` — список
- `GET /stores/:id` — один магазин
- `POST /stores` — создать (JSON: `{name, addr, desc?, image_url?}`)
- `PUT /stores/:id` — изменить (JSON: частичное `{name?, addr?, desc?, image_url?}`)
- `DELETE /stores/:id` — удалить

Загрузка файлов:
- `POST /upload` — `multipart/form-data` с полем `file`; ответ: `{"url": "/uploads/<имя>"}`
- Статическая раздача загруженных файлов: `GET /uploads/<имя>`

## Работа с изображениями

- В форме добавления/редактирования (товары/магазины) можно выбрать файл — фронтенд отправит его на `POST /upload` и подставит `image_url` из ответа.
- Если `image_url` начинается с `/uploads/...`, фронтенд автоматически строит абсолютную ссылку через `VITE_API_URL`.
- Плейсхолдер: `frontend/public/placeholder-can.svg`

## Страницы фронтенда

- Список товаров: `/products` (карточки, поиск, удаление, переход к деталям/редактированию)
- Товар: `/products/:id` (детальная страница)
- Добавить/изменить товар: `/products/new`, `/products/:id/edit`
- Список магазинов: `/stores`
- Магазин: `/stores/:id` (детальная страница с блоками, аналогично примеру)
- Добавить/изменить магазин: `/stores/new`, `/stores/:id/edit`

## Частые проблемы

- API не отвечает: проверьте, что бэкенд запущен на порту, указанном в `VITE_API_URL`, и не блокируется фаерволом.
- База недоступна: проверьте, что контейнер Mongo запущен (`docker compose ps`) и переменная `MONGODB_URI` корректна.
- Картинки не видны: убедитесь, что `image_url` указывает либо на `http(s)://...`, либо на `/uploads/...` и что `UPLOADS_DIR` существует (сервер создаёт автоматически).

## Полезные файлы

- Фронтенд: `frontend/README.md:1`
- Бэкенд: `backend/README.md:1`
- Compose: `docker-compose.yml:1`

