# Owyx site — ops

## Deploy

```bash
ssh owyxsite
cd /opt/owyx/owyxsite && bash deploy/vps-up.sh
```

Loopback binds only (`127.0.0.1:3000/3001/5432`). Host nginx owns TLS for `owyx.site` / `api.owyx.site`. Do **not** touch `panel.owyx.site`.

## Env (VPS `/opt/owyx/owyxsite/.env`)

| Key | Notes |
|-----|--------|
| `LAUNCHER_CLIENT_KEY` | Required for `api.owyx.site` (header `X-Owyx-Client-Key`) |
| `CORS_ORIGIN` | Include site + Tauri origins (`https://tauri.localhost`, …) |
| `TURNSTILE_*` / `NEXT_PUBLIC_TURNSTILE_SITE_KEY` | Browser captcha; launcher skips via client key |
| `EMAIL_FROM` | `noreply@owyx.site` (Mailjet + SPF/DKIM) |
| `NEXT_PUBLIC_LAUNCHER_DOWNLOAD_URL_WINDOWS` | Optional override; default latest `Owyx_*_x64-setup.exe` |
| `NEXT_PUBLIC_LAUNCHER_DOWNLOAD_URL_LINUX` | Optional override; default latest AppImage |

Never commit real keys.

## Admin role

```bash
docker exec -i owyx-postgres psql -U owyx_user -d owyx_db -c \
  "UPDATE users SET role = 'admin' WHERE LOWER(email) = LOWER('you@example.com');"
```

Re-login → **Админ-панель** in the header.

## Catalog seed

If admin deleted demo rows, re-insert via admin UI or:

```bash
# packs + servers (demo) — see deploy/tmp-reseed.sh pattern / ensureCatalogSchema
```

Play address for friends OBT: `45.131.186.146:1488`.

## Launcher

- Releases: `https://github.com/ebluffy/Owyx/releases` (Windows setup + Linux AppImage)
- Updater: `latest.json` via `owyx-github-release`
- SemVer from **0.2.0** — see `.cursor/rules/semver.mdc`
- Data dir: `%AppData%\Roaming\Owyx` (`identifier` = `Owyx`)
- Contract: `owyxsite/LAUNCHER_SITE_CONTRACT.md`

## Mail / DMARC

`_dmarc.owyx.site` is live (`p=none` → later quarantine/reject). BIMI/VMC not in scope for OBT.

## Discord RPC

App ID `1549541256370323527`, art key `owyx`.
