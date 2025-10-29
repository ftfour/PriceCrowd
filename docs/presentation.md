# Презентация: Как база данных работает в PriceCrowd 2.0

— формат: «слайды» в Markdown, удобно импортировать в Notion/Marp/Reveal/Google Slides

---

## Слайд 1 — Контекст и цель
- Проект: SPA (Vue 3) + Backend (Rust, Axum) + MongoDB
- Цель: показать, как хранятся и обрабатываются данные: модель, связи, операции, аналитика
- Роль MongoDB: основное хранилище документов (NoSQL)

Заметки докладчика:
- Уточнить, что это v2.0, без миграций, акцент на простоте и скорости

---

## Слайд 2 — Высокоуровневая архитектура
- Клиент (Vue) вызывает REST API (Axum)
- Backend подключается к MongoDB драйвером `mongodb`
- Коллекции: products, stores, categories, store_items, store_activities, users

Ссылки на код:
- Инициализация: `backend/src/main.rs:1`, `backend/src/state.rs:1`
- Роутинг: `backend/src/routes.rs:1`

Диаграмма (текст):
Vue ←HTTP→ Axum → MongoDB (коллекции: products, stores, ...)

---

## Слайд 3 — Подключение к БД и состояние
- Переменные окружения: `MONGODB_URI`, `DATABASE_NAME`, `JWT_SECRET`
- Создание клиента: `ClientOptions::parse(...)`, `Client::with_options(...)`
- Синглтон‑состояние `AppState` хранит коллекции и `Database`

Ссылки на код:
- `backend/src/state.rs:20` — чтение env и подключение
- `backend/src/state.rs:9` — структура `AppState`

---

## Слайд 4 — Модель данных (коллекции)
- products: товар (+ категории)
- stores: магазин
- categories: иерархия категорий (массив `parent_ids`)
- store_items: цена товара в конкретном магазине
- store_activities: события (добавлен товар, изм. цены, удалён)
- users: логины/роли (для админ‑API)

Ссылки на код моделей: `backend/src/models.rs:1`

---

## Слайд 5 — Пример схем (поля)
- Product: `_id?`, `title`, `desc`, `image_url?`, `category_ids[]`
- Store: `_id?`, `name`, `addr`, `desc`, `image_url?`
- StoreItem: `_id?`, `store_id`, `product_id`, `price`
- StoreActivity: `_id?`, `store_id`, `product_id?`, `kind`, `ts_ms`, `price?`, `product_name?`, `store_name?`
- User: `_id?`, `username`, `password_hash`, `role`

Ссылки: `backend/src/models.rs:4, 37, 92, 101, 131`

---

## Слайд 6 — CRUD: продукты/категории/магазины
- Список: `find(None)` курсором → вектор → JSON ответ
- Получение по `_id`: `find_one({"_id": oid})`
- Создание: `insert_one(...)` → повторный `find_one` по вставленному id
- Обновление: формируем `$set` из непустых полей → `find_one_and_update`
- Удаление: `delete_one`

Ссылки на код:
- Продукты: `backend/src/handlers/products.rs:1`
- Категории: `backend/src/handlers/categories.rs:1`
- Магазины: `backend/src/handlers/stores.rs:1`

---

## Слайд 7 — Связки: товары в магазине
- Храним цену и наличие в `store_items` (ключи: `store_id`, `product_id`)
- Добавление/обновление цены: `update_one(..., upsert=true)`
- Удаление товара из магазина: `delete_one({store_id, product_id})`

Ссылки:
- Upsert: `backend/src/handlers/stores.rs:134`
- Получение списка товаров магазина + сведения о продуктах: `stores.rs` (список, `$in` по `product_id`)

---

## Слайд 8 — Журнал активностей (аудит)
- На каждое действие с товаром в магазине записываем событие в `store_activities`
- Типы: `item_added`, `price_updated`, `item_removed`
- Храним `ts_ms`, цену (если применимо), имена товара/магазина для быстрого вывода

