import { type QueryClient, queryOptions, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, onScopeDispose, toValue } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { get_all } from '@/helpers/process'
import type { AppEvents } from '@/providers/app-events'

export interface ProcessMetadata {
	uuid: string
	instance_id: string
	instance_path: string
	instance_name: string
	start_time: string
}

const runningProcessesKey = ['instances', 'processes'] as const
const refreshers = new WeakMap<QueryClient, () => Promise<void>>()
const refreshGenerations = new WeakMap<QueryClient, number>()
const activeQueryClients = new Set<QueryClient>()

function runningProcessesQueryOptions(queryClient: QueryClient) {
	return queryOptions({
		queryKey: runningProcessesKey,
		queryFn: async (): Promise<ProcessMetadata[]> => {
			while (true) {
				const generation = refreshGenerations.get(queryClient) ?? 0
				try {
					const processes: ProcessMetadata[] = await get_all()
					if (generation === (refreshGenerations.get(queryClient) ?? 0)) return processes
				} catch (error) {
					if (generation === (refreshGenerations.get(queryClient) ?? 0)) throw error
				}
			}
		},
		networkMode: 'always',
		staleTime: Infinity,
		gcTime: Infinity,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
		retry: false,
	})
}

function getRefresher(queryClient: QueryClient) {
	const existing = refreshers.get(queryClient)
	if (existing) return existing

	let dirty = false
	let pending: Promise<void> | undefined
	function refresh(): Promise<void> {
		refreshGenerations.set(queryClient, (refreshGenerations.get(queryClient) ?? 0) + 1)
		dirty = true
		if (!pending) {
			pending = Promise.resolve().then(async () => {
				try {
					do {
						dirty = false
						const joinedExistingRequest =
							queryClient.isFetching({ queryKey: runningProcessesKey, exact: true }) > 0
						try {
							await queryClient.fetchQuery({
								...runningProcessesQueryOptions(queryClient),
								staleTime: 0,
							})
						} catch (error) {
							if (!dirty) throw error
						}
						if (joinedExistingRequest) dirty = true
					} while (dirty)
				} finally {
					pending = undefined
				}
			})
		}
		return pending
	}

	refreshers.set(queryClient, refresh)
	return refresh
}

export async function refreshRunningProcesses(): Promise<void> {
	await Promise.all([...activeQueryClients].map((queryClient) => getRefresher(queryClient)()))
}

export function useRunningProcessUpdates(events: AppEvents) {
	const queryClient = useQueryClient()
	const refresh = getRefresher(queryClient)
	activeQueryClients.add(queryClient)
	onScopeDispose(() => activeQueryClients.delete(queryClient))
	useAppEvent(
		'process',
		(event) => {
			if (event.event === 'launched' || event.event === 'finished') return refresh()
		},
		events,
	)
	return { refresh }
}

export function useRunningProcesses() {
	const queryClient = useQueryClient()
	const query = useQuery(runningProcessesQueryOptions(queryClient))
	const processes = computed(() => query.data.value ?? [])
	return { processes, refresh: getRefresher(queryClient) }
}

export function useInstanceRunningProcesses(
	instanceId: MaybeRefOrGetter<string | null | undefined>,
) {
	const { processes: allProcesses, refresh } = useRunningProcesses()
	const processes = computed(() =>
		allProcesses.value.filter((process) => process.instance_id === toValue(instanceId)),
	)
	const processCount = computed(() => processes.value.length)
	const playing = computed(() => processCount.value > 0)
	return { processes, processCount, playing, refresh }
}
