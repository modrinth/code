# Microsoft OAuth for Owyx Launcher

EN + RU. Documents the existing Theseus / `packages/app-lib` Microsoft login path used by this fork. **Do not invent a second auth stack.**

---

## English

### What works today

Owyx reuses Modrinth Theseus Minecraft auth:

- Shared public Minecraft launcher client id: `MICROSOFT_CLIENT_ID = 00000000402b5328`
- Flow: Microsoft OAuth (login.live.com) → Xbox SISU / XSTS → Minecraft services entitlements → profile
- Desktop reply: WebView navigates to `https://login.live.com/oauth20_desktop.srf?code=…`; the Tauri window in `apps/app/src/api/auth.rs` reads the `code` query and finishes login
- Deep-link / app scheme for the product is **`owyx://`** (`apps/app/tauri.conf.json` plugins.deep-link). Legacy `modrinth://` is still accepted in `packages/app-lib/src/api/handler.rs` for upstream pack installs

Offline nickname accounts are separate (`login_offline`) and never touch Microsoft tokens.

### When you need your own Azure / Entra app

The public Minecraft client id works for the classic desktop redirect (`oauth20_desktop.srf`) used by many launchers. Register your **own** Azure app when you need:

- Custom branding on the Microsoft consent screen
- Additional redirect URIs (custom `owyx://…` reply, loopback `http://127.0.0.1:…/callback`)
- Device-code or confidential-client flows
- Stricter tenant / org policy

### Register an Azure public client (optional)

1. Azure Portal → Microsoft Entra ID → App registrations → **New registration**
2. Name: `Owyx Launcher` (or similar)
3. Supported account types: **Personal Microsoft accounts** (and work/school if you need them)
4. Redirect URIs (Public client / native):
   - `https://login.live.com/oauth20_desktop.srf` (current WebView path)
   - Optional: `owyx://auth/callback` if you switch to scheme-based reply
   - Optional: `http://127.0.0.1:<port>/callback` for a loopback listener (`apps/app/src/api/oauth_utils`)
5. Authentication → **Allow public client flows** = Yes
6. API permissions / scopes used by the launcher chain include Xbox Live sign-in + offline access (see code; do not invent scopes)

### Where to put the client id

Prefer environment / build-time config — **never commit secrets**.

Today the id is the public Minecraft constant in:

- `packages/app-lib/src/state/minecraft_auth.rs` → `MICROSOFT_CLIENT_ID`

If you fork with a private Azure app, wire it via env (e.g. `OWYX_MICROSOFT_CLIENT_ID`) and read it in that module — keep the public id as default for open builds.

### High-level token chain (code map)

| Step | File |
|------|------|
| Begin login / auth URL | `packages/app-lib/src/state/minecraft_auth.rs` (`login_begin`) |
| Tauri WebView + code capture | `apps/app/src/api/auth.rs` (`login`) |
| Finish: OAuth → SISU → XSTS → Minecraft | `minecraft_auth.rs` (`login_finish`, `sisu_authorize`, `xsts_authorize`, `minecraft_token`) |
| Entitlements / profile | `minecraft_entitlements`, `Credentials::online_profile` |
| Launch args (`user_type=msa`) | `packages/app-lib/src/launcher/args.rs` |

### Troubleshooting

| Symptom | Likely cause |
|---------|----------------|
| Blank WebView2 | GPU/WebView2 runtime missing; try software rendering / reinstall WebView2 Evergreen |
| Redirect never completes | URL must start with `https://login.live.com/oauth20_desktop.srf` and include `code=`; window closed early |
| Scheme handler ignored | MS login does **not** require `owyx://` today; scheme is for deep links / packs |
| `invalid_grant` / account removed | Refresh token revoked — sign in again |
| Offline nickname fails on Hypixel etc. | Expected — offline accounts are for offline-mode servers only |

---

## Русский

### Что работает сейчас

Owyx использует существующий путь Theseus:

- Публичный client id Minecraft: `00000000402b5328`
- Цепочка: Microsoft OAuth → Xbox → Minecraft services
- Ответ в WebView: `oauth20_desktop.srf?code=…` (`apps/app/src/api/auth.rs`)
- Схема приложения: **`owyx://`**; `modrinth://` ещё принимается для совместимости с апстримом

Офлайн-ник (`login_offline`) — отдельный путь без токенов Microsoft.

### Когда нужен свой Azure / Entra

Свой app registration — для своего бренда на экране согласия, своих redirect URI (`owyx://…`, loopback) или device-code. Для текущего desktop-flow публичный Minecraft client id обычно достаточен.

### Куда класть client id

Через env / build config. **Не коммитить секреты.** Константа сейчас в `packages/app-lib/src/state/minecraft_auth.rs`.

### Если что-то ломается

- Пустой WebView2 → runtime / GPU
- Нет редиректа → окно закрыли или URL без `code`
- Офлайн-ник не заходит на лицензионные сервера → так и задумано (только offline-mode)

См. также архивный набросок: `docs/MICROSOFT_AUTH.owyxold-reference.md` (старый Path A / device-code; не путать с текущим SISU WebView).
