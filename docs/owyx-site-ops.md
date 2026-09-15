# Owyx site — ops (сейчас)

## Сделать сейчас: DMARC (письма в спаме)

Cloudflare → **DNS** (не только Email → DMARC Management) для `owyx.site`.

Добавь TXT:

| Name | Content |
|------|---------|
| `_dmarc` | `v=DMARC1; p=none; rua=mailto:noreply@owyx.site; fo=1` |

Проверка: `dig +short TXT _dmarc.owyx.site` — должна вернуть запись.

SPF/DKIM уже через Mailjet. Через 1–2 недели при чистых отчётах: `p=quarantine`, потом `p=reject`.

Аватарка отправителя (зелёный «O») — не favicon. Брендовый логотип в инбоксе = **BIMI** (нужен строгий DMARC + SVG + часто платный VMC). Пока можно жить без него.

`EMAIL_FROM=noreply@owyx.site` на VPS.

Опционально: Cloudflare → Email → **Enable DMARC Management** — удобные отчёты, не замена DNS-записи.

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
