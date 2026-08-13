# Cross-Platform Pages

Put pages for both Modrinth Website and Modrinth App in `packages/ui/src/layouts/`.

Use one of two layout types. Select the type from the differences between the platform logic.

## Shared Layouts (`layouts/shared/`)

Use a shared layout when the website and app use different logic.

For example, the app can use Tauri `invoke`, and the website can use `api-client`.

Make each shared layout a self-contained module:

```
shared/content-tab/
├── layout.vue            # Main layout component
├── types.ts              # TypeScript types
├── components/           # Internal UI components
├── composables/          # State logic for search, filters, and selection
└── providers/            # DI context definitions
```

### Structure

1. Define all platform operations in a dependency-injection (DI) contract in `providers/`.
2. Inject the contract into the layout component. Keep all common UI logic in this component.
3. Provide a different contract implementation from each platform.

Common UI logic can include search, filters, selection, bulk operations, and modals.

### DI Contract Example

```ts
// shared/content-tab/providers/content-manager.ts
export interface ContentManagerContext {
	items: Ref<ContentItem[]> | ComputedRef<ContentItem[]>
	loading: Ref<boolean> | ComputedRef<boolean>

	// Operations that have platform-specific implementations.
	toggleEnabled: (item: ContentItem) => Promise<void>
	deleteItem: (item: ContentItem) => Promise<void>
	refresh: () => Promise<void>

	// Optional capabilities are not available on all platforms.
	hasUpdateSupport: boolean
	updateItem?: (id: string) => void
	bulkDeleteItems?: (items: ContentItem[]) => Promise<void>

	mapToTableItem: (item: ContentItem) => ContentCardTableItem
}

export const [injectContentManager, provideContentManager] =
	createContext<ContentManagerContext>('ContentPageLayout', 'contentManagerContext')
```

### Platform Implementations

The website uses `api-client` and TanStack Query:

```vue
<!-- apps/frontend/src/pages/instance/content.vue -->
<script setup lang="ts">
import { provideContentManager, ContentPageLayout } from '@modrinth/ui'

const { data: items } = useQuery({
	queryKey: ['content', instanceId],
	queryFn: () => client.content_v1.getAddons(instanceId),
})

provideContentManager({
	items: computed(() => items.value?.map(addonToContentItem) ?? []),
	deleteItem: async (item) => {
		await client.content_v1.deleteAddon(instanceId, item.id)
	},
	// Implement the remaining contract fields.
})
</script>

<template>
	<ContentPageLayout />
</template>
```

The app uses Tauri `invoke`:

```vue
<!-- apps/app-frontend/src/pages/instance/Mods.vue -->
<script setup lang="ts">
import { provideContentManager, ContentPageLayout } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

const items = ref<ContentItem[]>([])
await invoke('get_instance_content', { instanceId }).then(/* Map the result to ContentItem[]. */)

provideContentManager({
	items,
	deleteItem: async (item) => {
		await invoke('delete_content', { instanceId, path: item.file_path })
	},
	// Implement the remaining contract fields.
})
</script>

<template>
	<ContentPageLayout />
</template>
```

### Optional Capabilities

Use optional contract fields for capabilities that are not available on all platforms.

Check that an optional field exists before you show its UI:

```ts
// Contract fields.
bulkUpdateItems?: (items: ContentItem[]) => Promise<void>
shareItems?: (items: ContentItem[], format: string) => void

// Show the UI only when the capability exists.
v-if="ctx.bulkUpdateItems && hasOutdatedProjects"
```

### Props and DI

| Use   | Condition                                                                  |
| ----- | -------------------------------------------------------------------------- |
| DI    | Use when API calls, file operations, or navigation differ by platform.     |
| Props | Use when configuration and display data are the same on all platforms.      |

## Wrapped Pages (`layouts/wrapped/`)

Use a wrapped page when both platforms use the same API source, data logic, and state logic.

A wrapped page is a complete page-level Vue component. Its directory structure matches the route structure:

```
wrapped/hosting/manage/
├── index.vue
├── content.vue
├── backups.vue
├── files.vue
└── [id]/onboarding.vue
```

Wrapped pages get their own data. They usually use TanStack Query and `api-client`.

Import the wrapped page as a simple component in both frontends:

```vue
<!-- apps/frontend/src/pages/hosting/manage/[id]/content.vue -->
<script setup lang="ts">
import { ServersManageContentPage } from '@modrinth/ui'
</script>

<template>
	<ServersManageContentPage />
</template>
```

### Prefetch Data in Platform Route Shells

#### `ReadyTransition` and `useReadyState`

Many wrapped pages put the main UI in [`ReadyTransition`](../../packages/ui/src/components/base/ReadyTransition.vue).

The `:pending` prop usually comes from [`useReadyState`](../../packages/ui/src/composables/use-ready-state.ts) for the primary TanStack query.

The state is true only during the first load when the cache has no data. Background refetches keep the page ready.

This behavior prevents empty content from appearing before the data exists.

```vue
<!-- This code is in a packages/ui wrapped layout. -->
<ReadyTransition :pending="readyPending">
	<SomePageLayout />
</ReadyTransition>
```

```ts
const primaryQuery = useQuery({ /* Query options. */ })
const readyPending = useReadyState(primaryQuery)

// Use this form when the complete query object is not available.
const readyPendingFromState = useReadyState({ isLoading, data })
```

Shell prefetch adds data to the cache before the layout mounts. On this fast path, `pending` stays false.

`ReadyTransition` can then omit its enter animation. Refer to the `ReadyTransition` documentation and stories for details.

#### Use `ensureQueryData` in Each Route Shell

When a wrapped layout uses this ready-state pattern, prefetch the primary query in each thin platform page.

For each query that controls the first paint, call `queryClient.ensureQueryData` in the website and app route shells.

Use the same `queryKey`, `queryFn`, and `staleTime` that the wrapped layout uses.

Put the call in a `try` block. Catch the error so that route setup can continue.

The mounted layout runs its `useQuery` call and shows the error to the user.

```ts
import { injectModrinthClient, injectModrinthServerContext, ServersManageFilesPage } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()

try {
	await queryClient.ensureQueryData({
		queryKey: ['files', serverId, '/'],
		queryFn: () => client.kyros.files_v0.listDirectory('/', 1, 2000),
		staleTime: 30_000,
	})
} catch {
	// Let the mounted layout show the query error. Do not stop route setup.
}
```

If the query needs a route parameter, call `ensureQueryData` only when the parameter exists.

Make this condition match the `enabled` condition in the layout query.

Duplicate query definitions in the shell until a shared query-option module exists. Keep the keys and fetch functions the same.

A wrapped page can contain shared layouts. For example, a hosting page can provide a `ContentManagerContext` to the shared content layout.

## Composables

Put reusable state logic in `packages/ui/src/layouts/shared/*/composables/`. The shared layout uses these composables:

- Search: Uses Fuse.js to search items.
- Filters: Supplies dynamic filter pills.
- Selection: Supplies item selection for bulk operations.
- Bulk operations: Runs operations in sequence and tracks progress.
