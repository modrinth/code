<template>
	<SharedFilesPublishModal
		ref="modal"
		:diffs="publishDiffs"
		:files="files"
		@confirm="confirmPublish"
		@cancel="finishReview"
	/>
</template>

<script setup lang="ts">
import { type ContentDiffItem, SharedFilesPublishModal } from '@modrinth/ui'
import { ref } from 'vue'

import { trackSharedInstancePublish } from '@/components/ui/download-manager/shared-instance-publishes'
import {
	getErrorMessage,
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import {
	get_shared_instance_publish_preview,
	get_shared_instance_publish_support_details,
	publish_shared_instance,
	type SharedInstancePublishFile,
} from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'

type SharedInstancePublishState = 'idle' | 'reviewing' | 'publishing'

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
	'state-change': [state: SharedInstancePublishState]
}>()

const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()
const modal = ref<InstanceType<typeof SharedFilesPublishModal>>()
const publishDiffs = ref<ContentDiffItem[]>([])
const files = ref<SharedInstancePublishFile[]>([])
const state = ref<SharedInstancePublishState>('idle')

async function show(e?: MouseEvent) {
	if (state.value !== 'idle') return

	setState('reviewing')
	let reviewOpened = false
	try {
		const preview = await get_shared_instance_publish_preview(props.instance.id)
		if (!preview) {
			notifySharedInstanceUnavailable()
			emit('published')
			return
		}

		publishDiffs.value = preview.diffs.map((diff) => ({
			type: diff.type,
			projectName: diff.projectName ?? undefined,
			fileName: diff.fileName ?? undefined,
			currentVersionName: diff.currentVersionName ?? undefined,
			newVersionName: diff.newVersionName ?? undefined,
			fileCount: diff.configFileCount ?? undefined,
			disabled: diff.disabled,
		}))
		files.value = preview.files
		if (!modal.value) return

		modal.value.show({ event: e })
		reviewOpened = true
	} catch (error) {
		handlePublishError(error)
	} finally {
		if (!reviewOpened) finishReview()
	}
}

function confirmPublish(paths: string[]) {
	void publishChanges(paths)
}

async function publishChanges(paths: string[]) {
	if (state.value !== 'reviewing') return

	setState('publishing')
	const task = trackSharedInstancePublish(props.instance)
	try {
		await publish_shared_instance(props.instance.id, paths, task.update)
		await loadSupportDetails(task)
		task.succeed()
		emit('published')
	} catch (error) {
		await loadSupportDetails(task)
		task.fail(getErrorMessage(error))
		handlePublishError(error)
	} finally {
		setState('idle')
	}
}

async function loadSupportDetails(task: ReturnType<typeof trackSharedInstancePublish>) {
	try {
		const details = await get_shared_instance_publish_support_details(props.instance.id)
		if (details) task.setDetails(details)
	} catch {
		return
	}
}

function finishReview() {
	if (state.value === 'reviewing') {
		setState('idle')
	}
}

function handlePublishError(error: unknown) {
	if (isSharedInstanceUnavailableError(error)) {
		notifySharedInstanceUnavailable(getSharedInstanceUnavailableReason(error))
		emit('published')
		return
	}

	notifySharedInstanceError(error)
}

function setState(nextState: SharedInstancePublishState) {
	state.value = nextState
	emit('state-change', nextState)
}

defineExpose({ show })
</script>
