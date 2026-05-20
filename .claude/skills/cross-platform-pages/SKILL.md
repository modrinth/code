---
name: cross-platform-pages
description: Convert a page to the cross-platform page system so it works in both the website and the desktop app. Use when moving a page into packages/ui/src/layouts/, creating shared or wrapped layouts, or setting up DI contracts for platform abstraction.
argument-hint: <path-to-page>
---

Refer to the standards: @standards/frontend/CROSS_PLATFORM_PAGES.md and @standards/frontend/DEPENDENCY_INJECTION.md

## Steps

1. **Read the target page** at `$ARGUMENTS` and understand its data sources, mutations, and navigation.
2. **Read the standards above** to understand the shared vs wrapped distinction and the DI pattern.
3. **Decide the category:**
   - **Wrapped** (`layouts/wrapped/`) — if the page uses the same API source on both platforms (e.g. web requests, not Tauri plugins). Just move the page component into `packages/ui` and import it from both frontends.
   - **Shared** (`layouts/shared/`) — if the page has different data-fetching logic per platform (e.g. website uses `api-client`, app uses Tauri `invoke`). Requires a DI contract.
4. **For shared layouts:**
   - Define a DI contract interface in `providers/` capturing all platform-specific operations.
   - Create the layout component that injects the context and handles all UI logic.
   - Extract reusable stateful logic (search, filtering, selection) into `composables/`.
   - Implement the contract separately in each frontend (`apps/frontend/`, `apps/app-frontend/`).
5. **For wrapped pages:**
   - Move the page component into `packages/ui/src/layouts/wrapped/` matching the route structure.
   - Replace any platform-specific imports with shared utilities.
   - Import and render the wrapped page from both frontends as a simple component.
   - If the layout uses TanStack Query for initial route paint with `ReadyTransition` / `useReadyState`, each platform route shell must call `ensureQueryData` for those queries with matching keys and fetchers — see **Platform route shells: prefetch with `ensureQueryData`** in `standards/frontend/CROSS_PLATFORM_PAGES.md`.
6. **Verify** the page renders correctly by checking for missing imports and that all DI contracts are satisfied.
