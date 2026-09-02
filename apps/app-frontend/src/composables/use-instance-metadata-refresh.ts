import { useQueryClient } from '@tanstack/vue-query'

import type { InstancePayload } from '@/generated/app-events/InstancePayload'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'
import type { AppEvents } from '@/providers/app-events'

import { useAppEvent } from './use-app-event'

const INSTANCE_METADATA_EVENTS = new Set<InstancePayload['event']>([
	'created',
	'synced',
	'edited',
	'removed',
])

export function useInstanceMetadataRefresh(events: AppEvents) {
	const queryClient = useQueryClient()
	let refreshQueued = false
	let refreshPromise: Promise<void> | undefined

	function queueRefresh() {
		refreshQueued = true
		if (!refreshPromise) {
			refreshPromise = Promise.resolve().then(async () => {
				try {
					do {
						refreshQueued = false
						const joinedExistingRequest =
							queryClient.isFetching({ queryKey: instanceKeys.list(), exact: true }) > 0
						const instances = await queryClient.fetchQuery({
							...instanceListQueryOptions(),
							staleTime: 0,
						})

						for (const instance of instances) {
							queryClient.setQueryData(instanceKeys.detail(instance.id), instance)
						}

						if (joinedExistingRequest) {
							refreshQueued = true
						}
					} while (refreshQueued)
				} finally {
					refreshPromise = undefined
				}
			})
		}

		return refreshPromise
	}

	useAppEvent(
		'instance',
		(event) => {
			if (INSTANCE_METADATA_EVENTS.has(event.event)) return queueRefresh()
		},
		events,
	)
	useAppEvent('instance_groups_changed', queueRefresh, events)
}
