import type { Archon, Labrinth } from '@modrinth/api-client'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { type Ref, ref } from 'vue'

import type { UpdateAllSelection } from '#ui/components/modal/update-all-modal/update-all-modal-types'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	type AddonToggleChanges,
	addonToggleKey,
} from '#ui/composables/server-content-toggle-state'
import { useServerPreferences } from '#ui/composables/server-preferences'
import type ConfirmEnvironmentModal from '#ui/layouts/shared/content-tab/components/modals/ConfirmEnvironmentModal.vue'
import type { ContentItem, ContentSide } from '#ui/layouts/shared/content-tab/types'
import {
	friendlyAddonName,
	getAddonEnvironment,
	isEnabledForAnySide,
	isIncompatibleEnvironment,
	isPlayerOnlyContent,
	isRequiredEnvironment,
} from '#ui/layouts/shared/content-tab/utils/addon-environment'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

import { useAddonToggles } from './use-addon-toggles'
import type { ContentQueries } from './use-content-queries'

const PERFORMANCE_WARNING_PROJECT_ID = 'uCdwusMi'

const messages = defineMessages({
	failedToRemoveContent: {
		id: 'hosting.content.failed-to-remove',
		defaultMessage: 'Failed to remove content',
	},
	failedToBulkDelete: {
		id: 'hosting.content.failed-to-bulk-delete',
		defaultMessage: 'Failed to delete content',
	},
	failedToToggle: {
		id: 'hosting.content.failed-to-toggle',
		defaultMessage: 'Failed to enable or disable {name}',
	},
	failedToSetEnabledFor: {
		id: 'hosting.content.failed-to-set-enabled-for',
		defaultMessage: 'Failed to change where {name} is enabled',
	},
	failedToUpdate: {
		id: 'hosting.content.failed-to-update',
		defaultMessage: 'Failed to update',
	},
	failedToBulkUpdate: {
		id: 'hosting.content.failed-to-bulk-update',
		defaultMessage: 'Failed to update content',
	},
})

export interface ContentMutationsOptions {
	serverId: string
	queries: ContentQueries
	items: Readonly<Ref<ContentItem[]>>
	actionDisabled: Readonly<Ref<boolean>>
	migrationRequired: Readonly<Ref<boolean>>
}

