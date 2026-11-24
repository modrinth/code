<template>
	<FloatingActionBar
		:shown="selectedInstanceCount > 0"
		:aria-label="formatMessage(messages.ariaLabel)"
		hide-when-modal-open
	>
		<div class="flex items-center gap-0.5">
			<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">
				{{ formatMessage(messages.selectedCount, { count: selectedLibraryInstances.size }) }}
			</span>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<Button type="quiet" :disabled="busy" @click="clearLibraryInstanceSelection">
				<span class="bar-label">{{ formatMessage(commonMessages.clearButton) }}</span>
			</Button>
		</div>
		<div class="ml-auto flex items-center gap-0.5">
			<Button
				v-if="displayState.group === 'Group' && !hasUninstalledSelection"
				type="quiet"
				:disabled="busy"
				:loading="creatingGroup"
				@click="createGroupFromSelection"
			>
				<SquarePlusIcon aria-hidden="true" />
				<span class="bar-label">{{ formatMessage(messages.newGroup) }}</span>
			</Button>
			<Button
				v-if="selectedGroupedInstances.length > 0 && !hasUninstalledSelection"
				type="quiet"
				:disabled="busy"
				:loading="removingFromGroup"
				@click="removeSelectedInstancesFromGroups"
			>
				<MinusIcon aria-hidden="true" />
				<span class="bar-label">{{ formatMessage(messages.removeFromGroup) }}</span>
			</Button>
			<template v-if="!hasUninstalledSelection">
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<Button
					v-tooltip="deleting ? formatMessage(messages.deleting) : undefined"
					type="quiet"
					color="red"
					interaction="filled"
					:disabled="busy"
					:loading="deleting"
					@click="confirmDeleteModal?.show()"
				>
					<TrashIcon aria-hidden="true" />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</Button>
			</template>
			<Button
				v-else
				v-tooltip="removingSharedInstances ? formatMessage(messages.removing) : undefined"
				type="quiet"
				color="orange"
				interaction="filled"
				:disabled="busy"
				:loading="removingSharedInstances"
				@click="requestRemoveSharedInstances(selectedUninstalledInstances, $event)"
			>
				<LogOutIcon v-if="removalMode === 'leave'" aria-hidden="true" />
				<UnlinkIcon v-else aria-hidden="true" />
				<span class="bar-label">{{
					formatMessage(sharedInstanceRemovalButtonMessages[removalMode])
				}}</span>
			</Button>
		</div>
	</FloatingActionBar>
	<ConfirmDeleteInstanceModal
		ref="confirmDeleteModal"
		:instances="selectedInstances"
		@delete="deleteSelectedInstances"
	/>
</template>

<script setup lang="ts">
import { LogOutIcon, MinusIcon, SquarePlusIcon, TrashIcon, UnlinkIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	FloatingActionBar,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { instanceKeys } from '@/features/instances/queries'
import { isUninstalledSharedInstance } from '@/features/library/library-instance'
import { injectSharedInstanceActions } from '@/features/library/shared-instances/actions-context'
import {
	getSharedInstanceRemovalMode,
	sharedInstanceRemovalButtonMessages,
} from '@/features/library/shared-instances/removal'
import { toError } from '@/helpers/errors'
import { remove } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import { injectLibraryGroups, injectLibrarySelection, injectLibraryView } from '../library-context'
import { getLibraryInstanceSelectionKey } from '../types'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const { instances, libraryInstances, displayState } = injectLibraryView()
const {
	selectedLibraryInstances,
	selectedInstanceIds,
	clearLibraryInstanceSelection,
	setSelectedLibraryInstances,
} = injectLibrarySelection()
const { creatingGroup, createDefaultGroup, setMemberships } = injectLibraryGroups()
const { removing: removingSharedInstances, requestRemove: requestRemoveSharedInstances } =
	injectSharedInstanceActions()

const confirmDeleteModal = ref<InstanceType<typeof ConfirmDeleteInstanceModal>>()
const removingFromGroup = ref(false)
const selectedInstanceCount = computed(() => selectedLibraryInstances.value.size)
const selectedInstances = computed(() =>
	instances.value.filter((instance) => selectedInstanceIds.value.has(instance.id)),
)
const selectedGroupedInstances = computed(() =>
	[...selectedLibraryInstances.value.values()].flatMap((selection) => {
		const instance = instances.value.find((candidate) => candidate.id === selection.instanceId)
		return instance?.group_ids.includes(selection.groupId) ? [{ instance, selection }] : []
	}),
)
const selectedUninstalledInstances = computed(() =>
	libraryInstances.value
		.filter(isUninstalledSharedInstance)
		.filter((instance) => selectedInstanceIds.value.has(instance.id)),
)
const hasUninstalledSelection = computed(() => selectedUninstalledInstances.value.length > 0)
const removalMode = computed(() => getSharedInstanceRemovalMode(selectedUninstalledInstances.value))
const deleteMutation = useMutation({
	networkMode: 'always',
	mutationFn: async (instanceIds: string[]) => {
		const results = await Promise.allSettled(instanceIds.map((instanceId) => remove(instanceId)))
		return results.flatMap((result, index) => {
			if (result.status === 'fulfilled') return [instanceIds[index]]
			handleError(toError(result.reason))
			return []
		})
	},
	onMutate: async (instanceIds) => {
		await queryClient.cancelQueries({ queryKey: instanceKeys.list() })
		const previousInstances = queryClient.getQueryData<GameInstance[]>(instanceKeys.list())
		const deletedIds = new Set(instanceIds)
		queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (current) =>
			current?.filter((instance) => !deletedIds.has(instance.id)),
		)
		return { previousInstances }
	},
	onError: (error, _, context) => {
		queryClient.setQueryData(instanceKeys.list(), context?.previousInstances)
		handleError(toError(error))
	},
	onSettled: () => queryClient.invalidateQueries({ queryKey: instanceKeys.list() }),
})
const deleting = computed(() => deleteMutation.isPending.value)

