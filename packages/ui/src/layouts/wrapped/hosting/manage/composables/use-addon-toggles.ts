import type { AbstractModrinthClient, Archon } from '@modrinth/api-client'
import { type QueryClient, useMutation } from '@tanstack/vue-query'
import { onUnmounted, type Ref } from 'vue'

import {
	type AddonToggleChanges,
	addonToggleKey,
	applyPendingAddonToggle,
	clearPendingAddonToggle,
	discardFailedAddonToggle,
	markPendingAddonToggleSent,
	queuePendingAddonToggle,
} from '#ui/composables/server-content-toggle-state'
import { isPlayerOnlyContent } from '#ui/layouts/shared/content-tab/utils/addon-environment'

const FLUSH_DELAY_MS = 250

export type AddonToggleBatch = {
	addon: Archon.Content.v1.Addon
	worldId: string
	queryKey: readonly string[]
	changes: AddonToggleChanges
}

type AddonToggleQueue = {
	pending?: AddonToggleBatch
	timer?: ReturnType<typeof setTimeout>
	running: boolean
}

/**
 * Debounces enable/disable and side toggles per addon, applies them optimistically to the
 * content query, and sends each addon's batched changes in order.
 */
export function useAddonToggles(options: {
	client: AbstractModrinthClient
	queryClient: QueryClient
	serverId: string
	worldId: Ref<string | null>
	queryKeyFor: (addon: Archon.Content.v1.Addon) => readonly string[]
	onError: (error: unknown, batch: AddonToggleBatch) => void
}) {
	const { client, queryClient, serverId, worldId } = options
	const queues = new Map<string, AddonToggleQueue>()

	function cancel(addon: Archon.Content.v1.Addon) {
		if (worldId.value) clearPendingAddonToggle(queryClient, serverId, worldId.value, addon)
		const key = `${worldId.value}:${addonToggleKey(addon)}`
		const queue = queues.get(key)
		if (!queue) return
		if (queue.timer) clearTimeout(queue.timer)
		queue.timer = undefined
		queue.pending = undefined
		if (!queue.running) queues.delete(key)
	}

	const mutation = useMutation({
		mutationFn: async ({ addon, worldId: targetWorldId, changes }: AddonToggleBatch) => {
			const request = { filename: addon.filename, kind: addon.kind }
			if (changes.enabled !== undefined) {
				if (changes.enabled) {
					await client.archon.content_v1.enableAddon(serverId, targetWorldId, request)
				} else {
					await client.archon.content_v1.disableAddon(serverId, targetWorldId, request)
				}
			}
			const sides = (['server', 'player'] as const).filter((side) => changes[side] !== undefined)
			if (sides.length === 0) return
			if (!isPlayerOnlyContent(addon) && !addon.side_toggle_unlocked) {
				await client.archon.content_v1.setAddonSideToggleLocked(serverId, targetWorldId, {
					...request,
					locked: false,
				})
			}
			for (const side of sides) {
				const sideRequest = { ...request, enabled: changes[side]! }
				if (side === 'server') {
					await client.archon.content_v1.setAddonEnabledServer(serverId, targetWorldId, sideRequest)
				} else {
					await client.archon.content_v1.setAddonEnabledPlayer(serverId, targetWorldId, sideRequest)
				}
			}
		},
		onError: (error, batch) => options.onError(error, batch),
	})

	async function flush(key: string, queue: AddonToggleQueue) {
		if (queue.running || !queue.pending) return
		const batch = queue.pending
		queue.pending = undefined
		queue.running = true
		markPendingAddonToggleSent(queryClient, serverId, batch.worldId, batch.addon, batch.changes)
		try {
			await mutation.mutateAsync(batch)
		} catch {
			discardFailedAddonToggle(queryClient, serverId, batch.worldId, batch.addon, batch.changes)
			void queryClient.invalidateQueries({ queryKey: batch.queryKey, exact: true })
		} finally {
			queue.running = false
			if (!queue.pending) {
				queues.delete(key)
			} else if (!queue.timer) {
				void flush(key, queue)
			}
		}
	}

	function queue(addon: Archon.Content.v1.Addon, changes: AddonToggleChanges) {
		const targetWorldId = worldId.value
		if (!targetWorldId) return
		const key = `${targetWorldId}:${addonToggleKey(addon)}`
		const entry: AddonToggleQueue = queues.get(key) ?? { running: false }
		queues.set(key, entry)
		const targetQueryKey = options.queryKeyFor(addon)
		void queryClient.cancelQueries({ queryKey: targetQueryKey, exact: true })
		const batch: AddonToggleBatch = entry.pending ?? {
			addon,
			worldId: targetWorldId,
			queryKey: targetQueryKey,
			changes: {},
		}
		batch.changes = { ...batch.changes, ...changes }
		entry.pending = batch
		queuePendingAddonToggle(queryClient, serverId, targetWorldId, addon, changes)
		queryClient.setQueryData<Archon.Content.v1.Addons>(targetQueryKey, (current) =>
			current
				? {
						...current,
						addons: (current.addons ?? []).map((candidate) =>
							addonToggleKey(candidate) === addonToggleKey(addon)
								? applyPendingAddonToggle(queryClient, serverId, targetWorldId, candidate)
								: candidate,
						),
					}
				: current,
		)
		if (entry.timer) clearTimeout(entry.timer)
		entry.timer = setTimeout(() => {
			entry.timer = undefined
			void flush(key, entry)
		}, FLUSH_DELAY_MS)
	}

	onUnmounted(() => {
		for (const [key, entry] of queues) {
			if (entry.timer) clearTimeout(entry.timer)
			entry.timer = undefined
			void flush(key, entry)
		}
	})

	return { queueAddonToggle: queue, cancelQueuedAddonToggle: cancel }
}
