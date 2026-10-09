# Structure

This standard applies only to `apps/app-frontend`. It describes how to organise the desktop frontend as we add features and refactor existing code. The website and `packages/ui` keep their existing structures.

Keep the friend list, its state, and its actions together. Someone changing friends should be able to find that code under one feature.

Use this structure for new work and refactors in the area you are changing. Existing files can move gradually as those areas change.

The `app/`, `platform/`, `features/`, and `shared/` folders are our project convention. The Vue and Tauri references below explain the behavior of their APIs; they do not prescribe this folder structure.

```text
src/
├── main.js
├── app/
│   ├── App.vue
│   ├── router.ts
│   ├── providers.ts
│   ├── runtime/            # startup flow, auth handling, etc. anything core to the app frontend working
│   └── shell/              # sidebar, title bar, navigation etc.
├── platform/
│   ├── modrinth-client.ts  # packages/api-client impl
│   ├── app-lib/            # invoke stuff
│   │   ├── instances/
│   │   ├── friends/
│   │   └── settings/
│   ├── events/             # app events/listeners
│   └── adapters/           # any cross platform pages get implemented here
├── features/               # pages, flows, modals, etc.
│   ├── instances/
│   ├── installation/
│   ├── library/
│   ├── friends/
│   ├── settings/
│   └── skins/
│       ├── pages/
│       │   └── skins-page.vue
│       ├── skin-preview.vue
│       └── queries.ts
├── shared/
│   ├── components/
│   ├── composables/
│   └── utils/
├── generated/...
├── assets/...
└── locales/...
```

This tree shows the intended structure, rather than a completed migration. The feature names are examples, and a small feature may need only a few files. Create a directory when it helps organise code, rather than creating empty folders in advance.

## Keep related code in a feature

A feature contains the code for one part of the app, such as instances, skins, friends, or screenshots. Its pages, components, composables, query definitions, and actions belong together. For example, `features/instances/queries.ts` is where other parts of the app can find instance queries.

Within a feature, keep a component's supporting files beside that component. A composable used only by the friend list stays with the friend list. Code used throughout the friends feature can live directly under `features/friends/`. [Component Structure](../COMPONENT_STRUCTURE.md) explains when a component needs its own folder.

Vue's [guide to extracting composables](https://vuejs.org/guide/reusability/composables.html#extracting-composables-for-code-organization) explains how separating related behavior can make a component easier to read, even when that behavior is not reused.

Desktop route views belong in the feature's `pages/` folder, with their routes registered in `app/router.ts`. For example, the skins route renders `features/skins/pages/skins-page.vue`, with skin previews and queries beside the page folder. A feature used only in the sidebar does not need a page.

A page can also render an existing layout from `@modrinth/ui`. Keep the route component in its feature and the shared layout in `packages/ui`. The adapters entry in the tree refers to the desktop implementations of the contracts those pages use.

## Let app connect the pieces

The `app/` folder contains startup, route registration, and the code that connects features to shared services. Its `shell/` folder contains the visible frame of the app: navigation, title bars, sidebars, and the area that displays routes. Plugins and directives that the app registers also belong here.

Keep the behavior behind those controls with the feature it serves. An update button belongs in the shell, while the update workflow belongs in `features/updates/`. The shell can call that workflow without containing all its rules.

The sidebar arranges feature components such as accounts, friends, onboarding, and news. Those components stay in their features; sidebar layout and promotion placement belong in the shell.

Use `app/runtime/` for work that coordinates the whole app, such as startup order, session restoration, route loading, or commands that open different features. Root setup creates app-wide services and registers their providers before loading data. The installation feature should not quietly register an unrelated file picker or user-country provider.

Root setup can contain several calls, as long as someone reading it can see how the app starts and which values it shares. [Dependency Injection](APP_FRONTEND_DEPENDENCY_INJECTION.md) explains how those values reach child components.

When moving shell code, preserve its props, events, exposed actions, provider scope, and loading behavior. Check remaining script references before removing imports, including icons used in computed menu options. A signed-in menu can fail to render even after authentication succeeds.

## Put native access in platform

The `platform/` folder is where the frontend talks to app-lib and the computer. Typed app-lib calls belong in `platform/app-lib/`, grouped by the area they serve. Native file dialogs, window operations, updates, and the desktop API client also belong under `platform/`.

Use `platform/modrinth-client.ts` to configure the client from `packages/api-client`, including URLs, auth, and logging. Root setup creates and provides that client.

An app-lib wrapper should call a command and return its typed result. The feature decides how to use that result, whether to show a notification, or which modal to open. For example, the settings command can move to `platform/app-lib/settings/commands.ts`:

```ts
import { invoke } from '@tauri-apps/api/core'

import type { AppSettings } from './types'

export function get(): Promise<AppSettings> {
	return invoke('plugin:settings|settings_get')
}
```

Keep those command modules independent of Vue setup and TanStack Query. Their types should describe what app-lib actually returns. The settings feature then adds query options, form state, and any changes needed to display the result.

Tauri's [guide to calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/) explains how commands receive arguments and return results through `invoke`.

Use `platform/events/` for receiving and decoding app events. Use `platform/adapters/` for native implementations of existing shared UI contracts, such as the file picker. Feature code uses these functions instead of importing Tauri directly.

## Keep shared small

The app's `shared/` folder is for code that several features use and no particular feature owns. A generic input component or formatting function can belong here. Code that still describes instances belongs to instances, even when the library also uses it.

This folder should remain independent of features and native operations. Components already shared through `packages/ui` stay in that package. The app supplies any desktop behavior through their existing contracts.

## Keep dependencies easy to follow

The app connects features, and features use platform functions and shared code. Platform code must not depend on feature implementations. Shared code must not depend on app setup, features, or native implementations.

A feature can use another feature's named queries or actions when it needs them. It should not reach into that feature's private components or route views. If two features depend on each other, move their shared work to one place or coordinate them from `app/runtime/`.

As existing helpers move, separate their different jobs. Friend commands go to `platform/app-lib/friends/`, while friend transformations and UI behavior stay in `features/friends/`. New code should follow those names instead of expanding general `helpers`, `providers`, or `composables` folders at the root.

[Composables](APP_FRONTEND_COMPOSABLES.md) explains how to separate reactive behavior within a feature. The [app data guidance](APP_FRONTEND_FETCHING_DATA.md) explains where queries belong and how to keep their state consistent.
