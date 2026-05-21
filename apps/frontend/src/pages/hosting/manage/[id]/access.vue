<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	ServersManageAccessPage,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

const client = injectModrinthClient()
const { server, serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()
const ACTION_LOG_PAGE_SIZE = 200
const actionLogDateFilter = defaultActionLogDateFilter()

try {
	await Promise.all([
		queryClient.ensureQueryData({
			queryKey: ['servers', 'users', 'v1', serverId],
			queryFn: () => client.archon.server_users_v1.list(serverId),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['servers', 'v1', 'detail', serverId],
			queryFn: () => client.archon.servers_v1.get(serverId),
			staleTime: 30_000,
		}),
		queryClient.prefetchInfiniteQuery({
			queryKey: [
				'servers',
				'action-log',
				'v1',
				'infinite',
				serverId,
				null,
				actionLogDateFilter.min_datetime,
				actionLogDateFilter.max_datetime,
			],
			queryFn: ({ pageParam = 0 }) => {
				const offset = typeof pageParam === 'number' ? pageParam : 0
				return client.archon.actions_v1.list(serverId, {
					limit: ACTION_LOG_PAGE_SIZE,
					offset,
					order: 'desc',
					...actionLogDateFilter,
				})
			},
			getNextPageParam: (lastPage) =>
				typeof lastPage.next_offset === 'number' ? lastPage.next_offset : undefined,
			initialPageParam: 0,
			staleTime: 30_000,
		}),
	])
} catch {
	// Let mounted layouts' useQuery surface errors; do not fail route setup.
}

useHead({
	title: computed(() => `Access - ${server.value?.name ?? 'Server'} - Modrinth`),
})

function defaultActionLogDateFilter() {
	const endDate = new Date()
	const startDate = new Date(endDate)
	startDate.setDate(startDate.getDate() - 29)

	return {
		min_datetime: startOfDay(startDate).toISOString(),
		max_datetime: endOfDay(endDate).toISOString(),
	}
}

function startOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate())
}

function endOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 23, 59, 59, 999)
}
</script>

<template>
	<ServersManageAccessPage />
</template>
