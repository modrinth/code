# apps/frontend — Modrinth Website

Nuxt 3 application serving the main Modrinth website. Uses Vue 3, Tailwind CSS v3, and file-based routing.

## Architecture

Nuxt 3 with SSR — pages are server-rendered and hydrated on the client. Uses `$fetch` for server-side data fetching and `@modrinth/api-client` (via `NuxtModrinthClient`) for client-side API calls.

## Key Directories

- **`src/pages/`** — file-based routing (`[param].vue` for dynamic segments, nested folders for nested routes)
- **`src/components/`** — website-specific components (not shared with the app)
- **`src/composables/`** — Vue composables, including `queries/` for TanStack Query options
- **`src/providers/`** — page-level DI context providers (e.g., version modal, project page)
- **`src/plugins/`** — Nuxt plugins (TanStack Query setup, theme, etc.)
- **`src/middleware/`** — route guards and auth checks
- **`src/layouts/`** — Nuxt layout components
- **`src/server/`** — server-side plugins, routes, and utilities
- **`src/store/`** — Pinia state management
- **`src/helpers/`** — utility functions
- **`src/locales/`** — i18n translation files

## Components

**Website-specific components go in `src/components/`.** These are components that only make sense in the website context — admin panels, moderation tools, dashboard widgets, brand components, etc.

**Shared components go in `packages/ui`.** If a component could be used by both the website and the desktop app, it belongs in `packages/ui/src/components/`. See `packages/ui/CLAUDE.md` for UI standards, color rules, and component patterns.

Rule of thumb: if it doesn't depend on Nuxt-specific APIs or website-only features, it should be in `packages/ui`.

## Data Fetching

Use `@modrinth/api-client` via `injectModrinthClient()` for all API calls. See `packages/api-client/CLAUDE.md` for the full API client documentation.

For caching and server state, use TanStack Query (`@tanstack/vue-query`). See the `tanstack-query` skill (`.claude/skills/tanstack-query/SKILL.md`) for patterns and conventions used in this codebase.

### Deprecated Composables

These composables are deprecated and should not be used in new code:

- **`useAsyncData`** - we use tanstack, not nuxt's built in async data utility.
- **`useBaseFetch`** (`src/composables/fetch.js`) — legacy Labrinth fetch wrapper. Use `client.labrinth.*` modules instead.
- **`useServersFetch`** (`src/composables/servers/servers-fetch.ts`) — legacy Archon fetch wrapper with manual retry/circuit-breaker. Use `client.archon.*` modules instead — refer to the `packages/api-client/CLAUDE.md` for more information.
