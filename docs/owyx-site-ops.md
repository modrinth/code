# Owyx site — email, admin, Discord RPC, updater, roadmap

## Cloudflare Turnstile («я не робот»)

Сейчас на проде капча **выключена**, пока в `.env` стоят заглушки `obt-pend…`.
Регистрация работает без виджета.

Чтобы включить:

1. [Cloudflare Dashboard](https://dash.cloudflare.com/) → Turnstile → Add site → `owyx.site`
2. Скопируй **Site Key** и **Secret Key** в `/opt/owyx/owyxsite/.env`:

```env
TURNSTILE_SITE_KEY=0x...
TURNSTILE_SECRET_KEY=0x...
NEXT_PUBLIC_TURNSTILE_SITE_KEY=0x...   # тот же Site Key
TURNSTILE_SKIP=false
```

3. `bash owyxsite/deploy/vps-up.sh` (frontend пересоберёт `NEXT_PUBLIC_*`).

Виджет «Я не робот» появится на `/register` и `/login`.

## Почта (Mailjet) — чеклист

Правильно, если:

- Domain `owyx.site` = **Active**
- Sender `noreply@owyx.site` = **Active**
- SPF/DKIM для `owyx.site` = **OK**
- **Page Domains** — не нужно (это Premium для лендингов Mailjet, не для SMTP)

`EMAIL_FROM` на сервере должен быть `noreply@owyx.site`.


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
