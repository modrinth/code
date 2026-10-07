# Dependency Injection

This standard applies only to `apps/app-frontend`. It follows the target structure in [Application Structure](APP_FRONTEND_STRUCTURE.md), while the website keeps its existing setup.

The [shared DI guide](../DEPENDENCY_INJECTION.md) describes `createContext` and the existing shared contracts.

In the app, use DI when several descendants need the same value or a shared UI component needs desktop behavior. Props and emits are easier to follow for direct children and short component chains. Reusing a composable does not require DI unless its callers need to share state.

Vue's [provide/inject guide](https://vuejs.org/guide/components/provide-inject.html) explains ancestor providers, optional defaults, reactive values, and symbol keys.

## Where the value comes from

Create app-wide values, such as the API client and file picker, once during root setup. Create page state in the page or route layout, and modal state in the modal that needs it. That way, each value lasts as long as the part of the app that uses it.

Keep registration visible in root setup rather than hiding it inside another feature. For example, starting installation should not also register the app's file picker. The root can use named setup functions when it has several providers to connect.

Creating a value and providing it are separate steps. The client factory belongs in `platform/modrinth-client.ts`; root setup provides its result:

```ts
import { provideModrinthClient } from '@modrinth/ui'

import { createAppClient } from '@/platform/modrinth-client'

const client = createAppClient()
provideModrinthClient(client)
```

Descendant components can now inject that client. Other setup code in the same component uses the local `client` variable or receives it as an argument. A component cannot inject a value that it provides itself, because injection reads its ancestors.

The [Vue dependency injection API reference](https://vuejs.org/api/composition-api-dependency-injection.html) describes ancestor lookup and the requirement to call `provide` and `inject` synchronously during setup.

Register the providers during setup before asynchronous loading starts. Listeners and resources created with them should end with their owning component. [Composables](APP_FRONTEND_COMPOSABLES.md) explains how to handle that cleanup.

## Keep the context close to its purpose

An app-only context belongs with its feature, page, or component. A contract already used by shared UI stays in `packages/ui`, with its desktop implementation in the app. The application folder structure does not change shared UI's own folders.

Use the existing `createContext` pattern for new component contexts. Keep the type and injection functions separate from lengthy feature behavior. Choose a unique context name, and preserve its key when moving an existing definition. The factory uses `Symbol.for`, so matching names identify the same context.

Share the state and actions that children need, such as `openSettings()` or `requestSignIn()`. Keep component refs inside the component that manages them. Children should not need to know which modal implements an action.

Keep account actions in a typed feature context when callers outside the sidebar need them. The shell can forward calls to its account component through that interface. If the provided value depends on a component ref, derive it reactively so consumers see the component when it mounts or is replaced. Treat the value as unavailable while an async component is still loading.

A small interface and object are usually enough for an app-only context. Keep existing shared manager classes when their contracts require them. Adding more fields to a context does not, by itself, require another class.

## Handle missing providers

A required injection should fail when its provider is missing. Fix where the value is provided, or pass it directly to the function that needs it. Do not catch that error and return a module-level singleton, because that hides a broken component relationship.

Use `injectSomething(null)` when the component supports running without that context. Check for `null` before using it. An empty fallback function can hide a required action that never runs.

DI shares an existing value rather than creating another copy of backend data. Components reading the same resource should use the same query definitions and cache. Keep contexts focused on one purpose instead of combining every app service into one large context.
