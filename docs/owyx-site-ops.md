# Owyx site — email, admin, Discord RPC, updater, roadmap

## Регистрация и подтверждение email

После `POST /api/auth/register` бэкенд создаёт пользователя и пытается отправить письмо
(`email_verification_tokens` → `/verify?token=…`).

Пароль: минимум 8 символов, **1 заглавная**, **1 цифра**, **1 спецсимвол**.

### Mailjet (прод)

На VPS в `/opt/owyx/owyxsite/.env`:

```env
SMTP_HOST=in-v3.mailjet.com
SMTP_PORT=587
SMTP_SECURE=false
SMTP_USER=<Mailjet API key>
SMTP_PASS=<Mailjet Secret key>
EMAIL_FROM=noreply@owyx.site
```

`EMAIL_FROM` должен быть **verified sender** в Mailjet (или адрес на домене `owyx.site`).

#### Дальше по Mailjet / DNS

1. В Mailjet: **Domains and senders** → домен `owyx.site` = Active (уже есть).
2. Вкладка **SPF/DKIM Authentication** — скопируй TXT-записи в Cloudflare DNS для `owyx.site`.
3. Добавь sender `noreply@owyx.site` (или `hello@owyx.site`) и дождись Active.
4. Пока SPF/DKIM не зелёный — можно временно слать с verified Gmail sender (`shadowgamesblacktube@gmail.com`), но лучше доменный From.
5. После смены `.env`:

```bash
ssh owyxsite
cd /opt/owyx/owyxsite && docker compose up -d --force-recreate backend
```

Проверка: зарегистрируй тестовый аккаунт → письмо в inbox/spam.

## Выдать себе админку

```bash
ssh owyxsite
docker exec -i owyx-postgres psql -U owyx_user -d owyx_db -c \
  "UPDATE users SET role = 'admin' WHERE LOWER(email) = LOWER('you@example.com');"
```

Перелогинься. В шапке сайта — **Админ-панель**. В лаунчере вкладка **Owyx Servers** (API settings) видна только staff (`admin` / `moderator`).

## Discord RPC

Application ID (зашит в сборку): `1549541256370323527`.

1. Discord Developer Portal → Rich Presence → Art Assets.
2. Загрузи PNG **1024×1024** (не SVG). Ключ ассета: **`owyx`**.
3. Файл для загрузки: `brand/v2/discord-rpc/owyx.png` (сгенерирован из бренд-кристалла).
4. Опционально второй ассет `owyx_playing` для статуса «в игре».

После загрузки ассета перезапусти Discord и лаунчер (кэш ассетов бывает долгим).

## Автообновление лаунчера

Включено в `apps/app/tauri-owyx-release.conf.json`:

- `createUpdaterArtifacts: true`
- endpoint: `https://github.com/ebluffy/Owyx/releases/latest/download/latest.json`
- pubkey + secret `TAURI_SIGNING_PRIVATE_KEY` в GitHub Actions

Релизный workflow кладёт в Release: установщики, `.sig`, `latest.json`.

## Аккаунты: MS приоритетнее offline/Owyx-ника

- Вход на сайте → лаунчер создаёт offline-профиль с логином сайта.
- Если есть **Microsoft (лицензия)** — он всегда active по умолчанию (ник/скин Mojang).
- Offline-ник остаётся в списке аккаунтов для offline-mode серверов.

## Roadmap (не в этом релизе)

- [ ] Скины без лицензии (идея TLSkins / локальный skin apply для offline-профиля). **Сейчас скины только через Microsoft / Mojang API.**
- [ ] Кастомный Discord Application Icon в портале (иконка приложения ≠ Rich Presence asset).
