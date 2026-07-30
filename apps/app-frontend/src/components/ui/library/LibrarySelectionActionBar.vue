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
			<ButtonStyled type="transparent">
				<button
					class="!text-primary"
					type="button"
					:disabled="busy"
					@click="clearLibraryInstanceSelection"
				>
					<span class="bar-label">{{ formatMessage(commonMessages.clearButton) }}</span>
				</button>
			</ButtonStyled>
		</div>
		<div class="ml-auto flex items-center gap-0.5">
			<ButtonStyled v-if="displayState.group === 'Group'" type="transparent">
				<button type="button" :disabled="busy" @click="createGroupFromSelection">
					<SquarePlusIcon />
					<span class="bar-label">{{ formatMessage(messages.newGroup) }}</span>
				</button>
			</ButtonStyled>
			<ButtonStyled v-if="selectedGroupedInstances.length > 0" type="transparent">
				<button type="button" :disabled="busy" @click="removeSelectedInstancesFromGroups">
					<MinusIcon />
					<span class="bar-label">{{ formatMessage(messages.removeFromGroup) }}</span>
				</button>
			</ButtonStyled>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<ButtonStyled type="transparent" color="red" color-fill="text" hover-color-fill="background">
				<button
					v-tooltip="deleting ? formatMessage(messages.deleting) : undefined"
					type="button"
					:disabled="busy"
					@click="confirmDeleteModal?.show()"
				>
					<TrashIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</button>
			</ButtonStyled>
		</div>
	</FloatingActionBar>
	<ConfirmDeleteInstanceModal
		ref="confirmDeleteModal"
		:count="selectedInstanceIds.size"
		@delete="deleteSelectedInstances"
	/>
</template>

<script setup lang="ts">
import { MinusIcon, SquarePlusIcon, TrashIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	FloatingActionBar,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { getLibraryInstanceSelectionKey, useLibrary } from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { toError } from '@/helpers/errors'
import { edit, remove } from '@/helpers/instance'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const {
	instances,
	selectedLibraryInstances,
	clearLibraryInstanceSelection,
	setSelectedLibraryInstances,
	creatingGroup,
	createDefaultGroup,
	displayState,
} = useLibrary()

const confirmDeleteModal = ref<InstanceType<typeof ConfirmDeleteInstanceModal>>()
const deleting = ref(false)
const removingFromGroup = ref(false)
const selectedInstanceCount = computed(() => selectedLibraryInstances.value.size)
const selectedInstanceIds = computed(
	() =>
		new Set([...selectedLibraryInstances.value.values()].map((selection) => selection.instanceId)),
)
const selectedGroupedInstances = computed(() =>
	[...selectedLibraryInstances.value.values()].flatMap((selection) => {
		const instance = instances.value.find((candidate) => candidate.id === selection.instanceId)
		return instance?.group_ids.includes(selection.groupId) ? [{ instance, selection }] : []
	}),
)
const busy = computed(() => deleting.value || removingFromGroup.value || creatingGroup.value)

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
})

async function createGroupFromSelection() {
	if (busy.value) return

	const instanceIds = selectedInstanceIds.value
	clearLibraryInstanceSelection()
	await createDefaultGroup(instanceIds)
}

async function removeSelectedInstancesFromGroups() {
	if (busy.value || selectedGroupedInstances.value.length === 0) return

	removingFromGroup.value = true
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
	const results = await Promise.allSettled(
		operations.map(({ instance, groupIds }) => {
			return edit(instance.id, {
				group_ids: instance.group_ids.filter((groupId) => !groupIds.has(groupId)),
			})
		}),
	)

	const nextSelectedInstances = new Map(selectedLibraryInstances.value)
	for (const [index, result] of results.entries()) {
		if (result.status === 'rejected') {
			handleError(toError(result.reason))
		} else {
			for (const groupId of operations[index].groupIds) {
				nextSelectedInstances.delete(
					getLibraryInstanceSelectionKey({
						instanceId: operations[index].instance.id,
						groupId,
					}),
				)
			}
		}
	}

	setSelectedLibraryInstances(nextSelectedInstances.values())
	removingFromGroup.value = false
}

async function deleteSelectedInstances() {
	if (busy.value || selectedInstanceCount.value === 0) return

	deleting.value = true
	const instanceIds = [...selectedInstanceIds.value]
	const results = await Promise.allSettled(instanceIds.map((instanceId) => remove(instanceId)))
	const deletedInstanceIds = new Set<string>()

	for (const [index, result] of results.entries()) {
		if (result.status === 'rejected') {
			handleError(toError(result.reason))
		} else {
			deletedInstanceIds.add(instanceIds[index])
		}
	}

	setSelectedLibraryInstances(
		[...selectedLibraryInstances.value.values()].filter(
			(selection) => !deletedInstanceIds.has(selection.instanceId),
		),
	)
	deleting.value = false
}
</script>
