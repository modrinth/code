<template>
	<ContentDiffModal
		ref="modal"
		:header="formatMessage(messages.updateToPlay)"
		:admonition-header="formatMessage(messages.updateRequired)"
		:description="instance ? formatMessage(messages.description, { name: instance.name }) : ''"
		:diffs="diffs"
		:confirm-label="formatMessage(commonMessages.updateButton)"
		:confirm-icon="DownloadIcon"
		:added-label="formatMessage(messages.addedLabel)"
		:removed-label="formatMessage(messages.removedLabel)"
		:show-report-button="false"
		show-external-warnings
		@confirm="update"
		@cancel="emit('cancel')"
	/>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import {
	commonMessages,
	type ContentDiffItem,
	ContentDiffModal,
	defineMessages,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getSharedInstanceUnavailableReason,
	install_update_shared_instance,
	isSharedInstanceUnavailableError,
	type SharedInstanceUnavailableReason,
	type SharedInstanceUpdatePreview,
	wait_for_install_job,
} from '@/helpers/install'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'

type UpdateCompleteCallback = () => void | Promise<void>

const emit = defineEmits<{
	cancel: []
	complete: []
	sharedInstanceUnavailable: [reason: SharedInstanceUnavailableReason | null]
}>()
const modal = ref<InstanceType<typeof ContentDiffModal>>()
const instance = ref<GameInstance | null>(null)
const preview = ref<SharedInstanceUpdatePreview | null>(null)
const onComplete = ref<UpdateCompleteCallback>(() => {})
const { formatMessage } = useVIntl()
const { notifySharedInstanceError } = useSharedInstanceErrors()
const diffs = computed<ContentDiffItem[]>(
	() =>
		preview.value?.diffs.map((diff) => ({
			type: diff.type,
			projectName: diff.projectName ?? undefined,
			fileName: diff.fileName ?? undefined,
			currentVersionName: diff.currentVersionName ?? undefined,
			newVersionName: diff.newVersionName ?? undefined,
			disabled: diff.disabled,
			external: diff.type === 'added' && !diff.projectId && !!diff.fileName,
		})) ?? [],
)

async function update() {
	try {
		if (instance.value) {
			const job = await install_update_shared_instance(instance.value.id)
			await wait_for_install_job(job.job_id)
			await onComplete.value()
		}
	} catch (error) {
		if (isSharedInstanceUnavailableError(error)) {
			emit('sharedInstanceUnavailable', getSharedInstanceUnavailableReason(error))
			return
		}
		notifySharedInstanceError(error)
	} finally {
		emit('complete')
	}
}

function show(
	instanceValue: GameInstance,
	previewValue: SharedInstanceUpdatePreview,
	callback: UpdateCompleteCallback = () => {},
	event?: MouseEvent,
) {
	instance.value = instanceValue
	preview.value = previewValue
	onComplete.value = callback
	modal.value?.show(event)
}
function hide() {
	modal.value?.hide()
}

const messages = defineMessages({
	updateToPlay: { id: 'app.modal.update-to-play.header', defaultMessage: 'Update to play' },
	updateRequired: {
		id: 'app.modal.update-to-play.update-required',
		defaultMessage: 'Update required',
	},
	description: {
		id: 'app.modal.update-to-play.update-required-description',
		defaultMessage:
			'An update is required to play {name}. Please update to latest version to launch the game.',
	},
	addedLabel: {
		id: 'app.modal.update-to-play.shared-instance-added-label',
		defaultMessage: 'Added',
	},
	removedLabel: {
		id: 'app.modal.update-to-play.shared-instance-removed-label',
		defaultMessage: 'Removed',
	},
})

defineExpose({ show, hide })
</script>
