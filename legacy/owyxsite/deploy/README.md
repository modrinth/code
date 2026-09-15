# Owyx site — Docker + nginx on a dedicated VPS

Owns **`owyx.site`** (website) and **`api.owyx.site`** (launcher API).  
Never writes **`panel.owyx.site`** (C³ CELERITY / VPN — other machine).

OBT prep: [`OBT_PREP.md`](./OBT_PREP.md). Fresh box: [`bootstrap-vps.sh`](./bootstrap-vps.sh).

## Local (Cloud VM / laptop)

```bash
cd owyxsite
bash deploy/vps-up.sh --dev
curl -sf http://127.0.0.1:3001/health
```

### Launcher against this compose

```bash
export OWYX_API_BASE_URL=http://127.0.0.1:3001
# client key optional on loopback (Host is not api.owyx.site)
cd ../Launcher/apps/launcher
npm run dev:tauri
```

Pack zip overlay is capped at **512 MB** in the launcher (`MAX_PACK_BYTES`).

## Dedicated Owyx VPS (recommended for OBT)

1. DNS A: `owyx.site`, `www.owyx.site`, `api.owyx.site` → this VPS. Leave `panel.owyx.site` alone.
2. `sudo bash deploy/bootstrap-vps.sh`
3. Clone under `/opt/owyx`, place `.env` (`LAUNCHER_CLIENT_KEY`, secrets, real `TURNSTILE_SECRET_KEY`)
4. `bash deploy/vps-up.sh` (no `--dev`)
5. Enable `deploy/nginx-owyx.site.conf`, then certbot for all three names
6. Firewall: 80/443 public; do **not** expose 3000/3001/5432

Launcher prod: `https://api.owyx.site` + header `X-Owyx-Client-Key`.

## Port playbook

| Service  | Default | Prod bind |
|----------|---------|-----------|
| frontend | 3000    | `127.0.0.1` |
| backend  | 3001    | `127.0.0.1` |
| postgres | 5432    | `127.0.0.1` |

## Fresh vs old Postgres volume

`initdb.d` runs only on empty volumes. Apply `007_servers_packs.sql` on old DBs; owner `owyx_user`.
