<script setup lang="ts">
import { TrashIcon } from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	commonMessages,
	type ContentItem,
	defineMessages,
	injectNotificationManager,
	ManagedContentModal,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref, watch } from 'vue'

import SyncedContentModal from '@/components/ui/instance/SyncedContentModal.vue'
import { useAppEvent } from '@/composables/use-app-event'
import {
	remove_synced_pack,
	set_synced_pack_enabled,
	syncedPackKeys,
	syncedPackQueryOptions,
	type SyncedPackType,
} from '@/helpers/synced-packs'
import { instanceKeys } from '@/pages/instance/query-options'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const manager = ref<InstanceType<typeof ManagedContentModal>>()
const confirmation = ref<InstanceType<typeof SyncedContentModal>>()
const projectType = ref<SyncedPackType>('resourcepack')
const isOpen = ref(false)

const messages = defineMessages({
	resourcePacks: {
		id: 'app.synced-packs.resource-packs.title',
		defaultMessage: 'Synced resource packs',
	},
	dataPacks: { id: 'app.synced-packs.data-packs.title', defaultMessage: 'Synced data packs' },
	empty: {
		id: 'app.synced-packs.empty',
		defaultMessage: 'Add packs from an instance’s content page to start syncing.',
	},
	loadFailed: {
		id: 'app.synced-packs.load-failed',
		defaultMessage: 'Packs could not be loaded. Try refreshing.',
	},
})

const title = computed(() =>
	formatMessage(projectType.value === 'resourcepack' ? messages.resourcePacks : messages.dataPacks),
)
const packsQuery = useQuery(
	computed(() => ({
		...syncedPackQueryOptions(projectType.value),
		enabled: isOpen.value,
	})),
)
const mutation = useMutation({
	mutationFn: (operation: () => Promise<void>) => operation(),
	onError: (error) => handleError(error),
	onSettled: async () => {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: syncedPackKeys.all }),
			queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
		])
	},
})

watch([packsQuery.data, packsQuery.isPending], ([items, pending]) => {
	if (isOpen.value && !pending) manager.value?.setItems(items ?? [])
})
watch(packsQuery.error, (error) => {
	if (error) handleError(error)
})
useAppEvent('instance', (event) => {
	if (isOpen.value && event.event === 'synced') {
		void queryClient.invalidateQueries({ queryKey: syncedPackKeys.all })
	}
})

async function show(type: SyncedPackType) {
	projectType.value = type
	isOpen.value = true
	await nextTick()
	if (packsQuery.data.value) manager.value?.show(packsQuery.data.value)
	else manager.value?.showLoading()
}

async function togglePacks(items: ContentItem[], enabled: boolean) {
	const changing = items.filter((item) => item.enabled !== enabled)
	if (changing.length === 0 || mutation.isPending.value) return
	if (!(await confirmation.value?.confirmChange(enabled ? 'enable' : 'disable', changing))) return
	mutation.mutate(async () => {
		for (const item of changing) await set_synced_pack_enabled(item.id, enabled)
	})
}

async function deletePack(item: ContentItem) {
	if ((await confirmation.value?.confirmDelete([item])) !== 'all') return
	mutation.mutate(() => remove_synced_pack(item.id))
}

function overflowOptions(item: ContentItem): ButtonMenuOption[] {
	return [
		{
			id: 'delete-synced-pack',
			label: formatMessage(commonMessages.deleteLabel),
			icon: TrashIcon,
			tone: 'red',
			action: () => deletePack(item),
		},
	]
}

defineExpose({ show })
</script>

<template>
	<SyncedContentModal ref="confirmation" />
	<ManagedContentModal
		ref="manager"
		:header="title"
		:empty-description="
			formatMessage(packsQuery.isError.value ? messages.loadFailed : messages.empty)
		"
		:show-version="false"
		filter-mode="status"
		enable-toggle
		:action-disabled="mutation.isPending.value"
		:get-overflow-options="overflowOptions"
		@update:enabled="(item, enabled) => togglePacks([item], enabled)"
		@bulk:enable="(items) => togglePacks(items, true)"
		@bulk:disable="(items) => togglePacks(items, false)"
		@hide="isOpen = false"
	>
		<template v-if="packsQuery.isError.value" #toolbar>
			<Button @click="packsQuery.refetch()">{{
				formatMessage(commonMessages.refreshButton)
			}}</Button>
		</template>
	</ManagedContentModal>
</template>
