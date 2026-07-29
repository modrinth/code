<template>
	<ContentDiffModal
		ref="publishReviewModal"
		:header="formatMessage(messages.header)"
		:admonition-header="formatMessage(messages.admonitionHeader)"
		:description="formatMessage(messages.description)"
		:diffs="publishDiffs"
		:confirm-label="formatMessage(messages.publishButton)"
		:confirm-icon="UploadIcon"
		:confirm-disabled="publishDiffs.length === 0 && selectedConfigPaths.length === 0"
		:added-label="formatMessage(messages.addedLabel)"
		:removed-label="formatMessage(messages.removedLabel)"
		@confirm="publishChanges"
		@cancel="finishReview"
	>
		<template #additional-content>
			<Accordion
				class="min-w-0"
				overflow-visible
				button-class="w-full bg-transparent m-0 p-0 border-none"
			>
				<template #title>
					<FileCogIcon class="size-4 shrink-0 text-primary" />
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.configTitle) }}
					</span>
				</template>
				<div class="flex min-w-0 flex-col gap-3 pt-4">
					<div class="max-h-[292px] overflow-y-auto rounded-[20px]">
						<FileTreeSelect
							v-model="selectedConfigPaths"
							:items="configFileItems"
							:show-size="false"
							:show-modified="false"
						/>
					</div>
				</div>
			</Accordion>
		</template>
	</ContentDiffModal>
</template>

<script setup lang="ts">
import { FileCogIcon, UploadIcon } from '@modrinth/assets'
import {
	Accordion,
	type ContentDiffItem,
	ContentDiffModal,
	defineMessages,
	FileTreeSelect,
	type FileTreeSelectItem,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import { get_shared_instance_publish_preview, publish_shared_instance } from '@/helpers/instance'
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

const { formatMessage } = useVIntl()
const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const publishDiffs = ref<ContentDiffItem[]>([])
const configFilePaths = ref<string[]>([])
const selectedConfigPaths = ref<string[]>([])
const state = ref<SharedInstancePublishState>('idle')
const configFileItems = computed<FileTreeSelectItem[]>(() =>
	configFilePaths.value.map((path) => ({ path, type: 'file' })),
)

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
		configFilePaths.value = preview.configFiles
		selectedConfigPaths.value = []
		if (!publishReviewModal.value) return

		publishReviewModal.value.show(e)
		reviewOpened = true
	} catch (error) {
		handlePublishError(error)
	} finally {
		if (!reviewOpened) finishReview()
	}
}

async function publishChanges() {
	if (state.value !== 'reviewing') return

	setState('publishing')
	try {
		await publish_shared_instance(props.instance.id, selectedConfigPaths.value)
		emit('published')
	} catch (error) {
		handlePublishError(error)
	} finally {
		setState('idle')
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

const messages = defineMessages({
	header: {
		id: 'instance.shared-instance.publish-review.header',
		defaultMessage: 'Review changes',
	},
	admonitionHeader: {
		id: 'instance.shared-instance.publish-review.admonition-header',
		defaultMessage: 'Push update to players',
	},
	description: {
		id: 'instance.shared-instance.publish-review.description',
		defaultMessage:
			'Review the content changes and choose any config files to include in this update.',
	},
	publishButton: {
		id: 'instance.shared-instance.publish-review.publish-button',
		defaultMessage: 'Push update',
	},
	addedLabel: {
		id: 'instance.shared-instance.publish-review.added-label',
		defaultMessage: 'Added',
	},
	removedLabel: {
		id: 'instance.shared-instance.publish-review.removed-label',
		defaultMessage: 'Removed',
	},
	configTitle: {
		id: 'instance.shared-instance.publish-review.config-title-v2',
		defaultMessage: 'Select config files',
	},
})

defineExpose({ show })
</script>
