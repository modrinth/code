# Microsoft authentication for Owyx Launcher (Path A)

Owyx supports three profile kinds: **Offline**, **Owyx** (email → `api.owyx.site`), and **Microsoft** (licensed Minecraft).

Tokens are stored as an opaque `ms1:` blob on the profile (machine-local
`profiles.json`). **Today this is base64 JSON, not OS-encrypted** — DPAPI /
keychain wrapping is a follow-up. Without the env var, the wizard stays
**`unconfigured`** and never invents fake user codes.

## What the owner must configure

1. Create an Azure AD app registration (Microsoft identity platform) as a **public client**.
2. Enable **Allow public client flows** and device-code / mobile+desktop auth.
3. Grant / enable Xbox Live sign-in for the app (same family of scopes as other Minecraft launchers: `XboxLive.signin` + `offline_access`).
4. Set at runtime (do not commit secrets; client ids for public native clients are often public):

```bash
export MICROSOFT_CLIENT_ID="<uuid-from-azure>"
```

5. Reply URLs: for device code, the verification URI is Microsoft’s (`https://www.microsoft.com/link` / login.microsoftonline.com). No loopback redirect is required for the device-code path.

## Launcher behavior

| Phase | Meaning |
|-------|---------|
| `unconfigured` | `MICROSOFT_CLIENT_ID` missing — wizard explains this doc |
| `waiting` | Real device code + verification URI from Azure |
| `success` | Minecraft gamertag + `authBlob` (`ms1:…`) — profile `needsAuth=false` |
| `error` | Azure / Xbox / Minecraft failure (clear message; no fake codes) |

Offline Play and Owyx email login are independent and must keep working when MS is unconfigured or failing.

## Honest follow-ups

- Windows DPAPI / OS keychain wrapping of `ms1:` blobs (today: opaque base64 JSON in profiles.json).
- Using the Minecraft access token in the Play / launch pipeline (Yggdrasil/MSA user type) — do not break offline / Owyx.
- SISU / Proof-of-Possession Xbox device-token flow (Modrinth Theseus) is not required for Path A device-code.

## Reference

- Modrinth Theseus auth: `packages/app-lib` (SISU + device patterns)
- Owyx profile storage: `src-tauri/src/profiles.rs`
- Site contract (Owyx account, not MS): `owyxsite/LAUNCHER_SITE_CONTRACT.md`
