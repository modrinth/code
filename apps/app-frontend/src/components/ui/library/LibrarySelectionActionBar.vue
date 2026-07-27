<template>
	<FloatingActionBar
		:shown="selectedInstanceCount > 0"
		:aria-label="formatMessage(messages.ariaLabel)"
		hide-when-modal-open
	>
		<div class="flex items-center gap-0.5">
			<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">
				{{ formatMessage(messages.selectedCount, { count: selectedLibraryInstanceIds.size }) }}
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
		:count="selectedInstanceCount"
		@delete="deleteSelectedInstances"
	/>
</template>

<script setup lang="ts">
import { MinusIcon, TrashIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	FloatingActionBar,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { toError } from '@/helpers/errors'
import { edit, remove } from '@/helpers/instance'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const {
	instances,
	selectedLibraryInstanceIds,
	clearLibraryInstanceSelection,
	setSelectedLibraryInstanceIds,
} = useLibrary()

const confirmDeleteModal = ref<InstanceType<typeof ConfirmDeleteInstanceModal>>()
const deleting = ref(false)
const removingFromGroup = ref(false)
const selectedInstanceCount = computed(() => selectedLibraryInstanceIds.value.size)
const selectedGroupedInstances = computed(() =>
	instances.value.filter(
		(instance) => selectedLibraryInstanceIds.value.has(instance.id) && instance.groups.length > 0,
	),
)
const busy = computed(() => deleting.value || removingFromGroup.value)

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
	removeFromGroup: {
		id: 'app.library.selection.remove-from-group',
		defaultMessage: 'Remove from group',
	},
})

async function removeSelectedInstancesFromGroups() {
	if (busy.value || selectedGroupedInstances.value.length === 0) return

	removingFromGroup.value = true
	const results = await Promise.allSettled(
		selectedGroupedInstances.value.map((instance) => edit(instance.id, { groups: [] })),
	)

	for (const result of results) {
		if (result.status === 'rejected') {
			handleError(toError(result.reason))
		}
	}

	removingFromGroup.value = false
}

async function deleteSelectedInstances() {
	if (busy.value || selectedInstanceCount.value === 0) return

	deleting.value = true
	const instanceIds = [...selectedLibraryInstanceIds.value]
	const results = await Promise.allSettled(instanceIds.map((instanceId) => remove(instanceId)))
	const nextSelectedInstanceIds = new Set(selectedLibraryInstanceIds.value)

	for (const [index, result] of results.entries()) {
		if (result.status === 'rejected') {
			handleError(toError(result.reason))
		} else {
			nextSelectedInstanceIds.delete(instanceIds[index])
		}
	}

	setSelectedLibraryInstanceIds(nextSelectedInstanceIds)
	deleting.value = false
}
</script>
