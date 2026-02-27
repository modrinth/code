# Dependency Injection

Modrinth uses a lightweight DI layer built on Vue's `provide`/`inject` for sharing platform-specific capabilities and page-level state across shared UI components.

## The `createContext` Factory

All providers are defined using `createContext` from `packages/ui/src/providers/index.ts` (adapted from Reka UI). It produces a typed `[inject, provide]` tuple:

```ts
import { createContext } from '@modrinth/ui'

interface MyContext {
	someValue: Ref<string>
	doSomething: () => void
}

export const [injectMyContext, provideMyContext] = createContext<MyContext>('MyComponent')
```

- **`provideMyContext(value)`** — call in a parent component's `setup()`.
- **`injectMyContext()`** — call in any descendant's `setup()`. Throws if never provided.
- **`injectMyContext(null)`** — returns `null` instead of throwing (for optional contexts).

## When to Use DI

Use DI when:
- **The same interface needs different implementations** depending on the platform (web vs desktop app).
- **Deeply nested components** need access to shared page-level state without prop drilling through 3+ levels.

### Platform Abstraction (Primary Use Case)

`packages/ui` components need capabilities that each frontend fulfils differently:

| Provider | App Frontend | Website Frontend |
|----------|-------------|-----------------|
| API client | Tauri IPC client | REST fetch client |
| Notifications | `ref()` state + app window mgmt | `useState()` for SSR hydration |
| File picker | Native Tauri dialogs | Browser file inputs |
| Tags | Tauri commands | Nuxt server state |
| Page context | `sidebar: true`, ad window hooks | `sidebar: false`, no ads |

### Page-Level Context

Sharing data between a page and deeply nested children — e.g. project page data consumed by sidebar, header, and version components.

## Creating a New Provider

### 1. Define the interface in `packages/ui/src/providers/`

```ts
// packages/ui/src/providers/my-feature.ts
import type { Ref } from 'vue'
import { createContext } from '.'

export interface MyFeatureContext {
	items: Ref<Item[]>
	addItem: (item: Item) => Promise<void>
	removeItem: (id: string) => Promise<void>
}

export const [injectMyFeature, provideMyFeature] = createContext<MyFeatureContext>('MyFeature')
```

Re-export from the barrel file (`packages/ui/src/providers/index.ts`).

### 2. For complex platform-specific logic, use an abstract class

```ts
export abstract class AbstractMyFeatureManager {
	abstract items: Ref<Item[]>
	abstract addItem(item: Item): Promise<void>

	// Shared logic lives on the base class
	handleError(err: unknown) {
		console.error(err)
	}
}

export const [injectMyFeature, provideMyFeature] =
	createContext<AbstractMyFeatureManager>('MyFeature')
```

See `AbstractWebNotificationManager` in `packages/ui/src/providers/web-notifications.ts` for a real example.

## Wiring Up Providers

### App Frontend (Tauri)

Create a setup function in `apps/app-frontend/src/providers/setup/`:

```ts
// apps/app-frontend/src/providers/setup/my-feature.ts
import { ref } from 'vue'
import { provideMyFeature } from '@modrinth/ui'

export function setupMyFeatureProvider() {
	const items = ref<Item[]>([])

	provideMyFeature({
		items,
		addItem: async (item) => {
			await invoke('add_item', { item })
			items.value.push(item)
		},
		removeItem: async (id) => {
			await invoke('remove_item', { id })
			items.value = items.value.filter(i => i.id !== id)
		},
	})
}
```

Register it in `apps/app-frontend/src/providers/setup.ts`, which is called from `App.vue`'s `setup()`.

### Website Frontend (Nuxt)

Provide directly in `apps/frontend/src/app.vue`, using Nuxt's `useState()` where SSR hydration is needed:

```ts
provideMyFeature({
	items: useState<Item[]>('my-feature-items', () => []),
	addItem: async (item) => {
		await $fetch('/api/items', { method: 'POST', body: item })
	},
	removeItem: async (id) => {
		await $fetch(`/api/items/${id}`, { method: 'DELETE' })
	},
})
```

## Consuming Providers

In any component across `packages/ui`, `apps/frontend`, or `apps/app-frontend`:

```vue
<script setup lang="ts">
import { injectMyFeature } from '@modrinth/ui'

const { items, addItem } = injectMyFeature()
</script>

<template>
	<div v-for="item in items" :key="item.id">{{ item.name }}</div>
	<button @click="addItem({ id: '1', name: 'New' })">Add</button>
</template>
```

## When NOT to Use DI

Default to props and emits. DI adds indirection — only use it with a concrete reason.

- **Parent to direct child** — use props.
- **Data only exists in one frontend** — keep context local to that app, not in `packages/ui`.
- **Shallow prop drilling (1–2 levels)** — passing through one intermediate is fine.
- **Component-local state** — use `ref()` / `reactive()` locally.

## Existing Providers

| Provider | File | Purpose |
|----------|------|---------|
| `provideModrinthClient` | `providers/api-client.ts` | API client instance |
| `provideNotificationManager` | `providers/web-notifications.ts` | Notification management |
| `providePageContext` | `providers/page-context.ts` | Page config (sidebar, ads) |
| `provideProjectPageContext` | `providers/project-page.ts` | Project page state + mutations |
| `provideServerContext` | `providers/server-context.ts` | Server hosting state |
| `provideUserPageContext` | `providers/user-page.ts` | User page state |

## Key Files

- `packages/ui/src/providers/index.ts` — `createContext` factory + barrel exports
- `packages/ui/src/providers/*.ts` — Provider definitions
- `apps/frontend/src/app.vue` — Nuxt root provider setup
- `apps/app-frontend/src/App.vue` — Tauri root provider setup
- `apps/app-frontend/src/providers/setup/` — App provider setup functions
