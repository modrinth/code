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
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

import type { ProjectFlagRun } from '@modrinth/api-client'
import { CheckIcon, CopyIcon, DownloadIcon, LoaderCircleIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'

import {
	projectFlagRunActivityKey,
	projectFlagRunHistoryKey,
	projectFlagRunResultsOptions,
} from '~/composables/queries/project-flag-run'

const emit = defineEmits<{ deleted: [id: string] }>()
defineSlots<{
	default(props: {
		runOptions: (run: ProjectFlagRun) => ButtonMenuOption[]
		busy: boolean
	}): unknown
}>()

const client = injectModrinthClient()
const api = client.labrinth.project_flags_internal
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
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: projectFlagRunHistoryKey }),
			queryClient.invalidateQueries({ queryKey: projectFlagRunActivityKey }),
		])
	},
})

// TODO: Use the dedicated export endpoint when available.
function runOptions(run: ProjectFlagRun): ButtonMenuOption[] {
	const runJson = async () => {
		const results = await queryClient.fetchQuery(projectFlagRunResultsOptions(client, run))
		return JSON.stringify({ ...run, results }, null, 2)
	}
	return [
		{
			id: 'copy-run-id',
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
			id: 'copy-run-json',
			label: 'Copy run data',
			icon: CopyIcon,
			action: async () => {
				try {
					if (typeof ClipboardItem !== 'undefined' && navigator.clipboard.write) {
						await navigator.clipboard.write([
							new ClipboardItem({
								'text/plain': runJson().then((json) => new Blob([json], { type: 'text/plain' })),
							}),
						])
					} else {
						await navigator.clipboard.writeText(await runJson())
					}
					addNotification({ title: 'Run data copied', type: 'success' })
				} catch {
					addNotification({ title: 'Could not copy run data', type: 'error' })
				}
			},
			trailingAction: {
				label: 'Download run data as JSON',
				icon: DownloadIcon,
				action: async (event) => {
					const document = (event.currentTarget as HTMLElement).ownerDocument
					const view = document.defaultView
					if (!view) return

					try {
						const blob = new Blob([await runJson()], { type: 'application/json' })
						const url = view.URL.createObjectURL(blob)
						const link = document.createElement('a')
						link.href = url
						link.download = `project-flag-run-${run.id}.json`
						link.click()
						view.setTimeout(() => view.URL.revokeObjectURL(url), 1000)
					} catch {
						addNotification({ title: 'Could not download run data', type: 'error' })
					}
				},
			},
		},
		{
			id: 'apply-run',
			label: run.applied_at ? 'Run applied' : 'Apply run',
			icon: CheckIcon,
			disabled: run.status !== 'done' || run.error !== null || run.applied_at !== null,
			action: () => applyModal.value?.show(),
		},
		{
			id: 'delete-run',
			label: 'Delete run',
			icon: TrashIcon,
			tone: 'red',
			action: () => confirmDelete(run),
		},
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

useFlagErrorNotification(deleteError, 'Could not delete the run')
</script>
