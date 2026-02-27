# Cross-Platform Page System

When a page needs to exist in both the Modrinth App (`apps/app-frontend`) and the Modrinth Website (`apps/frontend`), use the cross-platform page system.

## How It Works

1. **Pages live as Vue SFCs in `packages/ui`** — either in `src/pages/` or `src/layout/` (if `src/pages/` doesn't exist, it's been renamed to `src/layout/`).
2. **Platform-dependent data flows via DI** — the app uses Tauri `invoke` commands, the website uses `api-client` or the legacy `useBaseFetch` composable. The shared page never knows which. See the `dependency-injection` skill for full DI docs.
3. **Non-platform-dependent data flows via props** — if data doesn't change based on _how_ it's fetched, just pass it as a prop.

## Example: Content Page

`ContentPageLayout` demonstrates the full pattern.

### 1. Define a DI contract in `packages/ui/src/providers/`

The provider interface abstracts all platform-specific operations:

```ts
// packages/ui/src/providers/content-manager.ts
export interface ContentManagerContext {
	items: Ref<ContentItem[]>
	loading: Ref<boolean>
	error: Ref<Error | null>
	contentTypeLabel: Ref<string>

	// These are the platform-abstracted operations:
	// App uses invoke(), website uses api-client
	toggleEnabled: (item: ContentItem) => Promise<void>
	deleteItem: (item: ContentItem) => Promise<void>
	refresh: () => Promise<void>
	browse: () => void
	uploadFiles: () => void

	// Optional capabilities — not every platform supports everything
	hasUpdateSupport: boolean
	updateItem?: (item: ContentItem) => Promise<void>
	bulkUpdateItem?: (items: ContentItem[]) => Promise<void>

	mapToTableItem: (item: ContentItem) => ContentCardTableItem
}

export const [injectContentManager, provideContentManager] =
	createContext<ContentManagerContext>('ContentManager')
```

### 2. Build the shared page in `packages/ui`

The page component injects the context and handles all UI logic (search, filtering, selection, bulk operations, empty states, modals) without knowing the platform:

```vue
<!-- packages/ui/src/components/instances/ContentPageLayout.vue -->
<script setup lang="ts">
import { injectContentManager } from '../../providers/content-manager'

const { items, loading, toggleEnabled, deleteItem, refresh, mapToTableItem } =
	injectContentManager()

// All UI logic lives here — search, filters, sort, bulk ops, etc.
</script>

<template>
	<ContentCardTable :items="filteredItems" />
</template>
```

### 3. Each platform provides its implementation

**Website (Nuxt)** — uses `api-client` or `useBaseFetch`:

```vue
<!-- apps/frontend/src/pages/hosting/manage/[id]/content.vue -->
<script setup lang="ts">
import { provideContentManager, ContentPageLayout } from '@modrinth/ui'
const { labrinth } = injectModrinthClient()

const { data: items } = useQuery({
	queryKey: ['content', serverId],
	queryFn: () => labrinth.servers_v0.getAddons(serverId),
})

provideContentManager({
	items: computed(() => items.value?.map(addonToContentItem) ?? []),
	deleteItem: async (item) => {
		await labrinth.servers_v0.deleteAddon(serverId, item.id)
	},
	// ... rest of the contract
})
</script>

<template>
	<ContentPageLayout />
</template>
```

**App (Tauri)** — uses `invoke`:

```vue
<!-- apps/app-frontend/src/pages/instance/Content.vue -->
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

## When to Use Props vs DI

| Use       | When                                                                                                     |
| --------- | -------------------------------------------------------------------------------------------------------- |
| **DI**    | The data depends on _how_ it's fetched (different per platform) — API calls, file operations, navigation |
| **Props** | The data is the same regardless of platform — configuration flags, display options                       |

## Composables for Shared Logic

Extract reusable stateful logic into composables in `packages/ui/src/composables/`. The shared page orchestrates them internally:

- Search (Fuse.js fuzzy search over items)
- Filtering (dynamic filter pills)
- Selection (multi-select with bulk operations)
- Bulk operations (sequential execution with progress tracking)

## Key Files

- `packages/ui/src/pages/` (or `src/layout/`) — shared page components
- `packages/ui/src/providers/` — DI contracts
- `packages/ui/src/composables/` — shared stateful logic
- `apps/frontend/src/app.vue` — website root provider setup
- `apps/app-frontend/src/App.vue` — app root provider setup
- `apps/app-frontend/src/routes.js` — app route definitions
