# Composables

This standard applies to composables in `apps/app-frontend`. It follows the folder structure in [Application Structure](APP_FRONTEND_STRUCTURE.md).

A composable is a function that manages related Vue state or behavior. It might handle selection in the library, track installation progress, or listen for app events. Its name should describe that job, such as `useLibrarySelection` in `use-library-selection.ts`.

## When a composable helps

Extract a composable when several components need the same behavior, or when a component contains a separate job that hides its main purpose. Reuse is not a requirement. A composable used by one component can still make that component easier to read.

Keep small, clear state in the component. A single open flag rarely needs another file, while selection with several related actions may benefit from one. Dividing a long script into one equally long composable does not make its responsibilities clearer.

Functions that only parse, format, or transform values remain ordinary functions. The same applies to app-lib wrappers and functions that return query options. Reserve the `use` prefix for functions that need Vue state or setup. Use `create` for factories that can run without component context.

Keep layout in the component: wrappers, sizing, overflow, and overlay placement. A composable can track scroll position without deciding where the sidebar or its promotion belongs.

## Keep it near the code that uses it

A composable used by one component belongs in that component's folder. If several parts of a feature use it, keep it in the feature. Only move it to `shared/composables/` when it has no feature-specific behavior.

For example, a composable for selecting instances belongs to the library or instances feature. It does not become generic because more than one component uses it. Composables that coordinate startup or the whole app belong in `app/runtime/`.

## Be clear about state and inputs

Each call normally creates its own local state. Two components calling the same composable do not automatically share that state. When they need the same value, create it once in a common parent and pass it through props or DI.

Keep backend data in TanStack Query and derive display values from its results. Copying those results into another writable store creates two places that need to stay consistent. An editable form draft is different, because the user needs to change it before saving.

If an input can change while the composable is active, accept a ref or getter instead of capturing its initial value. Read it inside `computed`, `watch`, or reactive query options. This example uses the existing instance query factory from the proposed feature folder:

```ts
import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { instanceDetailQueryOptions } from './queries'

export function useInstanceQuery(instanceId: MaybeRefOrGetter<string>) {
	return useQuery(computed(() => instanceDetailQueryOptions(toValue(instanceId))))
}
```

Here, changing the instance ID changes the query. The query factory itself remains an ordinary function that can also serve prefetch code.

Return the refs and actions that callers need, with internal details kept inside the composable. A plain object of refs allows callers to destructure the result without losing reactivity. Avoid destructuring primitive fields from a reactive object.

Vue's [composable conventions and best practices](https://vuejs.org/guide/reusability/composables.html#conventions-and-best-practices) cover naming, reactive inputs, returned refs, and when composables can be called.

## Make setup and cleanup part of the job

Call a composable that uses injection or lifecycle hooks during setup, before asynchronous work starts. It can inject an existing context there. Ordinary functions and query-option factories should receive their dependencies as arguments instead.

Create DOM-dependent composables during setup with a nullable element ref, then assign the element after mount. For OverlayScrollbars, connect `useScrollIndicator` to `elements().viewport` and recheck when the scrollbar instance updates. Keep the scroll container constrained by its flex layout; fades can sit outside the viewport as absolute overlays with pointer events disabled.

If a control overlays scrolling content, reserve space equal to its rendered height. Measure controls whose text can wrap. When a fade also provides the background behind that control, keep it visible while the control is shown, including at the end of the scroll area.

Create listeners when the composable runs, rather than when its module loads. Remove external listeners and timers with `onScopeDispose`, and cancel pending work when the API allows it. Vue stops setup-owned watchers automatically, but external subscriptions need their own cleanup.

The [`onScopeDispose` reference](https://vuejs.org/api/reactivity-advanced.html#onscopedispose) explains how cleanup runs when the current effect scope ends, including the scope owned by a component's setup.

Give shared subscriptions a clear lifetime. A listener used throughout the app belongs in root setup, while a page-specific listener belongs with that page. Avoid module-level refs and singleton fallbacks that hide where shared state starts or ends.

If a function also supports callers outside Vue setup, make its cleanup function explicit. Callers need to know which resources they must release.

The [app DI guidance](APP_FRONTEND_DEPENDENCY_INJECTION.md) explains when components should share a value. The [app data guidance](APP_FRONTEND_FETCHING_DATA.md) explains how composables work with backend queries.
