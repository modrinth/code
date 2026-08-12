- [Dependency injection](#dependency-injection)
	- [The `createContext` factory](#the-createcontext-factory)
	- [When to use DI](#when-to-use-di)
		- [Platform abstraction](#platform-abstraction)
		- [Page context](#page-context)
	- [Create a provider](#create-a-provider)
		- [1. Define the interface](#1-define-the-interface)
		- [2. Use an abstract class for complex logic](#2-use-an-abstract-class-for-complex-logic)
	- [Connect providers](#connect-providers)
		- [App frontend](#app-frontend)
		- [Website frontend](#website-frontend)
	- [Use providers](#use-providers)
	- [When not to use DI](#when-not-to-use-di)
	- [Existing providers](#existing-providers)
	- [Key files](#key-files)

# Dependency Injection

Modrinth uses a small dependency-injection (DI) layer that uses Vue `provide` and `inject`.

This layer shares platform capabilities and page state with common UI components.

## The `createContext` Factory

Define all providers with `createContext` from `packages/ui/src/providers/index.ts`. This factory comes from the Reka UI pattern.

The factory returns a typed `[inject, provide]` tuple:

```ts
import { createContext } from '@modrinth/ui'

interface MyContext {
	someValue: Ref<string>
	doSomething: () => void
}

export const [injectMyContext, provideMyContext] = createContext<MyContext>('MyComponent')
```

- Call `provideMyContext(value)` in the `setup()` function of a parent component.
- Call `injectMyContext()` in the `setup()` function of a descendant. It throws an error when no provider exists.
- Call `injectMyContext(null)` to return `null` when the context is optional.

## When to Use DI

Use DI in these conditions:

- The same interface needs different implementations on the website and the desktop app.
- Deep descendant components need the same page state, and props must pass through three or more levels.

### Platform Abstraction

Components in `packages/ui` can need capabilities that each frontend implements differently:

| Provider      | App frontend                       | Website frontend                |
| ------------- | ---------------------------------- | ------------------------------- |
| API client    | Tauri IPC client                   | REST fetch client               |
| Notifications | `ref()` state and window control   | `useState()` for SSR hydration  |
| File picker   | Native Tauri dialogs               | Browser file inputs             |
| Tags          | Tauri commands                     | Nuxt server state               |
| Page context  | Sidebar and advertisement hooks    | No sidebar and no advertisements |

### Page Context

Use DI to share page data with deep descendants. Examples include the project sidebar, header, and version components.

## Create a Provider

### 1. Define the Interface

Define the interface in `packages/ui/src/providers/`:

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

Export the provider from `packages/ui/src/providers/index.ts`.

### 2. Use an Abstract Class for Complex Logic

Use an abstract class when the provider has complex platform logic:

```ts
export abstract class AbstractMyFeatureManager {
	abstract items: Ref<Item[]>
	abstract addItem(item: Item): Promise<void>

	// Put common logic in the base class.
	handleError(err: unknown) {
		console.error(err)
	}
}

export const [injectMyFeature, provideMyFeature] =
	createContext<AbstractMyFeatureManager>('MyFeature')
```

Refer to `AbstractWebNotificationManager` in `packages/ui/src/providers/web-notifications.ts` for an example.

## Connect Providers

### App Frontend

Make a setup function in `apps/app-frontend/src/providers/setup/`:

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

Register the function in `apps/app-frontend/src/providers/setup.ts`. `App.vue` calls this setup file from its `setup()` function.

### Website Frontend

Provide the context in `apps/frontend/src/app.vue`. Use Nuxt `useState()` when the state needs SSR hydration:

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

## Use Providers

Inject the provider in a component in `packages/ui`, `apps/frontend`, or `apps/app-frontend`:

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

## When Not to Use DI

Use props and emits by default. DI adds an indirect layer, so use it only for a specific reason.

- Use props from a parent to its direct child.
- Keep data in one frontend when only that frontend uses it.
- Use props through one or two intermediate levels.
- Use `ref()` or `reactive()` for component state.

## Existing Providers

| Provider                     | File                             | Purpose                       |
| ---------------------------- | -------------------------------- | ----------------------------- |
| `provideModrinthClient`      | `providers/api-client.ts`        | Supplies the API client.      |
| `provideNotificationManager` | `providers/web-notifications.ts` | Manages notifications.        |
| `providePageContext`         | `providers/page-context.ts`      | Supplies page configuration.  |
| `provideProjectPageContext`  | `providers/project-page.ts`      | Manages project page state.   |
| `provideServerContext`       | `providers/server-context.ts`    | Manages server hosting state. |
| `provideUserPageContext`     | `providers/user-page.ts`         | Manages user page state.      |

## Key Files

- `packages/ui/src/providers/index.ts`: Contains the `createContext` factory and provider exports.
- `packages/ui/src/providers/*.ts`: Contains provider definitions.
- `apps/frontend/src/app.vue`: Contains the Nuxt root-provider setup.
- `apps/app-frontend/src/App.vue`: Contains the Tauri root-provider setup.
- `apps/app-frontend/src/providers/setup/`: Contains the app provider setup functions.
