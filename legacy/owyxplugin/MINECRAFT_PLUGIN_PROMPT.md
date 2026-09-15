# ПРОМПТ ДЛЯ РАЗРАБОТКИ MINECRAFT ПЛАГИНА ИНТЕГРАЦИИ С CHIWAWA САЙТОМ

## КОНТЕКСТ ПРОЕКТА

Ты - эксперт Java разработчик Minecraft плагинов (Spigot/Paper API). Нужно создать полнофункциональный плагин для интеграции Minecraft сервера с веб-сайтом ChiwawaMine.

### АРХИТЕКТУРА САЙТА:
- **Backend**: Node.js + Express.js + PostgreSQL
- **Порт**: 3000 (localhost/production)
- **База данных**: PostgreSQL 14.18 с UUID поддержкой
- **Аутентификация**: JWT токены + session management
- **API Endpoint**: `http://localhost:3000/api` (временно, после резила будет https://chiwawa.site/)

### СТРУКТУРА БАЗЫ ДАННЫХ:

#### Таблица `users`:
```sql
- id (INTEGER, PRIMARY KEY)
- nickname (VARCHAR(32)) // Minecraft ник
- email (VARCHAR(128))
- password_hash (VARCHAR(128))
- first_name, last_name (VARCHAR(50))
- role (VARCHAR(32): 'user'|'admin'|'moderator')
- trust_level (INTEGER 0-5): 0=Проходимец(10ч лимит), 1=Новичок, 2=Проверенный(25ч+почта+10реп), 3=Ветеран(50ч+почта+20реп)
- is_active, is_email_verified, is_banned (BOOLEAN)
- registered_at, last_login (TIMESTAMP)
- ban_reason (TEXT)
```

#### Таблица `player_stats`:
```sql
- user_id (INTEGER, FK users.id)
- total_minutes, time_played_minutes (INTEGER) // Общее время игры
- is_time_limited (BOOLEAN) // Ограничение времени
- current_level (INTEGER) // Игровой уровень
- email_verified, discord_verified, minecraft_verified (BOOLEAN)
- reputation (INTEGER) // Репутация
- achievements_count, total_logins, warnings_count (INTEGER)
```

#### Таблица `applications`:
```sql
- id, user_id (INTEGER)
- minecraft_nick, email, discord (VARCHAR)
- status ('pending'|'approved'|'rejected')
- motivation, plans (TEXT)
- submitted_at, reviewed_at (TIMESTAMP)
```

#### Таблица `user_activity`:
```sql
- user_id (INTEGER, FK)
- activity_type (VARCHAR): 'login', 'logout', 'join_server', 'leave_server', 'playtime_update'
- description (TEXT)
- metadata (JSONB) // Дополнительные данные
- created_at (TIMESTAMP)
```

## ОСНОВНЫЕ API ЭНДПОИНТЫ ДЛЯ ПЛАГИНА:

### АУТЕНТИФИКАЦИЯ:
- `POST /api/auth/login` - Вход (email, password → JWT token)
- `GET /api/auth/verify` - Проверка токена
- `POST /api/auth/logout` - Выход

### ПОЛЬЗОВАТЕЛИ:
- `GET /api/profile` - Получить профиль игрока (нужен Bearer token)
- `PUT /api/profile` - Обновить профиль
- `GET /api/admin/users` - Список пользователей (admin only)
- `PUT /api/admin/users/:id/ban` - Забанить игрока
- `PUT /api/admin/users/:id/unban` - Разбанить игрока
- `PUT /api/admin/users/:id/trust-level` - Изменить trust level

### СТАТИСТИКА И АКТИВНОСТЬ:
- `POST /api/admin/user-activity` - Записать активность игрока
- `GET /api/profile/activity` - История активности
- `PUT /api/admin/users/:id/playtime` - Обновить время игры

### НАСТРОЙКИ СЕРВЕРА:
- `GET /api/settings/public` - Публичные настройки (название, IP, Discord)
- `GET /api/server-info` - Статус сервера (онлайн, игроки)

### ЗАЯВКИ:
- `GET /api/applications/server-access` - Проверить доступ к серверу
- `GET /api/applications/:id/status` - Статус заявки по ID

## ТЕХНИЧЕСКИЕ ТРЕБОВАНИЯ:

### 1. ОСНОВНАЯ ФУНКЦИОНАЛЬНОСТЬ:

#### A) СИСТЕМА WHITELIST:
```java
// При подключении игрока проверяй:
// 1. Есть ли user с таким nickname в БД
// 2. Одобрена ли его заявка (applications.status = 'approved')
// 3. Не забанен ли (users.is_banned = false)
// 4. Активен ли аккаунт (users.is_active = true)
```

#### B) TRUST LEVEL СИСТЕМА:
```java
// Trust Level влияет на:
// 0 (Проходимец): Лимит 10 часов игры ВСЕГО до подтверждения email (НЕ дневной!)
// 1 (Новичок): Стандартный доступ, БЕЗ лимитов времени
// 2 (Проверенный): Расширенные возможности
// 3 (Ветеран): Полный доступ игрока
// 4-5: Админские функции
```

#### C) СИСТЕМА ВРЕМЕНИ ИГРЫ:
```java
// Трекинг:
// - Время входа/выхода
// - Общее время игры (time_played_minutes)
// - ТОЛЬКО для проходимцев (trust_level = 0): лимит 10 часов ВСЕГО
// - Автоматический кик при превышении лимита (только проходимцы)
// - Дневных лимитов НЕТ для других рангов!
```

#### D) СИСТЕМА РЕПУТАЦИИ:
```java
// Возможности:
// - Команды для +/- репутации (/rep <player> <amount>)
// - История изменений репутации
// - Влияние на Trust Level повышение
```

### 2. КОМАНДЫ ПЛАГИНА:

#### ADMIN КОМАНДЫ:
```java
/chiwawa reload - Перезагрузка конфига
/chiwawa sync <player> - Синхронизация данных игрока с сайтом
/chiwawa ban <player> <reason> - Бан через сайт
/chiwawa unban <player> - Разбан через сайт
/chiwawa trust <player> <level> - Изменить trust level
/chiwawa playtime <player> - Показать статистику времени
/chiwawa rep <player> <amount> [reason] - Изменить репутацию
/chiwawa whitelist add <player> - Добавить в whitelist (создает заявку)
/chiwawa users list [page] - Список пользователей с сайта
/chiwawa stats - Общая статистика сервера
```

#### ИГРОК КОМАНДЫ:
```java
/profile - Показать свой профиль с сайта
/playtime - Показать время игры
/rep <player> - Дать репутацию игроку
/applications - Статус своей заявки
/discord - Связать Discord аккаунт
/rules - Правила сервера
```

### 3. КОНФИГУРАЦИЯ (config.yml):

```yaml
# Настройки API
api:
  base_url: "http://localhost:3000/api"
  timeout: 5000
  admin_token: "ваш_админ_jwt_токен"
  retry_attempts: 3

# Настройки сервера
server:
  enable_whitelist: true
  enable_time_limit: true
  enable_trust_system: true
  enable_reputation: true
  sync_interval: 300 # секунд для синхронизации данных

# Trust Level ограничения
trust_levels:
  0: # Проходимец
    max_total_hours: 10 # ВСЕГО 10 часов, не дневной лимит!
    description: "Новый игрок без подтвержденного email"
  1: # Новичок  
    description: "Подтвержден email, полный доступ"
  2: # Проверенный
    description: "Опытный игрок с хорошей репутацией"
  3: # Ветеран
    description: "Проверенный временем игрок"

# Сообщения
messages:
  # Кик сообщения (показываются при кике с сервера)
  no_access_kick: |
    &c&l┌─────────────────────────────┐
    &c&l│    &f&lНЕТ ДОСТУПА К СЕРВЕРУ    &c&l│
    &c&l├─────────────────────────────┤
    &f&l│ &7У вас нет доступа к серверу  &f&l│
    &f&l│ &7Подайте заявку на сайте:      &f&l│
    &f&l│ &bhttps://chiwawa.site         &f&l│
    &c&l└─────────────────────────────┘
    
  time_limit_kick: |
    &c&l┌─────────────────────────────┐
    &c&l│    &f&lЛИМИТ ВРЕМЕНИ ИСЧЕРПАН   &c&l│
    &c&l├─────────────────────────────┤
    &f&l│ &7Вы достигли лимита в 10 часов &f&l│
    &f&l│ &7Подтвердите email на сайте:   &f&l│
    &f&l│ &bhttps://chiwawa.site         &f&l│
    &c&l└─────────────────────────────┘
    
  ban_kick: |
    &4&l┌─────────────────────────────┐
    &4&l│        &f&lВЫ ЗАБАНЕНЫ         &4&l│
    &4&l├─────────────────────────────┤
    &f&l│ &7Причина: &c%reason%          &f&l│
    &f&l│ &7Обжалование на сайте:        &f&l│
    &f&l│ &bhttps://chiwawa.site         &f&l│
    &4&l└─────────────────────────────┘
  
  # Чат сообщения (обычные сообщения в чат)
  trust_level_up: "&a&l[★] &fВаш Trust Level повышен до &a&l%level% &7(%name%)&f!"
  reputation_given: "&e&l[REP] &fВы дали &a+1 репутацию &fигроку &b%player%&f!"
  reputation_received: "&e&l[REP] &fИгрок &b%player% &fдал вам &a+1 репутацию&f!"
```

### 4. АРХИТЕКТУРА КЛАССОВ:

#### ОСНОВНЫЕ КЛАССЫ:
```java
// Main Plugin Class
public class ChiwawaPlugin extends JavaPlugin

// API Manager
public class ChiwawaAPI {
    // HTTP клиент для взаимодействия с сайтом
    // Методы: getUser(), updatePlaytime(), checkWhitelist(), etc.
}

// User Manager
public class UserManager {
    // Кеширование пользователей
    // Синхронизация с БД
    // Trust Level management
}

// Playtime Manager
public class PlaytimeManager {
    // Трекинг времени игры
    // Лимиты времени
    // Автоматические действия
}

// Reputation Manager 
public class ReputationManager {
    // Система репутации
    // История изменений
    // Валидация
}

// Command Handler
public class CommandManager {
    // Обработка всех команд
    // Проверка прав доступа
}

// Event Listeners
public class PlayerListener implements Listener {
    // Join/Quit события
    // Chat события
    // Другие игровые события
}
```

### 5. СОБЫТИЯ ДЛЯ ОБРАБОТКИ:

```java
// При входе игрока (PlayerJoinEvent):
// 1. Проверить whitelist через API
// 2. Загрузить данные пользователя
// 3. Применить Trust Level ограничения (только для проходимцев)
// 4. Записать активность в БД
// 5. Показать приветственное сообщение

// При выходе (PlayerQuitEvent):
// 1. Сохранить время игры
// 2. Записать активность выхода
// 3. Очистить кеш

// Каждые 5 минут (Scheduled Task):
// 1. Обновить время игры всех онлайн игроков
// 2. Проверить лимит ТОЛЬКО для проходимцев (trust_level = 0)
// 3. Синхронизировать данные с сайтом

// При выполнении команд (PlayerCommandPreprocessEvent):
// 1. Логировать важные команды
// 2. НЕ блокировать команды по Trust Level (есть другой плагин)
```

### 6. ИНТЕГРАЦИЯ С САЙТОМ:

#### HTTP ЗАПРОСЫ:
```java
// Используй OkHttp или встроенный HttpURLConnection
// Все запросы с Bearer токеном в заголовках
// Content-Type: application/json
// Обработка ошибок сети и таймаутов
// Кеширование ответов для производительности
```

#### СИНХРОНИЗАЦИЯ ДАННЫХ:
```java
// Кеш пользователей в памяти (Map<UUID, ChiwawaUser>)
// Периодическая синхронизация каждые 5 минут
// Принудительная синхронизация при входе игрока
// Batch операции для множественных обновлений
```

### 7. БЕЗОПАСНОСТЬ:

```java
// Валидация всех входящих данных от API
// Защита от SQL инъекций (используй PreparedStatement если свой SQL)
// Rate limiting для команд (/rep только раз в час на игрока)
// Логирование всех важных действий
// Проверка прав доступа для админ команд
```

### 8. ПРОИЗВОДИТЕЛЬНОСТЬ:

```java
// Асинхронные HTTP запросы (не блокируй main thread)
// Кеширование данных пользователей
// Bulk операции для обновления статистики
// Ленивая загрузка данных
// Оптимизация SQL запросов
```

## ПРИМЕРЫ КОДА:

### ПРОВЕРКА WHITELIST:
```java
public boolean isPlayerWhitelisted(String playerName) {
    try {
        HttpResponse<String> response = httpClient.send(
            HttpRequest.newBuilder()
                .uri(URI.create(apiUrl + "/applications/server-access"))
                .header("Authorization", "Bearer " + adminToken)
                .GET()
                .build(),
            HttpResponse.BodyHandlers.ofString()
        );
        
        if (response.statusCode() == 200) {
            JsonObject json = JsonParser.parseString(response.body()).getAsJsonObject();
            return json.get("hasAccess").getAsBoolean();
        }
    } catch (Exception e) {
        getLogger().severe("Ошибка проверки whitelist для " + playerName + ": " + e.getMessage());
    }
    return false;
}
```

### ОБНОВЛЕНИЕ ВРЕМЕНИ ИГРЫ:
```java
public void updatePlaytime(UUID playerUuid, int minutesPlayed) {
    CompletableFuture.runAsync(() -> {
        try {
            String json = String.format("{\"playtime_minutes\": %d}", minutesPlayed);
            
            HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(apiUrl + "/admin/users/" + getUserId(playerUuid) + "/playtime"))
                .header("Authorization", "Bearer " + adminToken)
                .header("Content-Type", "application/json")
                .PUT(HttpRequest.BodyPublishers.ofString(json))
                .build();
                
            httpClient.send(request, HttpResponse.BodyHandlers.ofString());
        } catch (Exception e) {
            getLogger().severe("Ошибка обновления времени игры: " + e.getMessage());
        }
    });
}
```

## ДОПОЛНИТЕЛЬНЫЕ ФИЧИ:

### 1. DISCORD ИНТЕГРАЦИЯ:
- Уведомления о входе/выходе игроков
- Синхронизация ролей Discord с Trust Level
- Команды Discord бота для управления сервером

### 2. СТАТИСТИКА:
- Топ игроков по времени игры
- Топ по репутации
- История активности игроков
- Графики использования сервера

### 3. АВТОМАТИЗАЦИЯ:
- Автоматическое повышение Trust Level при выполнении условий
- Автоматические награды за достижения
- Уведомления администраторов о важных событиях

## ТЕСТИРОВАНИЕ:

### UNIT ТЕСТЫ:
- Тестирование API методов
- Проверка валидации данных
- Тестирование Trust Level логики

### ИНТЕГРАЦИОННЫЕ ТЕСТЫ:
- Подключение к тестовой БД
- Проверка HTTP запросов
- Тестирование событий Minecraft

## РАЗВЕРТЫВАНИЕ:

### PRODUCTION CONFIG:
```yaml
api:
  base_url: "https://chiwawa.site/api"
  admin_token: "production_jwt_token"
database:
  host: "212.15.49.139"
  port: 5432
  database: "chiwawa"
  ssl: true
```

### МОНИТОРИНГ:
- Логирование всех API запросов
- Метрики производительности
- Alerting при ошибках подключения к API
- Health check endpoints

Создай полнофункциональный плагин с архитектурой, которая легко расширяется и поддерживается. Используй современные практики Java разработки и паттерны проектирования.
