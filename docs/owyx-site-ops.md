# Owyx site — ops (сейчас)

## DMARC (сделано)

Запись `_dmarc.owyx.site` уже есть (через CF DMARC Management + `rua` на Cloudflare). Сейчас `p=none` — мониторинг. Дальше: `p=quarantine` → `p=reject` после чистых отчётов.

`EMAIL_FROM=noreply@owyx.site` на VPS. SPF/DKIM — Mailjet.

### Аватарка отправителя (BIMI) — не сейчас

Зелёный «O» в Mail.ru/Gmail ≠ favicon сайта. Бренд в инбоксе = **BIMI**:

1. DMARC минимум `p=quarantine` (лучше `p=reject`) — при `p=none` логотип не покажут.
2. Публичный SVG по HTTPS (профиль **SVG Tiny PS**, квадрат, без скриптов/внешних ссылок) — например `https://owyx.site/.well-known/bimi/owyx.svg` из упрощённого `brand/v2/owyx-icon-*.svg`.
3. DNS TXT `default._bimi.owyx.site` → `v=BIMI1; l=https://owyx.site/.well-known/bimi/owyx.svg;`
4. Для **Gmail / Apple Mail** почти всегда нужен платный **VMC** (Verified Mark Certificate, сотни–тысячи $/год, trademark на логотип). Без VMC BIMI иногда виден у Yahoo и части клиентов; **Mail.ru часто игнорит BIMI** и рисует свою букву.

Практично для ОБТ: DMARC уже ок → позже quarantine; BIMI/VMC отложить. Пока «Не спам» + контакт с `noreply@owyx.site`.

## Turnstile (уже на проде)

Ключи в `/opt/owyx/owyxsite/.env`:

- `TURNSTILE_SITE_KEY` / `TURNSTILE_SECRET_KEY`
- `NEXT_PUBLIC_TURNSTILE_SITE_KEY` (тот же site key, bake при build frontend)

После смены ключей: `cd /opt/owyx/owyxsite && bash deploy/vps-up.sh`.

## Админка

```bash
ssh owyxsite
docker exec -i owyx-postgres psql -U owyx_user -d owyx_db -c \
  "UPDATE users SET role = 'admin' WHERE LOWER(email) = LOWER('you@example.com');"
```

Перелогинься → **Админ-панель** в шапке.

## Автообновление лаунчера

Включено в `apps/app/tauri-owyx-release.conf.json`:

- endpoint: `https://github.com/ebluffy/Owyx/releases/latest/download/latest.json`
- артефакты `.sig` + `latest.json` кладёт `owyx-github-release`

Версии: SemVer от базы **0.2.0** (см. `.cursor/rules/semver.mdc`).

## Data dir лаунчера

Identifier / папка данных: **`Owyx`** → `%AppData%\Roaming\Owyx`.  
Установщик (NSIS) по-прежнему в `%LocalAppData%\Owyx` — так и задумано (как у upstream).

## Discord RPC

App ID: `1549541256370323527`. Art asset key: `owyx` (PNG 1024×1024 из `brand/v2/discord-rpc/`).

## Roadmap

- [ ] Скины на сайте/в ЛК — скрыты; выбор скина позже в лаунчере.
- [ ] Вкладка «Внешний вид» на сайте — кандидат на удаление.
- [ ] BIMI / брендовый avatar в почте (после DMARC quarantine/reject).
