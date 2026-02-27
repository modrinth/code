# TanStack Query

TanStack Query (`@tanstack/vue-query` v5) is used for server state management — caching, background refetching, and cache invalidation. Use it instead of manual `ref()` + `await` patterns for any data that comes from an API.

A TanStack MCP server is available — use `tanstack_doc` and `tanstack_search_docs` tools to look up API details when needed.

## Setup

TanStack Query is configured in `apps/frontend/src/plugins/tanstack.ts` as a Nuxt plugin with SSR hydration support. Default stale time is 5 seconds. The `QueryClient` is available via `useQueryClient()` or `useAppQueryClient()` (which also works in middleware).

## Queries

Use `useQuery` with the api-client for data fetching:

```ts
const client = injectModrinthClient()

const { data, isPending, isError, error } = useQuery({
	queryKey: ['project', 'v3', projectId],
	queryFn: () => client.labrinth.projects_v3.get(projectId),
	staleTime: 1000 * 60 * 5,
})
```

In templates:

```vue
<span v-if="isPending">Loading...</span>
<span v-else-if="isError">Error: {{ error.message }}</span>
<div v-else>{{ data.title }}</div>
```

### Query Option Factories

For queries used across multiple components, define reusable query option factories in `packages/ui/src/queries/`:

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

Then use them:

```ts
const { data } = useQuery(projectQueryOptions.v3(projectId, client))
```

### Conditional Queries

Use `enabled` as a computed for queries that depend on other data:

```ts
const { data: members } = useQuery({
	queryKey: ['project', projectId, 'members'],
	queryFn: () => client.labrinth.projects_v3.getMembers(projectId),
	enabled: computed(() => !!projectId.value),
})
```

## Mutations

Use `useMutation` for create/update/delete operations. Invalidate related queries on success:

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

For mutations where responsiveness matters, use optimistic updates with rollback:

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

Keys use a hierarchical array pattern:

```ts
// Resource type → version/qualifier → ID
['project', 'v3', projectId]

// Resource type → ID → sub-resource
['project', projectId, 'members']
['project', projectId, 'versions', 'v3']

// Domain → action → ID
['backups', 'list', serverId]
['tech-reviews']
```

Use `as const` for type safety. Put the resource ID last when possible — this makes partial key matching work for invalidation:

```ts
// Invalidates all project queries for this ID
queryClient.invalidateQueries({ queryKey: ['project', projectId] })
```

## Key Files

- `apps/frontend/src/plugins/tanstack.ts` — QueryClient setup + SSR hydration
- `apps/frontend/src/composables/query-client.ts` — `useAppQueryClient()` helper
- `apps/frontend/src/composables/queries/` — reusable query option factories
