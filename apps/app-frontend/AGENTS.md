# apps/app-frontend - Modrinth App Frontend

Vue 3 + Vite frontend for the Modrinth App. It runs inside a Tauri webview (`apps/app`) and talks to `packages/app-lib` (`theseus`) through Tauri commands.

## Standards

Read the standards for the area you are changing before editing:

- `standards/frontend/app-frontend/APP_FRONTEND_STRUCTURE.md` — where code goes (`app/`, `platform/`, `features/`, `shared/`)
- `standards/frontend/app-frontend/APP_FRONTEND_COMPOSABLES.md` — when and where to create composables
- `standards/frontend/app-frontend/APP_FRONTEND_DEPENDENCY_INJECTION.md`
- `standards/frontend/app-frontend/APP_FRONTEND_FETCHING_DATA.md`

The codebase is partway through migrating to that structure. Put new code in the new structure, even when the surrounding code still lives in `pages/`, `helpers/`, `providers/` or `composables/`.

- Do not add logic to `App.vue` or `routes.js`. Put the behavior in the feature it belongs to, or in `app/runtime/` if it coordinates the whole app, and call it from there.
- Feature code calls functions in `platform/` instead of importing `@tauri-apps/api` directly.
- Shared components belong in `packages/ui`. See `packages/ui/AGENTS.md`.

## Tauri

### Drag and drop

Tauri intercepts native file drops, so plain DOM `drop` events don't receive files from the OS. Use `useFileDropTarget` from `@modrinth/ui`. It combines DOM drag events with the native `onDragDropEvent` listener that `providers/setup/file-drop.ts` provides. Don't attach a separate `onDragDropEvent` listener in a component.

### Logs

In dev builds, `console.*` calls from the main webview are forwarded to the Rust logger (target `webview`) and printed in the `pnpm app:dev` terminal. Use `useDebugLogger` from `@modrinth/ui` for temporary debug logging. To isolate it, grep the namespace you passed:

```bash
pnpm app:dev 2>&1 | grep "\[MyNamespace\]"
```

## Checks

- Lint and type check with `pnpm prepr:frontend:app` from the repo root, only when asked or before a PR.
- After changing or adding strings, run `pnpm intl:extract` in `apps/app-frontend`.
