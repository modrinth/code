# OBT prep — separate VPS (site + API only)

**Status:** preparation only (domains, client key, secrets, bootstrap). Full public cutover when DNS + Turnstile + SMTP are live.

## Domains

| Host | Role |
|------|------|
| `https://owyx.site` | Website (Next). Browser `/api` same-origin → backend. **No** launcher client key. |
| `https://api.owyx.site` | Launcher API + `/uploads`. Requires header `X-Owyx-Client-Key`. |
| `panel.owyx.site` | VPN / C³ — **other VPS**, do not touch. |

Game join (FRPC): `45.131.186.146:1488` (RU VPS → MiniPC). Not hosted on the Owyx site VPS.

## Client key

- Site env: `LAUNCHER_CLIENT_KEY`
- Launcher: `OWYX_CLIENT_KEY` or `%USERPROFILE%/owyx/client_key`
- Header: `X-Owyx-Client-Key`
- `/health` on api host stays open for probes

This is **not** end-user login. It stops random scrapers from using the launcher API as a free public API. JWT still required for `/me` and admin.

## Local secrets (gitignored)

Already generated on the owner machine (do not commit):

- `owyxsite/.env`
- `Launcher/.env`
- `%USERPROFILE%/owyx/client_key`

For VPS: copy `owyxsite/.env`, set `NODE_ENV=production`, real `TURNSTILE_SECRET_KEY`, SMTP, strong `ADMIN_PASSWORD`.

## Clean VPS checklist

```bash
ssh owyxsite
# then:
curl -fsSL https://raw.githubusercontent.com/ebluffy/Owyx/main/owyxsite/deploy/bootstrap-vps.sh | sudo bash
# or from a cloned repo:
sudo bash /opt/owyx/owyxsite/deploy/bootstrap-vps.sh
```

1. DNS A records → new VPS IP  
2. `bootstrap-vps.sh` (docker, nginx, swap, ufw)  
3. Clone repo → place `.env`  
4. `bash deploy/vps-up.sh` (no `--dev`)  
5. Enable `deploy/nginx-owyx.site.conf`  
6. `certbot --nginx -d owyx.site -d www.owyx.site -d api.owyx.site`  
7. Smoke: `curl https://owyx.site/` · `curl https://api.owyx.site/health` · launcher with client key  

## Launcher OBT build

```powershell
# Launcher/.env already has api.owyx.site + client key
cd Launcher/apps/launcher
npm run build:smoke
Copy-Item -Force src-tauri\target\debug\owyx.exe ..\..\Owyx.debug.exe
```

Ship a GitHub Release later (separate task).

## Out of this prep

- Full production Turnstile + email go-live  
- Microsoft OAuth  
- Full mrpack / multi-GB resume  
- Merging onto the VPN VPS  
