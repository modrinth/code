# Fetching Data

This standard applies only to `apps/app-frontend` and follows [Application Structure](APP_FRONTEND_STRUCTURE.md).

Use TanStack Query for backend data that the app displays, including data returned by app-lib commands. Keep the raw command in `platform/app-lib/` and its query definitions in the feature that owns the resource. Instance queries belong to instances, even when the library also uses them.

The same placement applies to HTTP resources such as the news feed. Keep its query options in `features/news/queries.ts` and read them in the news component. The sidebar renders that component; root startup does not need to fetch its articles into a separate ref. Keep the query key, cache settings, and refresh rules with the query when moving it.

When two views need the same data, use the same query keys and options. Read the query result directly or derive display values with `computed`. A separate writable copy would need its own updates whenever the query changes. Form drafts can keep editable copies until the user saves them.

TanStack's [query key guide](https://tanstack.com/query/latest/docs/framework/vue/guides/query-keys) explains how keys identify cached data. Its [Vue reactivity guide](https://tanstack.com/query/latest/docs/framework/vue/reactivity) explains how refs and getters keep queries responsive to changing inputs.

Query-option factories should receive their dependencies as arguments, so components and prefetch code can both use them. They do not need injection or the `use` prefix. [Composables](APP_FRONTEND_COMPOSABLES.md) covers the reactive behavior around those queries.

The [query options guide](https://tanstack.com/query/latest/docs/framework/vue/guides/query-options) shows how to reuse query definitions across components and prefetch calls.

Keep refresh rules with the feature that owns the data. App-wide event subscriptions start once at the root and refresh the affected queries. The frontend cache works alongside app-lib's existing caches, without replacing their backend responsibilities.

## Network mode

TanStack Query defaults to `networkMode: 'online'`, which pauses queries and mutations while the system reports offline. App-lib commands that only touch local state (SQLite, settings, files, instance state, or app-lib's cache) must set `networkMode: 'always'`, otherwise they never run offline and the UI shows empty or stale state. Put it in the query-option factory so prefetch and `ensureQueryData` callers get it too. For a mutation, `onMutate` still runs while it is paused, so the optimistic update appears but the write never happens.

Keep the default for anything that needs the network: Modrinth API client calls, the server panel, billing, and app-lib commands that make network requests on the Rust side (invites, friends, shared-instance previews and eligibility, server pings). Do not change the default on the global `QueryClient`. If a function does both local and network work, split it, or keep the default and handle the offline case explicitly.

The [network mode guide](https://tanstack.com/query/latest/docs/framework/vue/guides/network-mode) explains each mode.

Native actions such as opening a file dialog or closing a window can use platform functions directly. They do not need query caching because they do not read a backend resource.

The [shared TanStack Query guide](../FETCHING_DATA.md) contains examples of queries, mutations, and optimistic updates.
