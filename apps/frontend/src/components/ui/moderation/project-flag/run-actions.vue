<template>
	<slot :run-options="runOptions" :busy="isDeleting || isDeploying" />

	<NewModal
		ref="deployModal"
		header="Deploy tested rules"
		:disable-close="isDeploying"
		:on-hide="closeDeploy"
	>
		<div v-if="runToDeploy" class="flex w-[32rem] max-w-full flex-col gap-4">
			<p class="m-0 break-words">
				Deploy revision <code>{{ runToDeploy.ruleset_id }}</code> for live project reconciliation?
				This revision contains the rules and modules tested by this run. Deployment does not apply
				the run results to projects.
			</p>
			<p v-if="deploymentPending" class="m-0 text-secondary">Loading deployment status…</p>
			<p v-else-if="deploymentError" role="alert" class="m-0 text-red">
				Could not load deployment status.
			</p>
			<p v-else-if="deployRunPending" class="m-0 text-secondary">Checking run history…</p>
			<p v-else-if="deployRunError || !deployableRun" role="alert" class="m-0 text-red">
				The tested revision could not be verified against Temporal history.
			</p>
			<div class="flex justify-end gap-2">
				<Button :disabled="isDeploying" @click="deployModal?.hide()">Cancel</Button>
				<Button
					type="colored"
					color="brand"
					:loading="isDeploying"
					:disabled="!canDeploy"
					@click="deployRules(runToDeploy.id)"
				>
					Deploy rules
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
				This terminates any running execution and permanently deletes its Temporal history, results,
				and unretained inactive flags. Active and retained flags are detached and preserved.
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
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

import { projectFlagDeploymentQueryOptions } from '~/composables/queries/project-flag'
import {
	projectFlagRunActivityKey,
	projectFlagRunDetailOptions,
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

const deployModal = useTemplateRef<InstanceType<typeof NewModal>>('deployModal')
const runToDeploy = ref<ProjectFlagRun | null>(null)
const deploymentOptions = projectFlagDeploymentQueryOptions(client)
const {
	data: deployment,
	isPending: deploymentPending,
	error: deploymentError,
} = useQuery(deploymentOptions)
const {
	data: deployRun,
	isPending: deployRunPending,
	error: deployRunError,
} = useQuery(
	computed(() => ({
		...projectFlagRunDetailOptions(client, runToDeploy.value?.id ?? ''),
		enabled: runToDeploy.value !== null,
	})),
)
const deployableRun = computed(
	() =>
		deployRun.value?.status === 'completed' &&
		deployRun.value.ruleset_id === runToDeploy.value?.ruleset_id,
)
const {
	mutate: deployRules,
	isPending: isDeploying,
	isSuccess: deploySuccess,
	error: deployError,
	reset: resetDeploy,
} = useMutation({
	mutationFn: (id: string) => api.deployRun(id),
	retry: false,
	onSuccess: async (live) => {
		queryClient.setQueryData(deploymentOptions.queryKey, live)
		addNotification({ type: 'success', title: 'Tested rules deployed' })
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag'] })
	},
})
const canDeploy = computed(
	() =>
		deployableRun.value &&
		!deployRunError.value &&
		!deploymentPending.value &&
		!deploymentError.value &&
		!isDeploying.value &&
		deployment.value?.ruleset_id !== runToDeploy.value?.ruleset_id,
)
watch(
	deploySuccess,
	(success) => {
		if (success) deployModal.value?.hide()
	},
	{ flush: 'post' },
)

function closeDeploy() {
	runToDeploy.value = null
	resetDeploy()
}
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
	const alreadyDeployed = !!run.ruleset_id && deployment.value?.ruleset_id === run.ruleset_id
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
			id: 'deploy-run',
			label: alreadyDeployed ? 'Rules already deployed' : 'Deploy tested rules',
			icon: CheckIcon,
			disabled: run.status !== 'completed' || !run.ruleset_id || alreadyDeployed,
			action: () => {
				runToDeploy.value = run
				deployModal.value?.show()
			},
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

useFlagErrorNotification(deployError, 'Could not deploy tested rules')
useFlagErrorNotification(deleteError, 'Could not delete the run')
</script>