const busy = computed(
	() =>
		deleting.value ||
		removingFromGroup.value ||
		creatingGroup.value ||
		removingSharedInstances.value,
)

const messages = defineMessages({
	ariaLabel: {
		id: 'app.library.selection.aria-label',
		defaultMessage: 'Selected instances',
	},
	selectedCount: {
		id: 'app.library.selection.selected-count',
		defaultMessage: '{count} selected',
	},
	deleting: {
		id: 'app.library.selection.deleting',
		defaultMessage: 'Deleting selected instances',
	},
	newGroup: {
		id: 'app.library.selection.new-group',
		defaultMessage: 'New group',
	},
	removeFromGroup: {
		id: 'app.library.selection.remove-from-group',
		defaultMessage: 'Remove from group',
	},
	removing: {
		id: 'app.library.selection.removing-shared-instances',
		defaultMessage: 'Removing selected shared instances',
	},
})

async function createGroupFromSelection() {
	if (busy.value) return

	const instanceIds = selectedInstanceIds.value
	clearLibraryInstanceSelection()
	await createDefaultGroup(instanceIds)
}

async function removeSelectedInstancesFromGroups() {
	if (busy.value || selectedGroupedInstances.value.length === 0) return

	const selectedGroupIdsByInstanceId = new Map<string, Set<string>>()
	for (const { instance, selection } of selectedGroupedInstances.value) {
		const groupIds = selectedGroupIdsByInstanceId.get(instance.id) ?? new Set()
		groupIds.add(selection.groupId)
		selectedGroupIdsByInstanceId.set(instance.id, groupIds)
	}
	const operations = [...selectedGroupIdsByInstanceId].flatMap(([instanceId, groupIds]) => {
		const instance = instances.value.find((candidate) => candidate.id === instanceId)
		return instance ? [{ instance, groupIds }] : []
	})
	removingFromGroup.value = true
	try {
		await setMemberships.mutateAsync(
			operations.map(({ instance, groupIds }) => ({
				instance_id: instance.id,
				group_ids: instance.group_ids.filter((groupId) => !groupIds.has(groupId)),
			})),
		)

		const nextSelectedInstances = new Map(selectedLibraryInstances.value)
		for (const { instance, groupIds } of operations) {
			for (const groupId of groupIds) {
				nextSelectedInstances.delete(
					getLibraryInstanceSelectionKey({
						instanceId: instance.id,
						groupId,
					}),
				)
			}
		}

		setSelectedLibraryInstances(nextSelectedInstances.values())
	} catch {
		return
	} finally {
		removingFromGroup.value = false
	}
}

async function deleteSelectedInstances() {
	if (busy.value || selectedInstanceCount.value === 0) return

	const deletedInstanceIds = new Set(
		await deleteMutation.mutateAsync([...selectedInstanceIds.value]).catch(() => []),
	)
	setSelectedLibraryInstances(
		[...selectedLibraryInstances.value.values()].filter(
			(selection) => !deletedInstanceIds.has(selection.instanceId),
		),
	)
}
</script>
