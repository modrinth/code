# Site translations (EN / RU)

Add or edit keys in:

- `en_US.json` — English
- `ru_RU.json` — Russian

Keep the same key tree in both files. Legal docs live under `legal.terms|privacy|eula`.

Regenerate from the scripted source of truth (optional):

```bash
node owyxsite/frontend/scripts/generate-locales.mjs
```

Runtime loader: `lib/i18n.ts` + `hooks/useLocale.tsx`. Language toggle sits next to the account / download controls in the header.
