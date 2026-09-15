# Owyx — единый план

Один файл на весь монорепо. Старые `Launcher/PLAN.md`, `owyxsite/PLAN.md`, `owyxplugin/PLAN.md` — только указатели сюда.

**Репо:** https://github.com/ebluffy/Owyx  
**Обновлено:** 2026-08-30 (launcher mass review + Play/Modrinth E2E; лаунчер **v0.5 rc**)  
**Сейчас цель:** Windows GitHub Releases (debug + release exe) у владельца. Cloud Play + 2 мода закрыты.

MiniMax Windows-QA (`MINIMAX_WIN_QA.md`) — **после** этого контура, не сейчас.

---

## 1. Продукт

**Owyx** — экосистема своих Minecraft-серверов: сайт (аккаунт, роли, каталог серверов/паков) ↔ лаунчер (скачал, ник или аккаунт, Play) ↔ плагин (сессия на сервере, позже).

Играть открыто, без заявок:

| Путь | Что нужно |
|------|-----------|
| Гость | Скачал `Owyx.exe` → ник → Play |
| Аккаунт Owyx | Регистрация на сайте → тот же логин в лаунчере → скин и плюшки |
| Microsoft | Честный stub, не ломает offline/Owyx |

**Игрок:** один `Owyx.exe`, данные `%USERPROFILE%/owyx`.  
**Админ (роль `admin` на сайте):** добавляет сервера и сборки **в лаунчере** (под своим профилем) и **на сайте** — кому как удобнее. Один каталог, два UI.

Лаунчер без сайта умеет offline + ваниль/локальные инстансы. Список **своих серверов и паков** живёт на сайте — это нормально.

Хостинг по этапам: сейчас друзья через мини-ПК + FRPC → потом отдельный РФ VPS под сайт. Игровой порт за FRPC часто показывает клиентам `127.0.0.1` — это про Minecraft, не про «сайт нельзя хостить».

Дизайн: [`DESIGN.md`](./DESIGN.md) — фон `#050508`, accent `#00e5ff`. Контракт API: [`owyxsite/LAUNCHER_SITE_CONTRACT.md`](./owyxsite/LAUNCHER_SITE_CONTRACT.md). Сборка лаунчера: [`Launcher/BUILD.md`](./Launcher/BUILD.md).

---

## 2. Где что лежит

| Путь | Роль |
|------|------|
| `Launcher/` | Tauri 2 + Rust + Vite. Референс поведения: `../modrinth-code` / Theseus — **портировать**, не подменять продукт |
| `owyxsite/` | Next `:3000` + Express `:3001` + Postgres 17 + Adminer. Сайт = control-plane |
| `owyxplugin/` | Paper/Purpur. Ещё бренд Chiwawa в коде. Не блокер друзей-паков |

Секреты только в `.env`. В git нет `*.exe`, `target/`, `node_modules/`, дампов БД.

---

## 3. Статус на 2026-08-24 (после overnight friends sprint)

### Сайт — UI **v0.1**, launcher API **v1.1.0**

Сделано раньше: cyan-редизайн, открытый доступ, Owyx-login, скины на сайте, ник 30д, смена почты, новости+админ CRUD, HTML-письма.

Сделано в этом PR:

- Таблицы `packs` + `servers` (`007_servers_packs.sql`), compose mount, idempotent `ensureCatalogSchema()` на старте backend
- Публично: `GET /api/launcher/v1/servers|packs|packs/:id|packs/:id/manifest|news` — **не stubs**, seed + живые строки
- Admin CRUD `/api/admin/packs` + `/api/admin/servers` + zip ingest (≤50 MB → `/uploads/packs/`)
- UI `/admin`: вкладки **Серверы** / **Сборки**, модалки, hide/show, загрузка zip
- Игрокам **не** отдаём SFTP-логины; admin GET видит `hasPassword`, не пароль
- News в лаунчер API = та же таблица `news`, что сайт

### Лаунчер — **v0.5 progress** (не ship)

Сделано в этом PR:

