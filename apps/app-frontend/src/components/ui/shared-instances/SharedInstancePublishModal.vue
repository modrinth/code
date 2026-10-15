<template>
	<ContentDiffModal
		ref="publishReviewModal"
		:header="formatMessage(messages.header)"
		:admonition-header="formatMessage(messages.admonitionHeader)"
		:description="formatMessage(messages.description)"
		:diffs="publishDiffs"
		:confirm-label="formatMessage(messages.publishButton)"
		:confirm-icon="UploadIcon"
		:confirm-disabled="
			overStorageLimit || (publishDiffs.length === 0 && selectedFilePaths.length === 0)
		"
		:added-label="formatMessage(messages.addedLabel)"
		:removed-label="formatMessage(messages.removedLabel)"
		:hide-summary="filesExpanded"
		@confirm="publishChanges"
		@cancel="finishReview"
	>
		<template #additional-content>
			<button
				type="button"
				class="m-0 flex w-full cursor-pointer items-center gap-1 border-none bg-transparent p-0 text-contrast"
				:aria-expanded="filesExpanded"
				@click="filesExpanded = !filesExpanded"
			>
				<FolderOpenIcon class="size-4 shrink-0 text-primary" />
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.filesTitle) }}
				</span>
				<DropdownIcon
					class="ml-auto size-5 shrink-0 text-contrast transition-transform duration-300"
					:class="{ 'rotate-180': filesExpanded }"
				/>
			</button>
			<div
				class="grid transition-[grid-template-rows] duration-300 ease-in-out motion-reduce:transition-none"
				:class="filesExpanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
				:inert="!filesExpanded"
			>
				<div class="flex min-h-0 min-w-0 flex-col gap-4 overflow-hidden">
					<div
						ref="fileTreeContainer"
						class="mt-2 max-h-[calc(70vh-10rem)] overflow-y-auto rounded-[20px]"
					>
						<FileTreeSelect
							v-model="includedFilePaths"
							v-model:excluded-paths="excludedFilePaths"
							:items="fileItems"
							:show-modified="false"
							@navigate="scrollFileTreeToTop"
						/>
					</div>
					<div class="flex flex-col gap-2">
						<div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
							<span class="font-semibold text-contrast">
								{{ formatMessage(messages.storageTitle) }}
							</span>
							<span
								class="font-semibold tabular-nums"
								:class="overStorageLimit ? 'text-red' : 'text-contrast'"
								aria-live="polite"
							>
								{{
									formatMessage(messages.storageUsage, {
										used: formatBytes(selectedSize, 1),
										limit: formatBytes(STORAGE_LIMIT_BYTES, 0),
									})
								}}
							</span>
						</div>
						<div
							role="meter"
							:aria-label="formatMessage(messages.storageTitle)"
							:aria-valuenow="selectedSize"
							:aria-valuemin="0"
							:aria-valuemax="STORAGE_LIMIT_BYTES"
							class="relative h-3.5 overflow-hidden rounded-full bg-surface-4"
						>
							<div
								class="absolute inset-y-0 left-0 bg-gradient-to-b from-white/10 to-black/10 motion-safe:transition-[width,background-color] motion-safe:duration-300"
								:style="{ width: `${withinLimitPercent}%`, backgroundColor: storageBarColor }"
							/>
							<div
								v-if="overStorageLimit"
								class="absolute inset-y-0 right-0 bg-red bg-gradient-to-b from-white/10 to-black/10 motion-safe:transition-[left] motion-safe:duration-300"
								:style="{ left: `calc(${limitMarkerPercent}% + 2px)` }"
							/>
							<div
								v-if="overStorageLimit"
								class="absolute inset-y-0 w-0 border-0 border-l-2 border-dotted border-contrast"
								:style="{ left: `${limitMarkerPercent}%` }"
								aria-hidden="true"
							/>
						</div>
						<p v-if="overStorageLimit" class="m-0 text-sm text-red">
							{{
								formatMessage(messages.storageOverLimit, {
									size: formatBytes(selectedSize - STORAGE_LIMIT_BYTES, 1),
								})
							}}
						</p>
					</div>
				</div>
			</div>
		</template>
	</ContentDiffModal>
</template>

