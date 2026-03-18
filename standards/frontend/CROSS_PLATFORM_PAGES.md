# Cross-Platform Pages

Pages that need to exist in both the Modrinth Website (`apps/frontend`) and the Modrinth App (`apps/app-frontend`) live in `packages/ui/src/layouts/`. There are two categories based on whether the page logic differs between platforms.

## Shared Layouts (`layouts/shared/`)

For pages where the **logic differs** between the website and app (e.g. the app fetches data via Tauri `invoke` while the website uses `api-client`). Each shared layout is a self-contained module:

```
shared/content-tab/
├── layout.vue            # Main layout component
├── types.ts              # TypeScript types
├── components/           # Internal UI components
├── composables/          # Stateful logic (search, filtering, selection)
└── providers/            # DI context definitions
```

### How it works

1. A **DI contract** in `providers/` defines all platform-specific operations as an interface.
2. The **layout component** injects that context and handles all UI logic (search, filtering, selection, bulk operations, modals) without knowing the platform.
3. Each **platform provides its own implementation** of the contract.

### DI contract example

```ts
// shared/content-tab/providers/content-manager.ts
export interface ContentManagerContext {
	items: Ref<ContentItem[]> | ComputedRef<ContentItem[]>
	loading: Ref<boolean> | ComputedRef<boolean>

	// Platform-abstracted operations
	toggleEnabled: (item: ContentItem) => Promise<void>
	deleteItem: (item: ContentItem) => Promise<void>
	refresh: () => Promise<void>

	// Optional capabilities — not every platform supports everything
	hasUpdateSupport: boolean
	updateItem?: (id: string) => void
	bulkDeleteItems?: (items: ContentItem[]) => Promise<void>

	mapToTableItem: (item: ContentItem) => ContentCardTableItem
}

export const [injectContentManager, provideContentManager] =
	createContext<ContentManagerContext>('ContentPageLayout', 'contentManagerContext')
```

### Platform implementations

**Website** — uses `api-client` and TanStack Query:

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
	// ... rest of the contract
})
</script>

<template>
	<ContentPageLayout />
</template>
```

**App** — uses Tauri `invoke`:

```vue
<!-- apps/app-frontend/src/pages/instance/Mods.vue -->
<script setup lang="ts">
import { provideContentManager, ContentPageLayout } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

const items = ref<ContentItem[]>([])
await invoke('get_instance_content', { instanceId }).then(/* map to ContentItem[] */)

provideContentManager({
	items,
	deleteItem: async (item) => {
		await invoke('delete_content', { instanceId, path: item.file_path })
	},
	// ... rest of the contract
})
</script>

<template>
	<ContentPageLayout />
</template>
```

### Optional capabilities

The DI contract uses optional fields for features that not every platform supports. The layout checks for them before rendering the corresponding UI:

```ts
// Contract
bulkUpdateItems?: (items: ContentItem[]) => Promise<void>
shareItems?: (items: ContentItem[], format: string) => void

// Layout checks before showing UI
v-if="ctx.bulkUpdateItems && hasOutdatedProjects"
```

### Props vs DI

| Use       | When                                                                                       |
| --------- | ------------------------------------------------------------------------------------------ |
| **DI**    | Data depends on _how_ it's fetched — API calls, file operations, navigation (per-platform) |
| **Props** | Data is the same regardless of platform — configuration flags, display options              |

## Wrapped Pages (`layouts/wrapped/`)

For pages where the **logic is identical** on both platforms — same API source, same data fetching, same state management. These are full page-level Vue components that directly implement routes:

```
wrapped/hosting/manage/
├── index.vue
├── content.vue
├── backups.vue
├── files.vue
└── [id]/onboarding.vue
```

Wrapped pages handle their own data fetching (typically via TanStack Query and `api-client`) and are consumed as simple component imports in both frontends:

```vue
<!-- apps/frontend/src/pages/hosting/manage/[id]/content.vue -->
<script setup lang="ts">
import { ServersManageContentPage } from '@modrinth/ui'
</script>

<template>
	<ServersManageContentPage />
</template>
```

A wrapped page may still compose shared layouts internally — for example, the hosting content page uses the shared `content-tab` layout, providing its own `ContentManagerContext` with web API calls.

## Composables

Reusable stateful logic lives in `packages/ui/src/layouts/shared/*/composables/`. These are consumed internally by the shared layout:

- **Search** — Fuse.js fuzzy search over items
- **Filtering** — Dynamic filter pills
- **Selection** — Multi-select with bulk operation support
- **Bulk operations** — Sequential execution with progress tracking
