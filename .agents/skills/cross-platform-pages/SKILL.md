---
name: cross-platform-pages
description: Convert a page to the shared Modrinth page system for the website and desktop app. Use for shared layouts, wrapped layouts, or platform dependency-injection contracts.
---

# Convert a Cross-Platform Page

Read the applicable `AGENTS.md` files before you edit code.

Read these standards in full:

- [Cross-platform pages](../../../standards/frontend/CROSS_PLATFORM_PAGES.md)
- [Dependency injection](../../../standards/frontend/DEPENDENCY_INJECTION.md)

1. Identify the target page from the request.
2. Read the page and its route shell. Identify data sources, mutations, navigation, and platform APIs.
3. Use a wrapped layout when both platforms use the same API source and page logic.
4. Use a shared layout when platform data or operations have different implementations.

For a shared layout:

1. Define a provider contract for all platform operations.
2. Put common UI and state logic in the shared layout.
3. Put reusable search, filter, and selection logic in local composables.
4. Implement the contract in `apps/frontend/` and `apps/app-frontend/`.
5. Use optional contract fields only for capabilities that are not available on both platforms.

For a wrapped layout:

1. Move the page to `packages/ui/src/layouts/wrapped/` and preserve its route structure.
2. Replace platform-only imports with common utilities or provider calls.
3. Make each frontend route shell render the wrapped component.
4. Match primary query options in both route shells when the layout uses `ReadyTransition` and `useReadyState`.
5. Prefetch these queries with `ensureQueryData`, as the standard specifies.

Check that both route shells resolve their imports. Check that all required provider fields have implementations.

Run only the checks that the user or the applicable `AGENTS.md` permits.
