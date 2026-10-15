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
		:hide-summary="configFilesExpanded"
		@confirm="publishChanges"
		@cancel="finishReview"
	>
		<template #additional-content>
			<button
				type="button"
				class="m-0 flex w-full cursor-pointer items-center gap-1 border-none bg-transparent p-0 text-contrast"
				:aria-expanded="configFilesExpanded"
				@click="configFilesExpanded = !configFilesExpanded"
			>
				<FileCogIcon class="size-4 shrink-0 text-primary" />
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.configTitle) }}
				</span>
				<DropdownIcon
					class="ml-auto size-5 shrink-0 text-contrast transition-transform duration-300"
					:class="{ 'rotate-180': configFilesExpanded }"
				/>
			</button>
			<div
				class="grid transition-[grid-template-rows] duration-300 ease-in-out motion-reduce:transition-none"
				:class="configFilesExpanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
				:inert="!configFilesExpanded"
			>
				<div class="min-h-0 min-w-0 overflow-hidden">
					<div
						ref="configFileTreeContainer"
						class="mt-4 max-h-[calc(70vh-4rem)] overflow-y-auto rounded-[20px]"
					>
						<FileTreeSelect
							v-model="includedConfigPaths"
							v-model:excluded-paths="excludedConfigPaths"
							:items="configFileItems"
							:show-size="false"
							:show-modified="false"
							@navigate="scrollConfigFileTreeToTop"
						/>
					</div>
				</div>
			</div>
		</template>
	</ContentDiffModal>
</template>

<script setup lang="ts">
import { DropdownIcon, FileCogIcon, UploadIcon } from '@modrinth/assets'
import {
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
const configFileTreeContainer = ref<HTMLElement>()
const publishDiffs = ref<ContentDiffItem[]>([])
const configFilePaths = ref<string[]>([])
const includedConfigPaths = ref<string[]>([])
const excludedConfigPaths = ref<string[]>([])
const configFilesExpanded = ref(false)
const state = ref<SharedInstancePublishState>('idle')
const configFileItems = computed<FileTreeSelectItem[]>(() =>
	configFilePaths.value.map((path) => ({ path, type: 'file' })),
)
const selectedConfigPaths = computed(() => {
	const includedPaths = new Set(includedConfigPaths.value)
	const excludedPaths = new Set(excludedConfigPaths.value)

	return configFilePaths.value.filter((path) =>
		isConfigPathSelected(path, includedPaths, excludedPaths),
	)
})

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
		includedConfigPaths.value = []
		excludedConfigPaths.value = []
		configFilesExpanded.value = false
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

function isConfigPathSelected(
	path: string,
	includedPaths: Set<string>,
	excludedPaths: Set<string>,
) {
	let selected = false
	let prefix = ''

	for (const segment of path.split('/').filter(Boolean)) {
		prefix = prefix ? `${prefix}/${segment}` : segment
		if (includedPaths.has(prefix)) selected = true
		if (excludedPaths.has(prefix)) selected = false
	}

	return selected
}

function scrollConfigFileTreeToTop() {
	if (configFileTreeContainer.value) {
		configFileTreeContainer.value.scrollTop = 0
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
