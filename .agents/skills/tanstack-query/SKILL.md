---
name: tanstack-query
description: Convert Vue server-state code to TanStack Query. Use for useQuery, useMutation, cache invalidation, optimistic updates, or replacement of useAsyncData and manual ref patterns.
---

# Convert Data Code to TanStack Query

Read the applicable `AGENTS.md` files before you edit code.

Read [the TanStack Query standard](../../../standards/frontend/FETCHING_DATA.md) in full.

1. Identify the target file from the request.
2. Find `useAsyncData`, `useFetch`, manual API refs, and fetch calls in `onMounted`.
3. Identify mutations that use manual loading, error, or result refs.

For queries:

1. Replace manual fetch logic with `useQuery`.
2. Get `api-client` with `injectModrinthClient()`.
3. Use a hierarchical query key with the resource, qualifier, and parameters.
4. Use a computed query key for reactive parameters.
5. Use a computed `enabled` option when the query depends on other data.
6. Use a shared query-option factory when multiple components use the query.

For mutations:

1. Replace manual mutation state with `useMutation`.
2. Invalidate or update related query data after success.
3. Use an optimistic update only when the UI needs an immediate response.
4. Cancel the applicable query and save its prior data before an optimistic update.
5. Restore the prior data after an error. Invalidate the query after settlement.

Remove manual loading and error refs that TanStack Query replaces. Remove obsolete `onMounted` fetch calls.

Keep Nuxt SSR behavior. Match route-shell prefetch options when `ReadyTransition` and `useReadyState` depend on the query.

Check query keys, invalidation prefixes, reactive values, and rollback data.

Run only the checks that the user or the applicable `AGENTS.md` permits.
