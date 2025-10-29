**Обзор**
- Бэкенд на Rust (Axum) работает с MongoDB через официальный драйвер `mongodb`.
- Все доступы к БД инкапсулированы в состоянии приложения `AppState` и обработчиках HTTP.
- Схема документ-ориентированная (MongoDB), миграции не используются; структура коллекций описана через Rust-структуры.

**Подключение**
- Инициализация подключения выполняется в `backend/src/state.rs:20`.
  - `MONGODB_URI` — строка подключения, по умолчанию `mongodb://localhost:27017`.
  - `DATABASE_NAME` — имя базы, по умолчанию `pricecrowd`.
  - `JWT_SECRET` — секрет для JWT (не влияет на соединение с БД, но хранится в состоянии).
- Разбор URI и создание клиента: `ClientOptions::parse(...)` и `Client::with_options(...)` в `backend/src/state.rs:24`.
- Получение `Database` и коллекций: `db.collection("<name>")` в `backend/src/state.rs:29`.

**Состояние приложения (`AppState`)**
- Хранит активные коллекции и ссылку на базу:
  - `products`, `stores`, `categories`, `store_items`, `store_activities` — типы `mongodb::Collection<T>`.
  - `db` — `mongodb::Database` для произвольного доступа (например, коллекция `users`).
  - Объявление: `backend/src/state.rs:9`.

**Коллекции и модели**
- `products` — товары, `backend/src/models.rs:4`
  - Поля: `_id:ObjectId?`, `title:String`, `desc:String`, `image_url:Option<String>`, `category_ids:Vec<ObjectId>`.
- `stores` — магазины, `backend/src/models.rs:37`
  - Поля: `_id:ObjectId?`, `name:String`, `addr:String`, `desc:String`, `image_url:Option<String>`.
- `categories` — категории, `backend/src/models.rs:64`
  - Поля: `_id:ObjectId?`, `name:String`, `desc:String`, `parent_ids:Vec<ObjectId>`.
- `store_items` — наличие и цены товара в магазине, `backend/src/models.rs:92`
  - Поля: `_id:ObjectId?`, `store_id:ObjectId`, `product_id:ObjectId`, `price:f64`.
- `store_activities` — журнал событий по товарам/ценам, `backend/src/models.rs:101`
  - Поля: `_id:ObjectId?`, `store_id:ObjectId`, `product_id:Option<ObjectId>`, `kind:String` (например, `item_added|price_updated|item_removed`), `ts_ms:i64`, `price:Option<f64>`, `product_name:Option<String>`, `store_name:Option<String>`.
- `users` — пользователи (используется для аутентификации), `backend/src/models.rs:131`
  - Поля: `_id:ObjectId?`, `username:String`, `password_hash:String`, `role:String` (`admin|user`).

**CRUD‑паттерны и запросы**
- Продукты (`backend/src/handlers/products.rs`):
  - Список: `find(None)` → потоковым курсором векторизуется в ответ.
  - Получение: `find_one({"_id": <oid>})`.
  - Создание: `insert_one(product)` затем возврат созданного документа через `find_one` по вставленному `_id`.
  - Обновление: сбор `$set` из непустых полей, `find_one_and_update({"_id"}, {"$set": ...})` с возвратом обновлённого документа.
  - Удаление: `delete_one({"_id"})`.
- Категории (`backend/src/handlers/categories.rs`) — аналогично продуктам; `parent_ids` маппятся из строк в `ObjectId`.
- Магазины (`backend/src/handlers/stores.rs`) — аналогично; дополнительно операции над товарами магазина:
  - Список товаров магазина: `store_items.find({"store_id": <oid>})`, далее подгружаются продукты по `{"_id": {"$in": [...]}}`.
  - Добавление товара в магазин (upsert): `update_one(filter={store_id,product_id}, update={"$set": {price}, "$setOnInsert": {...}}, upsert=true)` с записью активности в `store_activities` (тип `item_added`).
  - Обновление цены: `update_one({store_id,product_id}, {"$set": {price}})` + запись активности `price_updated`.
  - Удаление товара из магазина: `delete_one({store_id,product_id})` + запись активности `item_removed`.

**Аналитика и агрегации**
- Продуктовая аналитика (`GET /products/:id/insights`, `backend/src/handlers/insights.rs:12`):
  - Текущее состояние: читаются `store_items` для продукта и имена магазинов.
  - История цен: читаются `store_activities` по `product_id` с сортировкой по `ts_ms`.
  - Среднее по городу/минимум/максимум: считается в приложении из загруженных документов (MongoDB агрегирующие пайплайны не используются).
- Аналитика по магазину (`GET /stores/:id/products/insights`, `backend/src/handlers/insights.rs:38`):
  - Читаются `store_items` текущего магазина, продукты по списку `product_id`, затем все `store_items` по этим `product_id` для расчёта средних/минимумов.
  - История цен для пары (магазин, продукт) — из `store_activities` с сортировкой по `ts_ms`.

**Аутентификация и пользователи**
- Вход (`backend/src/handlers/auth.rs:11`): чтение `users.find_one({username})`, проверка `password_hash` (Argon2), выпуск JWT.
- Middleware администратора: валидация JWT и проверка `role == "admin"` (`backend/src/handlers/auth.rs:25`).

**Сидирование администратора**
- При старте вызывается `seed_admin(...)` (`backend/src/state.rs:36`).
  - При наличии `ADMIN_USERNAME` и `ADMIN_PASSWORD` создаётся пользователь-админ в коллекции `users` (если не существует), пароль хэшируется Argon2.

**Переменные окружения**
- Бэкенд (`backend/.env`):
  - `MONGODB_URI` — URI MongoDB, по умолчанию `mongodb://localhost:27017`.
  - `DATABASE_NAME` — имя базы, по умолчанию `pricecrowd`.
  - `JWT_SECRET` — секрет для JWT; по умолчанию `dev-secret-change-me` (замените в проде).
  - `ADMIN_USERNAME`, `ADMIN_PASSWORD` — опционально, для одноразового сидирования администратора.
  - `PORT` — порт API, по умолчанию `8080`.

**Запуск MongoDB (Docker Compose)**
- Файл: `docker-compose.yml:1`.
  - Сервисы: `mongo` (порт `27017:27017`) и опционально `mongo-express` (порт `8081:8081`).
  - Значение `MONGO_INITDB_DATABASE: pricecrowd` создаёт базу по умолчанию.

**Индексы (рекомендации)**
- В коде явного создания индексов нет; для производительности стоит добавить:
  - `store_items`: составные индексы на `{store_id: 1, product_id: 1}` (уникальный, если логически один документ на пару) и отдельные на `{product_id: 1}` и `{store_id: 1}`.
  - `store_activities`: индексы на `{product_id: 1, ts_ms: 1}` и `{store_id: 1, ts_ms: 1}` для истории и выборок по времени.
  - `users`: уникальный индекс на `{username: 1}`.
  - При необходимости: `products.name`/`categories.name` для поиска по имени.

**Жизненный цикл данных**
- Инициализация: подключение к MongoDB, получение коллекций, опциональное сидирование администратора.
- Работа: CRUD‑операции над `products`, `categories`, `stores`; поддержание `store_items`; логирование событий в `store_activities`.
- Завершение: явного закрытия соединений не требуется (управляется рантаймом Tokio/драйвером MongoDB).

**Где смотреть в коде**
- Подключение и состояние: `backend/src/state.rs:1`
- Маршруты и защита админ‑API: `backend/src/routes.rs:1`
- Обработчики CRUD: `backend/src/handlers/*.rs`
- Модели документов: `backend/src/models.rs:1`