- Вкладка **Серверы**: каталог с сайта, empty/offline states, Скачать / Play
- `http_zip` / `http_manifest`: качка + sha256 + zip-slip-safe extract; SFTP-клиент **не** для игроков
- Play: `servers.dat` + `--quickPlayMultiplayer` (MC ≥ 1.20) иначе `--server/--port`
- Админ в лаунчере (`/me.role === admin`): модалка «Добавить сервер» → тот же admin API
- Owyx login: 401/403 → `no_session`, UI не падает; offline-ник не трогали
- Скин с `/me`: файл `~/owyx/skins/{nick}.png` (+ опционально `instances/{id}/game/owyx/skin.png`). In-game apply (CustomSkinLoader) — **не** в этом PR
- Modrinth: фильтр loader/MC инстанса, выбор версии, install + required deps (глубина ≤ 5)

Cloud Linux: **native** `npm run dev:tauri` (WebKitGTK) + `build:smoke` ELF — закрыто в Linux-native PR. Vite `:1420` остаётся быстрым UI-preview без invoke. Windows Play / `Owyx.exe` — hand-off после мержа. MiniMax — отдельно.

### Плагин — не спринт друзей

Ребрендинг `chiwawa` → `owyx` — после живых серверов. Game-token deprecated.

---

## 4. Этот спринт — что закрыто / что осталось

