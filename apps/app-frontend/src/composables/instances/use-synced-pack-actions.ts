import { LinkIcon, XIcon } from '@modrinth/assets'
import {
	type ButtonMenuOption,
	type ContentItem,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { type Ref, watch } from 'vue'

import type SyncedContentModal from '@/components/ui/instance/SyncedContentModal.vue'
import { get_global_synced_options } from '@/helpers/instance'
import {
	desync_pack,
	remove_synced_pack,
	sync_pack,
	type SyncedPackAction,
	syncedPackKeys,
} from '@/helpers/synced-packs'
import type { GameInstance } from '@/helpers/types'
import { instanceKeys } from '@/pages/instance/query-options'

export function useSyncedPackActions(
	instance: Ref<GameInstance>,
	modal: Ref<InstanceType<typeof SyncedContentModal> | undefined>,
	canMutate: (item: ContentItem) => boolean,
	refresh: () => Promise<unknown>,
) {
	const { formatMessage } = useVIntl()
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const globalOptions = useQuery({
		queryKey: ['global-synced-options'],
		queryFn: get_global_synced_options,
	})
	const deleteEverywhere = new Set<string>()
	watch(
		[
			() => globalOptions.data.value?.resource_packs,
			() => globalOptions.data.value?.data_packs,
			() => instance.value.synced_options.resource_packs,
			() => instance.value.synced_options.data_packs,
		],
		() => refresh().catch(handleError),
	)
	const messages = defineMessages({
		sync: { id: 'app.content.sync-pack', defaultMessage: 'Sync' },
		desync: { id: 'app.content.desync-pack', defaultMessage: 'Desync' },
		resourceDisabled: {
			id: 'app.content.sync-resource-packs-disabled',
			defaultMessage: 'Turn on resource pack syncing in app settings first.',
		},
		dataDisabled: {
			id: 'app.content.sync-data-packs-disabled',
			defaultMessage: 'Turn on data pack syncing in app settings first.',
		},
		override: {
			id: 'app.content.sync-packs-override',
			defaultMessage: 'Turn off the pack syncing override in this instance’s settings first.',
		},
	})

	const mutation = useMutation({
		mutationFn: (operation: () => Promise<void>) => operation(),
		onError: (error) => handleError(error),
		onSettled: async () => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: syncedPackKeys.all }),
				queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
			])
			await refresh()
		},
	})

	function sync(item: ContentItem) {
		if (!item.file_path || mutation.isPending.value) return
		const instanceId = instance.value.id
		const projectPath = item.file_path
		mutation.mutate(() => sync_pack(instanceId, projectPath))
	}

	async function desync(item: ContentItem) {
		if (!item.synced_pack) return
		const mode = await modal.value?.confirmDesync(item)
		if (!mode) return
		mutation.mutate(() => desync_pack(instance.value.id, item.synced_pack!.id, mode))
	}

	function overflowOptions(item: ContentItem): ButtonMenuOption[] {
		if (
			!['resourcepack', 'datapack'].includes(item.project_type) ||
			!canMutate(item) ||
			(item.source_kind && item.source_kind !== 'local')
		)
			return []
		const option = item.project_type === 'resourcepack' ? 'resource_packs' : 'data_packs'
		const disabledReason = !globalOptions.data.value?.[option]
			? formatMessage(
					option === 'resource_packs' ? messages.resourceDisabled : messages.dataDisabled,
				)
			: !instance.value.synced_options[option]
				? formatMessage(messages.override)
				: undefined
		return [
			{
				id: item.synced_pack ? 'desync-pack' : 'sync-pack',
				label: formatMessage(item.synced_pack ? messages.desync : messages.sync),
				icon: item.synced_pack ? XIcon : LinkIcon,
				disabled: mutation.isPending.value || !!disabledReason,
				tooltip: disabledReason,
				action: () => (item.synced_pack ? desync(item) : sync(item)),
			},
		]
	}

	async function confirmAction(action: SyncedPackAction, items: ContentItem[]) {
		if (!items.some((item) => item.synced_pack)) return true
		return (await modal.value?.confirmChange(action, items)) ?? false
	}

	async function confirmDeleteItems(items: ContentItem[]) {
		deleteEverywhere.clear()
		if (!items.some((item) => item.synced_pack)) return undefined
		const choice = await modal.value?.confirmDelete(items)
		if (!choice) return false
		if (choice === 'all') {
			for (const item of items) if (item.synced_pack) deleteEverywhere.add(item.synced_pack.id)
		}
		return true
	}

	async function deleteSyncedItem(item: ContentItem) {
		if (!item.synced_pack) return false
		if (deleteEverywhere.has(item.synced_pack.id)) {
			await remove_synced_pack(item.synced_pack.id)
			return true
		}
		await desync_pack(instance.value.id, item.synced_pack.id, 'keep_in_other_instances')
		return false
	}

	return {
		overflowOptions,
		confirmAction,
		confirmDeleteItems,
		deleteSyncedItem,
		isPending: mutation.isPending,
	}
}