Ссылки:
- Добавление: `backend/src/handlers/stores.rs` (после upsert)
- Обновление цены/удаление: `backend/src/handlers/stores.rs` (log + 204)
- Чтение списка активностей: `backend/src/handlers/activities.rs:1`

---

## Слайд 9 — Аналитика: инсайты по продукту
- Текущие цены по магазинам: читаем `store_items` для `product_id`
- Названия магазинов: запрос в `stores` по `$in` списку `_id`
- История цен: `store_activities.find({product_id}).sort({ts_ms:1})`
- Городские метрики: min/max/avg считаются в приложении

Ссылка: `backend/src/handlers/insights.rs:12`

---

## Слайд 10 — Аналитика: инсайты по магазину
- Берём все `store_items` магазина → список `product_id`
- Подтягиваем продукты для названий/картинок
- Для городских средних и «самого дешёвого» читаем все `store_items` по этим `product_id`
- История цен в этом магазине: `store_activities` по `(store_id, product_id)`, сортировка `ts_ms`

Ссылка: `backend/src/handlers/insights.rs:38`

---

## Слайд 11 — Аутентификация и роль admin
- Логин: `users.find_one({username})`, проверка Argon2 `password_hash`
- JWT c `sub`, `role`, `exp`; хранится только на клиенте
- Middleware: проверка подписи и `role == admin` для админ‑эндпоинтов

Ссылки: `backend/src/handlers/auth.rs:1`

---

## Слайд 12 — Переменные окружения и запуск
- Backend (`backend/.env`): `MONGODB_URI`, `DATABASE_NAME`, `JWT_SECRET`, `ADMIN_USERNAME/PASSWORD`, `PORT`
- MongoDB в Docker: `docker-compose.yml:1` (mongo + mongo-express)
- Сидирование админа: выполняется на старте при наличии `ADMIN_*`

Ссылки: `backend/src/state.rs:36`, `docker-compose.yml:1`, `backend/README.md:1`

---

## Слайд 13 — Индексы (рекомендации)
- `store_items`: уникальный `{store_id:1, product_id:1}`; и одиночные `{product_id:1}`/`{store_id:1}`
- `store_activities`: `{product_id:1, ts_ms:1}` и `{store_id:1, ts_ms:1}`
- `users`: уникальный `{username:1}`
- Опционально: поиск по `products.title` (текстовый индекс)

Идея: создать `ensure_indexes()` при старте (не реализовано в коде сейчас)

---

## Слайд 14 — Демонстрация (curl)
1) Создать товар:
```
curl -X POST http://localhost:8080/products \
  -H "Authorization: Bearer <ADMIN_JWT>" \
  -H "Content-Type: application/json" \
  -d '{"title":"Кола 0.33л","desc":"Ж/б","image_url":null,"category_ids":[]}'
```
2) Добавить в магазин и поставить цену:
```
curl -X POST http://localhost:8080/stores/<storeId>/products \
  -H "Authorization: Bearer <ADMIN_JWT>" \
  -H "Content-Type: application/json" \
  -d '{"product_id":"<productId>","price":59.9}'
```
3) Посмотреть инсайты товара:
```
curl http://localhost:8080/products/<productId>/insights
```

---

## Слайд 15 — Что дальше
- Добавить индексацию и валидацию схем (JSON Schema) для коллекций
- Вынести часть аналитики в Mongo агрегирующие пайплайны (`$lookup`, `$group`, `$avg/$min/$max`)
- Пагинация/фильтры в API, ограничение CORS доменами
- Интеграционные тесты с поднятой Mongo (testcontainers)

---

## Приложение — Быстрые ссылки на код
- Состояние/подключение: `backend/src/state.rs:1`
- Роутинг/гварды: `backend/src/routes.rs:1`
- Обработчики CRUD: `backend/src/handlers/*.rs`
- Модели: `backend/src/models.rs:1`
- Compose для Mongo: `docker-compose.yml:1`

