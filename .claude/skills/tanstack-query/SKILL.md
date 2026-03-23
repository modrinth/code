---
name: tanstack-query
description: Convert a page or component from useAsyncData/manual ref patterns to TanStack Query for server state management. Use when migrating data fetching to useQuery/useMutation, adding cache invalidation, or replacing useAsyncData with TanStack Query.
argument-hint: <path-to-file>
---

Refer to the standard: @standards/frontend/FETCHING_DATA.md

## Steps

1. **Read the target file** at `$ARGUMENTS` and identify all data-fetching patterns: `useAsyncData`, `useFetch`, manual `ref()` + `await`, or `onMounted` fetch calls.
2. **Read the standard above** for the query/mutation patterns, query key conventions, and optimistic update approach.
3. **Convert queries:**
   - Replace `useAsyncData` / `useFetch` / manual fetches with `useQuery`.
   - Use the `api-client` via `injectModrinthClient()` for the `queryFn`.
   - Design query keys with the `['resource', 'version', ...params]` convention.
   - Use `computed` query keys for reactive parameters.
   - Use the `enabled` option for conditional queries that depend on other data.
4. **Convert mutations:**
   - Replace manual `try/catch` + `ref` patterns with `useMutation`.
   - Add `onSuccess` handlers that invalidate or update related query caches.
   - Consider optimistic updates for UI-critical mutations (follow the pattern in the standard).
5. **Clean up:**
   - Remove manual loading/error `ref()`s that are now handled by TanStack Query's return values (`isPending`, `isError`, `error`).
   - Remove manual `onMounted` fetch calls.
   - Ensure SSR compatibility — queries in Nuxt pages are automatically awaited during SSR.
6. **Verify** the page still renders correctly and that cache invalidation triggers re-fetches where expected.
