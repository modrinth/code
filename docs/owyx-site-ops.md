# Owyx site — email, admin, Discord RPC

## Регистрация и подтверждение email

После `POST /api/auth/register` бэкенд **создаёт** пользователя и **пытается** отправить письмо с подтверждением (`email_verification_tokens`, ссылка на `/verify?token=…`).

Если SMTP не настроен, регистрация всё равно проходит, но письмо не уйдёт — смотри логи `owyx-backend`.

Переменные в `owyxsite/.env` (на VPS: `/opt/owyx/owyxsite/.env`):

| Переменная | Назначение |
|------------|------------|
| `SMTP_HOST` | Хост SMTP |
| `SMTP_PORT` | Порт (`465` + `SMTP_SECURE=true` или `587` + STARTTLS) |
| `SMTP_USER` / `SMTP_PASS` | Логин и пароль |
| `EMAIL_FROM` | От кого (если задано в коде/шаблонах) |

### Бесплатные / простые SMTP-варианты

1. **Brevo (Sendinblue)** — бесплатный tier (~300 писем/день), SMTP `smtp-relay.brevo.com`, порт 587.
2. **Resend** — бесплатный tier для dev; удобный API, есть SMTP на платных планах.
3. **Mailgun** — trial, затем pay-as-you-go; SMTP `smtp.mailgun.org`.
4. **Gmail** — только для теста: App Password + `smtp.gmail.com:465` (лимиты, не для продакшена).
5. **Yandex 360 / обычный Ящик** — `smtp.yandex.ru:465` (дефолт в `emailService.js`).

После смены `.env` на VPS: `docker compose -f owyxsite/docker-compose.yml up -d --force-recreate backend`.

## Выдать себе админку

1. Зарегистрируйся на https://owyx.site/register  
2. На VPS:

```bash
ssh owyxsite
docker exec -i owyx-postgres psql -U owyx_user -d owyx_db -c \
  "UPDATE users SET role = 'admin' WHERE LOWER(email) = LOWER('you@example.com');"
```

3. Перелогинься на сайте (или очисти JWT в браузере). В шапке появится **Админ-панель** (`/admin`).

Роли: `admin`, `moderator` (staff в UI).

## Discord RPC (лаунчер показывает «Modrinth»)

Discord берёт **название приложения** из [Discord Developer Portal](https://discord.com/developers/applications), не из текста в коде.

1. Создай приложение **Owyx**, загрузи иконку из `brand/v2/`.
2. Скопируй **Application ID**.
3. Собери лаунчер с `OWYX_DISCORD_APP_ID=<id>` (GitHub Actions: добавь secret и env в workflow build step).
4. В Rich Presence → Art Assets добавь ключ **`owyx`** (как в `packages/app-lib/src/state/discord.rs`).

Пока ID не свой, в Discord будет отображаться старое приложение Modrinth.

## Автообновление лаунчера

Релизы публикуются через `.github/workflows/owyx-github-release.yml`.  
Сборка использует `tauri-owyx-release.conf.json` с **`createUpdaterArtifacts: false`** — автообновление с GitHub **ещё не включено** (нужны signing pubkey + endpoint на `latest.json` релиза). Обновление вручную: скачать новый setup с Releases.