<script setup lang="ts">
import { DropdownIcon, FolderOpenIcon, UploadIcon } from '@modrinth/assets'
import {
	type ContentDiffItem,
	ContentDiffModal,
	defineMessages,
	FileTreeSelect,
	type FileTreeSelectItem,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import {
	get_shared_instance_publish_preview,
	publish_shared_instance,
	type SharedInstancePublishFile,
} from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'

type SharedInstancePublishState = 'idle' | 'reviewing' | 'publishing'

const STORAGE_LIMIT_BYTES = 2 * 1024 ** 3
const STORAGE_WARNING_RATIO = 0.75

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
	'state-change': [state: SharedInstancePublishState]
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const fileTreeContainer = ref<HTMLElement>()
const publishDiffs = ref<ContentDiffItem[]>([])
const files = ref<SharedInstancePublishFile[]>([])
const includedFilePaths = ref<string[]>([])
const excludedFilePaths = ref<string[]>([])
const filesExpanded = ref(false)
const state = ref<SharedInstancePublishState>('idle')
const fileItems = computed<FileTreeSelectItem[]>(() =>
	files.value.map(({ path, size }) => ({ path, size, type: 'file' })),
)
const selectedFiles = computed(() => {
	const includedPaths = new Set(includedFilePaths.value)
	const excludedPaths = new Set(excludedFilePaths.value)

	return files.value.filter((file) => isPathSelected(file.path, includedPaths, excludedPaths))
})
const selectedFilePaths = computed(() => selectedFiles.value.map((file) => file.path))
const selectedSize = computed(() =>
	selectedFiles.value.reduce((total, file) => total + file.size, 0),
)
const overStorageLimit = computed(() => selectedSize.value > STORAGE_LIMIT_BYTES)
const storageBarScale = computed(() => Math.max(selectedSize.value, STORAGE_LIMIT_BYTES))
const withinLimitPercent = computed(
	() => (Math.min(selectedSize.value, STORAGE_LIMIT_BYTES) / storageBarScale.value) * 100,
)
const limitMarkerPercent = computed(() => (STORAGE_LIMIT_BYTES / storageBarScale.value) * 100)
const storageBarColor = computed(() => {
	const ratio = Math.min(selectedSize.value / STORAGE_LIMIT_BYTES, 1)
	if (ratio <= STORAGE_WARNING_RATIO) {
		const orangeWeight = (ratio / STORAGE_WARNING_RATIO) * 100
		return `color-mix(in oklch, var(--color-green), var(--color-orange) ${orangeWeight}%)`
	}
	const redWeight = ((ratio - STORAGE_WARNING_RATIO) / (1 - STORAGE_WARNING_RATIO)) * 100
	return `color-mix(in oklch, var(--color-orange), var(--color-red) ${redWeight}%)`
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
		files.value = preview.files
		includedFilePaths.value = []
		excludedFilePaths.value = []
		filesExpanded.value = false
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
	if (state.value !== 'reviewing' || overStorageLimit.value) return

	setState('publishing')
	try {
		await publish_shared_instance(props.instance.id, selectedFilePaths.value)
		emit('published')
	} catch (error) {
		handlePublishError(error)
	} finally {
		setState('idle')
	}
}

function isPathSelected(path: string, includedPaths: Set<string>, excludedPaths: Set<string>) {
	let selected = false
	let prefix = ''

	for (const segment of path.split('/').filter(Boolean)) {
		prefix = prefix ? `${prefix}/${segment}` : segment
		if (includedPaths.has(prefix)) selected = true
		if (excludedPaths.has(prefix)) selected = false
	}

	return selected
}

function scrollFileTreeToTop() {
	if (fileTreeContainer.value) {
		fileTreeContainer.value.scrollTop = 0
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
		defaultMessage: 'Review the content changes and choose any files to include in this update.',
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
	filesTitle: {
		id: 'instance.shared-instance.publish-review.files-title',
		defaultMessage: 'Select files',
	},
	storageTitle: {
		id: 'instance.shared-instance.publish-review.storage-title',
		defaultMessage: 'Storage allowance',
	},
	storageUsage: {
		id: 'instance.shared-instance.publish-review.storage-usage',
		defaultMessage: '{used} of {limit}',
	},
	storageOverLimit: {
		id: 'instance.shared-instance.publish-review.storage-over-limit',
		defaultMessage: '{size} over the limit. Deselect some files to push this update.',
	},
})

defineExpose({ show })
</script>