Закрыто в overnight PR (ветка `cursor/friends-control-plane-servers-7784`) и quality-pass (#12). Linux **dev + smoke ELF** закрыт отдельным PR; MiniMax / магазин / Discord RPC / Linux `.deb` — не сюда.

### 4.1 Сделано

- Control-plane: миграция + API + admin UI + seed `demo-vanilla` (tiny fixture zip)
- Лаунчер Servers живой; качка http_zip/manifest; Play пишет адрес
- Админ создаёт сервер с сайта **и** из лаунчера
- Modrinth: version picker + required deps (не полный Theseus)
- Скин: файл на диске + UI, не «только URL в JSON»

### 4.2 Осталось (честный backlog, не Theseus-complete)

- Windows hand-off: полный install ванили + overlay + Play на IP:port у друга
- Онлайн-статус серверов (`status.online` пока `null`); ping/motd нет
- Полный mrpack parser / Theseus resolver / SQLite content DB / upgrade UI
  (сейчас schema + `downloadAvailable: false`, overlay честно падает на `.mrpack`)
- Optional Modrinth deps — opt-in в UI; required deps fail-fast + cycle guard
- SFTP warehouse → HTTP publish (сейчас креды только админу + ручной HTTP URL)
- Resume больших качек; дельта-синк «3 мода из 200»
- In-game skin apply (CustomSkinLoader / Theseus player-data); файл на диске есть
- Microsoft OAuth, shop, plugin rebrand (`chiwawa` → `owyx`)
- Native Linux **ship** (`.deb` / AppImage) — **не** этот PR; dev + debug ELF уже есть
- GitHub Release v0.1 / v0.5 exe
- MiniMax Windows QA

### 4.3 Что не сейчас

- MiniMax полный прогон
- Платежи, shop, форум
- Ребрендинг плагина
- Microsoft OAuth
- Возврат заявок

### 4.4 Решения (спорное → простой путь для друзей)

| Тема | Решение |
|------|---------|
| Игрок vs склад | Игрокам только HTTP zip/manifest + sha. SFTP только admin, никогда в `GET /servers` |
| Ids | TEXT slug (`owyx-demo`), не serial |
| Байты | Не в Postgres. Fixture/dev: `/fixtures/packs/*.zip` на backend; прод: URL склада |
| Старый Docker volume | `initdb.d` не накатывает. README: `psql -f 007_servers_packs.sql`. Backend ещё `ensureCatalogSchema()` — таблица должна быть **OWNER** `owyx_user` |
| Play адрес | `servers.dat` (NBT gzip) + quickPlay ≥ 1.20, иначе `--server/--port` |
| HTTP allowlist | HTTPS публичный склад + loopback/private + хост API (dev fixture) |
| google_drive / mrpack | Схема + admin UI; качка игроку только если есть прямой download URL |
| Cloud | Native `dev:tauri` + Linux smoke ELF. Vite — secondary. Не Wine/WebView2 |

---

## 5. Откуда берётся сборка (решение)

В карточке пака админ **выбирает источник**. Лаунчер/сайт не зашивают один SFTP в код.

Две роли протокола:

| Кто | Канал | Зачем |
|-----|--------|--------|
| **Игрок** | почти всегда **HTTP(S) + манифест + sha256** | У лаунчера уже есть качалка, allowlist, прогресс. Не тащить SFTP-пароль всем друзьям |
| **Админ / склад** | HTTP, SFTP, диск админа, Drive, mrpack | Как **положить** файлы на мини-ПК |

Мини-ПК быстрее Google Drive на 2 ГБ у друзей — это склад. На складе крутится **HTTP-раздача** (Caddy/nginx за тем же FRPC). SFTP — чтобы **ты** заливал, не чтобы каждый клиент логинился в SFTP.

### Источники в модалке (v0.5 friends)

1. **HTTP zip / папка на складе** — URL уже готового архива или индекса на мини-ПК. Основной путь для игроков.  
2. **SFTP** — `host:port` (VPN/FRPC), пользователь, путь к **файлу или папке**. Хранится на сайте как секрет админа. Игрокам **не** отдавать логин в `GET /servers`. Сайт/админ-лаунчер забирает со склада → публикует HTTP+манифест.  
3. **Google Drive** — запас, если ссылка уже есть. Квоты и промежуточная страница — не основной канал.  
4. **mrpack** — файл или URL. Распаковка формата Modrinth (как Theseus), не свой формат.  
5. **Локальный профиль админа** — папка инстанса (CurseForge/свой модпак + ручные моды). Админский лаунчер пакует zip, считает хеши, **заливает на склад** (SFTP или HTTP PUT), пишет манифест. Друзья качают как (1).

### «Скормить zip — лаунчер сам разберётся»

Отдельный сервис **Pack ingest** (сначала в админ-лаунчере, потом можно на сайте):

1. Принял zip / папку / mrpack  
2. Распаковал во временную  
3. Узнал MC-версию и лоадер (minecraftinstance.json / mrpack index / mods)  
4. Собрал манифест файлов + sha256  
5. Залил на склад в `packs/{packId}/`  
6. Привязал к карточке сервера  

Первая версия может быть тупой: один zip целиком. Умный ingest — сразу после того, как друзья один раз успешно качнули.

**Не делать:** сырой SFTP с паролем в публичном API; 2 ГБ в теле запроса сайта; единственная надежда на Drive.

---

## 6. Версии продукта

| Версия | Смысл |
|--------|--------|
| Сайт UI **0.1** | Кабинет, auth, скин на сайте, новости — есть |
| API лаунчера **v1.1.0** | login + `/me` + **живой** каталог servers/packs/news — есть в этом PR |
| Лаунчер **v0.1** | Classic Play — есть |
| Лаунчер **v0.5** (цель спринта) | **progress:** каталог + качка HTTP + admin create + Modrinth versions + скин на диск. Windows Play у друзей — после мержа |
| Лаунчер **v1.0** | Стабильный Windows exe для внешнего круга |
| Плагин **позже** | Ребренд + session-login, не game-token |

После **v0.5** у друзей: хост на мини-ПК. Потом сайт на РФ VPS. Потом MiniMax QA по чеклисту.

---

## 7. Лаунчер — стек и диск

| Слой | Технология |
|------|------------|
| Backend | Rust |
| Окно | Tauri 2 + WebView2 (Windows) / WebKitGTK 4.1 (Linux) |
| UI | HTML/CSS/TS + Vite |

```text
%USERPROFILE%/owyx/
  config.json · profiles.json
  instances/{id}/instance.json · game/
  meta/versions|libraries|assets|natives|java_versions
  cache/ · logs/
```

Сборка: `npm run check` → `check:rust` → Linux `build:smoke` (ELF) → Windows `build:release` только для ship. См. `Launcher/BUILD.md`.

Theseus-пути install/launch — опора. UI свой (Owyx), не копия Modrinth App целиком. **Modrinth content** — наоборот, копировать паттерны Theseus.

Известный техдолг: AUTH stub MS; полный Theseus content (optional deps, upgrade UI); Wine WebView2; GitHub Release; CI Windows artifact; in-game skin; server ping; mrpack parse; SFTP→HTTP publish.

---

## 8. Сайт — стек и отложенное

Postgres 17 + Express + Next. Роли: `user | helper | moderator | admin`. Бан блокирует `/me`. Applications deprecated.

Когда control-plane живой, можно вернуться к: SMTP/Turnstile прод, SSL, Directus, shop, мониторинг.

Compose: монтировать **все** миграции в `initdb.d`. На старый volume — ручной `psql -f`.

---

## 9. Плагин (после друзей)

1. Package/команды/JAR `owyx`, не Chiwawa  
2. API base `api.owyx.site` / локальный  
3. Логин: цель — сессия лаунчера; game-token только переходный  
4. Не менять JWT в одиночку  

---

## 10. Архитектура (цель)

```text
Админ (лаунчер или сайт)
    │ JWT role=admin
    ▼
Сайт API  —  servers + packs + users + skins
    │
    ├─ игроки: GET catalogs, login, /me
    ▼
Owyx.exe  —  список серверов, качка пака, Play, Modrinth, скин
    │
    ▼
Склад файлов (мини-ПК): HTTP для игроков, SFTP/диск для админа
    │
    ▼
Minecraft + плагин (позже автологин)
```

---

## 11. Чеклист «можно звать друзей»

- [x] Админ с сайта/лаунчера создаёт сервер + пак (API + оба UI)
- [x] Друг в лаунчере видит карточку и качает http_zip/manifest (native `dev:tauri` + compose)
- [x] Play на Cloud Linux: Fabric 1.20.1 + Fabric API + Sodium → MC main menu (`dev:tauri`)
- [ ] Play заходит на IP:port на **Windows** у друга (код есть; owner собирает exe)
- [x] Источник пака задан в карточке (HTTP zip; seed `demo-vanilla`)
- [x] Обрыв качки не оставляет вечный `installing` без выхода (ошибка → fail, не silent)
- [x] Modrinth: найти мод → выбрать версию → поставить (+ required deps)
- [x] Скин аккаунта доезжает до папки профиля (`~/owyx/skins`); in-game apply — долг
- [x] Offline-ник по-прежнему работает без сайта
- [x] VPS: `deploy/vps-up.sh` + nginx `owyx.site` (не ломать `panel.owyx.site`)
- [ ] Потом: MiniMax Windows QA, только отчёт

---

## 12. Процесс

- Сначала этот `PLAN.md`, потом код.
- Фичи сайта/лаунчера — PR, не сразу в `main`, пока сами не скажем мержить.
- Авторевью на PR. Секреты не коммитить.
- Облачный агент не мержит сам.
- Референс экрана лаунчера: сначала Theseus/`modrinth-code`, потом порт в Owyx.

---

## 13. Overnight friends sprint — находки

- Старый Postgres volume **не** применяет новые SQL сам. `ensureCatalogSchema()` падает `42501 must be owner of table packs`, если таблицы создал `postgres`, а backend ходит как `owyx_user` — `ALTER TABLE packs, servers OWNER TO owyx_user`.
- Cloud agent VM: Docker не обязателен. Postgres 16 + apt хватает; compose ждёт 17.
- Linux Tauri: `Launcher/scripts/linux-dev-setup.sh` (Rust ≥ 1.85 + WebKitGTK 4.1 / GTK). Primary cloud path = `npm run dev:tauri`, не Vite-only и не Wine.
- Сайтовый `npm run lint` (frontend) чинится в quality-pass PR (`AuthProvider`,
  profile/admin без sync setState-in-effect).
- Полный vanilla install из cloud не гоняли (долго + нет native Play). Overlay-путь в `catalog.rs` есть.
- Идея на потом: админский «publish to configured HTTP PUT» с мини-ПК, без SFTP в лаунчере игрока.

---

## 14. Deep quality pass (2026-08-24)

Один проход по `Launcher/` + `owyxsite/` с опорой на [modrinth/code](https://github.com/modrinth/code)
(Theseus-паттерны: fail-fast deps, zip-slip, redirect policy). `owyxplugin/` не трогали.

### Лаунчер

- Overlay fail больше не оставляет инстанс `ready`: сносится `meta.json`, `status=error`
- Required Modrinth deps: fail-fast, цикл `visited`, optional только по флагу
- Catalog HTTP: своя redirect policy (нет HTTPS→HTTP, нет public→loopback)
- Zip-slip / symlink overlay, caps, `META-INF` только как первый компонент
- Skin download: тот же HTTP-клиент + post-redirect `assert_asset_url`
- Quick Play только на MC ≥ 1.20.2
- Честный mrpack: overlay ошибка, не «это zip»
- UI: Play блокирует `needsAuth`; i18n relative time; dialog/`showModal`; combobox;
  optional deps checkbox; mrpack/sftp Download disabled

### Сайт

- Сессии: SHA-256 hash (legacy base64 мигрирует при использовании)
- `GET /verify` проверяет строку сессии; banned/inactive → 403
- Plugin write routes: long-term API token **only**, website JWT не проходит
- Turnstile в production fail-closed без секрета; `trust proxy` из `TRUST_PROXY_HOPS`
- Публичный player list без `ip_address`; admin settings redact password/secret
- Chat: membership на private rooms; socket join до send
- `mrpack` public `downloadAvailable: false`; ingest `.mrpack` + удаление файла
- Frontend: один `AuthProvider`, logout через API, пароль ≥ 8, `/servers` IP из settings

Cloud Linux: `dev:tauri` + compose API. Vite `:1420` без invoke. WebView2 / Wine — не cloud path.

---

## 15. Launcher ↔ site E2E + VPS (2026-08-29)

- Native catalog/login/Modrinth browse против `OWYX_API_BASE_URL=http://127.0.0.1:3001`
- Библиотека: вкладки — фильтры (не disabled stubs). Modrinth: иконки + «ещё» (offset)
- Admin create server: честный лимит **512 MB** (`MAX_PACK_BYTES`) — Foreg 2GB не этот PR
- `owyxsite/deploy/vps-up.sh`: port audit, `.env` если нет, health
- `docker-compose.prod.yml`: loopback + Adminer `profiles: [debug]`
- nginx: `owyx.site` → Next/API, не Coming Soon; `panel.owyx.site` не трогаем

---

## 16. Launcher mass review / pre-release QA (2026-08-30)

Cloud `dev:tauri` + site API `:3001`. Цель — release-ready лаунчер до двух Windows GitHub Release.

### Починено в этом проходе

- Modrinth required deps резолвятся по **loader/MC инстанса**, не `.first()` у родителя
- Upgrade мода: atomic replace, без delete-before-write
- `servers.dat` на Play больше не глотается; Forge natives merge — ошибка, не `let _`
- macOS: `-XstartOnFirstThread` если нет в jvm args
- UI: XSS в фильтрах browse; ошибки через `textContent`; lock на Modrinth install
- Профильный wizard блокирует rail; Settings/Stats **прибиты к низу rail**; create — Enter + sticky footer
- i18n win chrome; active tabs/rail — cyan, не violet fill; `btn-warn` без amber primary

### E2E (cloud)

- Instance `Fabric 1.20.1` · Fabric `0.19.5` · путь `~/owyx/instances/Fabric 1.20.1/`
- Моды: `fabric-api-0.92.11+1.20.1.jar`, `sodium-fabric-0.5.13+mc1.20.1.jar`
- Play → процесс Java `KnotClient` + main menu «Minecraft 1.20.1/Fabric (Modded)»
- Создан `qa-fabric-mods` (pending install — тот же MC/loader; полный install уже на `Fabric 1.20.1`)

### Windows hand-off

Owner: `npm run build:smoke` → debug exe; `npm run build:release` → `Launcher/Owyx.exe`. Теги v0.5.0-rc1 / v0.5.0.
