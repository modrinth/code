<template>
	<slot :run-options="runOptions" :busy="isDeleting" />

	<NewModal ref="applyModal" header="Apply run">
		<div class="flex w-[32rem] max-w-full flex-col gap-4">
			<p class="m-0 text-secondary">Applying flags from a run is not available yet.</p>
			<div class="flex justify-end gap-2">
				<Button @click="applyModal?.hide()">Close</Button>
				<Button type="colored" color="brand" disabled>
					<CheckIcon />
					Apply run
				</Button>
			</div>
		</div>
	</NewModal>

	<NewModal
		ref="deleteModal"
		header="Delete run"
		:disable-close="isDeleting && runToDelete !== null"
		:on-hide="closeDelete"
	>
		<div v-if="runToDelete" class="flex w-[32rem] max-w-full flex-col gap-4">
			<p class="m-0 break-words">
				Delete run <span class="font-mono">{{ runToDelete.id }}</span
				>?
			</p>
			<p class="m-0 text-secondary">
				This permanently deletes the run, its results, and its inactive project flags. Active flags
				are preserved and detached from the run. This cannot be undone.
			</p>
			<p v-if="deleteError" role="alert" class="m-0 text-red">
				Could not delete the run: {{ errorMessage(deleteError) }}
			</p>
			<div class="flex justify-end gap-2">
				<Button :disabled="isDeleting" @click="deleteModal?.hide()">Cancel</Button>
				<Button
					type="colored"
					color="red"
					:disabled="isDeleting"
					@click="deleteRun(runToDelete.id)"
				>
					<LoaderCircleIcon v-if="isDeleting" class="animate-spin" />
					<TrashIcon v-else />
					{{ isDeleting ? 'Deleting…' : 'Delete run' }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { ProjectFlagRun } from '@modrinth/api-client'
import { CheckIcon, CopyIcon, LoaderCircleIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'

import { projectFlagRunHistoryKey } from '~/composables/queries/project-flag-run'

const emit = defineEmits<{ deleted: [id: string] }>()
defineSlots<{
	default(props: {
		runOptions: (run: ProjectFlagRun) => ButtonMenuOption[]
		busy: boolean
	}): unknown
}>()

const api = injectModrinthClient().labrinth.project_flags_internal
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()

const applyModal = useTemplateRef<InstanceType<typeof NewModal>>('applyModal')
const deleteModal = useTemplateRef<InstanceType<typeof NewModal>>('deleteModal')
const runToDelete = ref<ProjectFlagRun | null>(null)
const {
	mutate: deleteRun,
	isPending: isDeleting,
	error: deleteError,
	reset: resetDelete,
} = useMutation({
	mutationFn: (id: string) => api.deleteRun(id),
	retry: false,
	onSuccess: async (_, id) => {
		runToDelete.value = null
		await nextTick()
		deleteModal.value?.hide()
		await queryClient.cancelQueries({ queryKey: projectFlagRunHistoryKey })
		queryClient.removeQueries({ queryKey: ['moderation', 'project-flag', 'runs', id] })
		queryClient.setQueryData<ProjectFlagRun[]>(projectFlagRunHistoryKey, (runs) =>
			runs?.filter((run) => run.id !== id),
		)
		emit('deleted', id)
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag'] })
	},
})

function runOptions(run: ProjectFlagRun): ButtonMenuOption[] {
	return [
		{
			label: 'Copy run ID',
			icon: CopyIcon,
			action: async () => {
				try {
					await navigator.clipboard.writeText(run.id)
					addNotification({ title: 'Run ID copied', type: 'success' })
				} catch {
					addNotification({ title: 'Could not copy run ID', type: 'error' })
				}
			},
		},
		{
			label: run.applied_at ? 'Run applied' : 'Apply run',
			icon: CheckIcon,
			disabled: run.status !== 'done' || run.error !== null || run.applied_at !== null,
			action: () => applyModal.value?.show(),
		},
		{ label: 'Delete run', icon: TrashIcon, tone: 'red', action: () => confirmDelete(run) },
	]
}

function confirmDelete(run: ProjectFlagRun) {
	resetDelete()
	runToDelete.value = run
	deleteModal.value?.show()
}

function closeDelete() {
	runToDelete.value = null
	resetDelete()
}

function errorMessage(value: unknown) {
	return value instanceof Error ? value.message : 'Please try again.'
}
</script>
