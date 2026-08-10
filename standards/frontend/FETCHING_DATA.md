- [TanStack Query](#tanstack-query)
	- [Setup](#setup)
	- [Queries](#queries)
		- [Query-option factories](#query-option-factories)
		- [Conditional queries](#conditional-queries)
	- [Mutations](#mutations)
		- [Optimistic updates](#optimistic-updates)
	- [Query keys](#query-keys)
	- [Key files](#key-files)

# TanStack Query

TanStack Query (`@tanstack/vue-query` v5) manages server state. It supplies caching, background refetches, and cache invalidation.

Use TanStack Query for all data that comes from an API. Do not use a manual `ref()` and `await` pattern.

A TanStack MCP server is available. Use `tanstack_doc` or `tanstack_search_docs` when you need API details.

## Setup

`apps/frontend/src/plugins/tanstack.ts` configures TanStack Query as a Nuxt plugin. The plugin supports server-side rendering (SSR) hydration.

The default stale time is 5 seconds. Get the `QueryClient` with `useQueryClient()` or `useAppQueryClient()`.

`useAppQueryClient()` also operates in middleware.

## Queries

Use `useQuery` with `api-client` to get data:

```ts
const client = injectModrinthClient()

const { data, isPending, isError, error } = useQuery({
	queryKey: ['project', 'v3', projectId],
	queryFn: () => client.labrinth.projects_v3.get(projectId),
	staleTime: 1000 * 60 * 5,
})
```

Use the query state in templates:

```vue
<span v-if="isPending">Loading...</span>
<span v-else-if="isError">Error: {{ error.message }}</span>
<div v-else>{{ data.title }}</div>
```

### Query-Option Factories

For a query that multiple components use, define a query-option factory in `packages/ui/src/queries/`:

```ts
// composables/queries/project.ts
export const STALE_TIME = 1000 * 60 * 5
export const STALE_TIME_LONG = 1000 * 60 * 10

export const projectQueryOptions = {
	v3: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', 'v3', projectId] as const,
		queryFn: () => client.labrinth.projects_v3.get(projectId),
		staleTime: STALE_TIME,
	}),

	members: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', projectId, 'members'] as const,
		queryFn: () => client.labrinth.projects_v3.getMembers(projectId),
		staleTime: STALE_TIME,
	}),
}
```

Use the factory in each applicable component:

```ts
const { data } = useQuery(projectQueryOptions.v3(projectId, client))
```

### Conditional Queries

Use a computed `enabled` value when a query depends on other data:

```ts
const { data: members } = useQuery({
	queryKey: ['project', projectId, 'members'],
	queryFn: () => client.labrinth.projects_v3.getMembers(projectId),
	enabled: computed(() => !!projectId.value),
})
```

## Mutations

Use `useMutation` for create, update, and delete operations. Invalidate related queries after a successful operation:

```ts
const queryClient = useQueryClient()
const client = injectModrinthClient()

const createMutation = useMutation({
	mutationFn: (name: string) => client.archon.backups_v0.create(serverId, { name }),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: ['backups', 'list', serverId] }),
})
```

Use `createMutation.isPending.value` to disable buttons during submission.

### Optimistic Updates

Use an optimistic update and rollback when a mutation needs an immediate UI response:

```ts
const patchMutation = useMutation({
	mutationFn: async ({ projectId, data }) => {
		await client.labrinth.projects_v3.patch(projectId, data)
		return data
	},

	onMutate: async ({ projectId, data }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v3', projectId] })
		const previous = queryClient.getQueryData(['project', 'v3', projectId])

		queryClient.setQueryData(['project', 'v3', projectId], (old) => {
			if (!old) return old
			return { ...old, ...data }
		})

		return { previous }
	},

	onError: (_err, _variables, context) => {
		if (context?.previous) {
			queryClient.setQueryData(['project', 'v3', projectId], context.previous)
		}
	},

	onSettled: () => {
		queryClient.invalidateQueries({ queryKey: ['project', 'v3', projectId] })
	},
})
```

## Query Keys

Use hierarchical arrays for query keys:

```ts
// Resource type, version or qualifier, and ID.
['project', 'v3', projectId]

// Resource type, ID, and subresource.
['project', projectId, 'members']
['project', projectId, 'versions', 'v3']

// Domain, action, and ID.
['backups', 'list', serverId]
['tech-reviews']
```

Use `as const` for type safety. Put stable category segments before reactive parameters.

TanStack Query uses key prefixes during invalidation:

```ts
// Invalidate all v3 project queries.
queryClient.invalidateQueries({ queryKey: ['project', 'v3'] })
```

## Key Files

- `apps/frontend/src/plugins/tanstack.ts`: Contains the `QueryClient` setup and SSR hydration.
- `apps/frontend/src/composables/query-client.ts`: Contains the `useAppQueryClient()` helper.
- `apps/frontend/src/composables/queries/`: Contains reusable query-option factories.