export function useContentMutations({
	serverId,
	queries,
	items,
	actionDisabled,
	migrationRequired,
}: ContentMutationsOptions) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { addNotification } = injectNotificationManager()
	const { worldId } = injectModrinthServerContext()
	const userPreferences = useServerPreferences(serverId)
	const { getAddonForItem, getAddonQueryKey, contentProjectsById, optimisticallyUpdateAddons } =
		queries

	const environmentWarningModal = ref<InstanceType<typeof ConfirmEnvironmentModal>>()

	function notifyError(title: string, error: unknown) {
		addNotification({
			type: 'error',
			title,
			text: error instanceof Error ? error.message : undefined,
		})
	}

	const { queueAddonToggle, cancelQueuedAddonToggle } = useAddonToggles({
		client,
		queryClient,
		serverId,
		worldId,
		queryKeyFor: getAddonQueryKey,
		onError: (error, batch) =>
			notifyError(
				formatMessage(
					batch.changes.enabled !== undefined
						? messages.failedToToggle
						: messages.failedToSetEnabledFor,
					{ name: friendlyAddonName(batch.addon) },
				),
				error,
			),
	})

	function isRequiredForSide(addon: Archon.Content.v1.Addon, side: ContentSide) {
		const project = addon.project_id ? contentProjectsById.value.get(addon.project_id) : undefined
		return isRequiredEnvironment(addon, side, project)
	}

	async function setEnabledFor(item: ContentItem, side: ContentSide, enabled: boolean) {
		if (actionDisabled.value || migrationRequired.value) return
		const addon = getAddonForItem(item)
		if (!addon || (side === 'server' && isPlayerOnlyContent(addon))) return
		const targetWorldId = worldId.value
		const showPerformanceWarning =
			enabled && side === 'server' && addon.project_id === PERFORMANCE_WARNING_PROJECT_ID
		if (
			userPreferences.value.warnOnIncompatibleContent &&
			(showPerformanceWarning ||
				(enabled
					? !(side === 'server' && addon.pack_client_depends) &&
						isIncompatibleEnvironment(addon, side)
					: isRequiredForSide(addon, side)))
		) {
			const confirmed = await environmentWarningModal.value?.show(
				item.project?.title ?? item.file_name,
				side,
				enabled,
				getAddonEnvironment(addon) === 'singleplayer_only',
				showPerformanceWarning,
			)
			if (!confirmed || actionDisabled.value || worldId.value !== targetWorldId) return
		}
		queueAddonToggle(addon, { [side]: enabled })
	}

	function toggleChanges(addon: Archon.Content.v1.Addon, enabled: boolean): AddonToggleChanges {
		if (enabled && !addon.disabled && !isEnabledForAnySide(addon) && !migrationRequired.value) {
			const sides = (['server', 'player'] as const).filter(
				(side) =>
					!(side === 'server' && isPlayerOnlyContent(addon)) &&
					!isIncompatibleEnvironment(addon, side),
			)
			const changes: AddonToggleChanges = {}
			for (const side of sides.length > 0 ? sides : (['player'] as const)) changes[side] = true
			return changes
		}
		return { enabled }
	}

	async function setEnabled(targets: ContentItem[], enabled: boolean) {
		if (actionDisabled.value) return
		for (const item of targets) {
			const addon = getAddonForItem(item)
			if (addon) queueAddonToggle(addon, toggleChanges(addon, enabled))
		}
	}

	function withoutAddons(requests: Archon.Content.v1.RemoveAddonRequest[]) {
		return (addons: Archon.Content.v1.Addon[]) =>
			addons.filter(
				(addon) =>
					!requests.some(
						(request) => request.filename === addon.filename && request.kind === addon.kind,
					),
			)
	}

	const deleteMutation = useMutation({
		mutationFn: (request: Archon.Content.v1.RemoveAddonRequest) =>
			client.archon.content_v1.deleteAddon(serverId, worldId.value!, request),
		onMutate: (request) => optimisticallyUpdateAddons(withoutAddons([request])),
		onError: (err, _request, rollback) => {
			rollback?.()
			notifyError(formatMessage(messages.failedToRemoveContent), err)
		},
	})

	const bulkDeleteMutation = useMutation({
		mutationFn: (requests: Archon.Content.v1.RemoveAddonRequest[]) =>
			client.archon.content_v1.deleteAddons(serverId, worldId.value!, requests),
		onMutate: (requests) => optimisticallyUpdateAddons(withoutAddons(requests)),
		onError: (err, _requests, rollback) => {
			rollback?.()
			notifyError(formatMessage(messages.failedToBulkDelete), err)
		},
	})

	async function remove(targets: ContentItem[]) {
		if (actionDisabled.value) return
		const addons = targets.flatMap((item) => {
			const addon = getAddonForItem(item)
			return addon && (targets.length === 1 || !item.installing) ? [addon] : []
		})
		if (addons.length === 0) return
		for (const addon of addons) cancelQueuedAddonToggle(addon)
		const requests = addons.map((addon) => ({ filename: addon.filename, kind: addon.kind }))
		if (targets.length === 1) {
			await deleteMutation.mutateAsync(requests[0]).catch(() => undefined)
		} else {
			await bulkDeleteMutation.mutateAsync(requests).catch(() => undefined)
		}
	}

	function markInstalling(ids: Set<string>) {
		return optimisticallyUpdateAddons((addons) =>
			addons.map((addon) =>
				ids.has(addonToggleKey(addon)) ? { ...addon, installing: true } : addon,
			),
		)
	}

	const bulkUpdateMutation = useMutation({
		mutationFn: ({
			addons,
		}: {
			addons: Array<{ filename: string; version_id: string }>
			addonIds: Set<string>
		}) => client.archon.content_v1.updateAddons(serverId, worldId.value!, addons),
		onMutate: ({ addonIds }) => markInstalling(addonIds),
		onError: (err, _variables, rollback) => {
			rollback?.()
			notifyError(formatMessage(messages.failedToBulkUpdate), err)
		},
	})

	async function applyUpdates(selections: UpdateAllSelection[]) {
		if (actionDisabled.value) return
		const addonIds = new Set<string>()
		const addons = selections.flatMap((selection) => {
			const item = items.value.find((candidate) => candidate.id === selection.id)
			if (
				!item?.has_update ||
				item.locked ||
				item.installing ||
				item.project?.id !== selection.projectId
			)
				return []
			addonIds.add(item.id)
			return [{ filename: item.file_name, version_id: selection.version.id }]
		})
		if (addons.length === 0) return
		await bulkUpdateMutation.mutateAsync({ addons, addonIds })
	}

	async function installVersion(item: ContentItem, version: Labrinth.Versions.v2.Version) {
		if (actionDisabled.value) return
		const addon = getAddonForItem(item)
		if (!addon) return
		const rollback = await markInstalling(new Set([item.id]))
		try {
			await client.archon.content_v1.updateAddon(serverId, worldId.value!, {
				filename: addon.filename,
				version_id: version.id,
			})
		} catch (err) {
			rollback()
			notifyError(formatMessage(messages.failedToUpdate), err)
		}
	}

	return {
		environmentWarningModal,
		setEnabled,
		setEnabledFor,
		remove,
		applyUpdates,
		installVersion,
	}
}
