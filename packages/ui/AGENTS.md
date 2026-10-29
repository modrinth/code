# `@modrinth/ui`

The shared UI package used by `apps/frontend` (Nuxt 3) and `apps/app-frontend` (Vue 3 + Tauri).

## Rules

- Components are platform-agnostic. Platform behavior comes in through dependency injection, never through Nuxt or Tauri imports.
- Every folder under `src/components/` has an `index.ts` barrel. The package root `index.ts` re-exports the public API.
- `src/providers/` holds only platform contracts the host apps implement and app-wide page state. A context for one layout lives in that layout's `providers/` folder. A context that coordinates one component lives beside that component. See `standards/frontend/DEPENDENCY_INJECTION.md`.
- Cross-platform pages live in `src/layouts/` and import the rest of the package through the `#ui/*` alias. See `standards/frontend/CROSS_PLATFORM_PAGES.md` for `shared/` versus `wrapped/`.
- Use `bg-surface-*` and `border-surface-*` utilities for backgrounds and strokes, never aliased `bg-*` color variables. See `standards/frontend/SURFACE_SYSTEM.md` for the layer scale, text colors, and palette.
- Access `window` or `document` only as described in `standards/frontend/POPOUT-WINDOWS.md`.
- When you change a component in `src/components/`, update its story in `src/stories/`, or create one. Skip this for minor or styling-only changes.
- Never newline-separate statements in a template event handler. Vue cannot parse `@click="a = true\n$emit('b')"`. Write `@click="a = true; $emit('b')"`.

## Key Providers

- `provideModrinthClient` / `injectModrinthClient`: API client
- `provideNotificationManager` / `injectNotificationManager`: notifications

## Folder Structure

```
src/
├── components/   # Vue components by feature domain
├── composables/  # Vue composition API hooks
├── layouts/      # Cross-platform pages (shared/ modules and wrapped/ pages)
├── providers/    # Platform DI contracts
├── utils/        # Utility functions and constants
├── locales/      # FormatJS locale files
├── styles/       # Tailwind CSS utilities
└── stories/      # Storybook stories
```
