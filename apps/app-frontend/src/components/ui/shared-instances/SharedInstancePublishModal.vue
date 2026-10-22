<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		:on-hide="finishReview"
		max-width="544px"
		width="544px"
		no-padding
		scrollable
	>
		<div class="flex flex-col gap-4 px-6 py-4">
			<Tabs v-model:value="activeTab" :tabs="tabs" size="large" />
			<p class="m-0 text-base text-primary">
				{{
					formatMessage(
						activeTab === 'changes' ? messages.changesDescription : messages.filesDescription,
					)
				}}
			</p>
		</div>

		<ContentDiffList
			v-show="activeTab === 'changes'"
			:diffs="publishDiffs"
			:added-label="formatMessage(messages.addedLabel)"
			:removed-label="formatMessage(messages.removedLabel)"
		/>
		<div v-show="activeTab === 'files'">
			<div
				ref="fileTreeContainer"
				class="max-h-[336px] overflow-y-auto border-0 border-y border-solid border-surface-5"
			>
				<FileTreeSelect
					v-model="includedFilePaths"
					v-model:excluded-paths="excludedFilePaths"
					:items="fileItems"
					:show-modified="false"
					flush
					@navigate="scrollFileTreeToTop"
				/>
			</div>
			<div class="flex flex-col gap-2 border-0 border-b border-solid border-surface-5 px-6 py-4">
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

		<template #actions>
			<div class="flex justify-end gap-2 p-2">
				<Button type="outlined" class="!border" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="confirmDisabled" @click="confirmPublish">
					<UploadIcon />
					{{ formatMessage(messages.publishButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { UploadIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	type ContentDiffItem,
	ContentDiffList,
	defineMessages,
	FileTreeSelect,
	type FileTreeSelectItem,
	NewModal,
	Tabs,
	type TabsTab,
	type TabsValue,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

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
const modal = ref<InstanceType<typeof NewModal>>()
const fileTreeContainer = ref<HTMLElement>()
const publishDiffs = ref<ContentDiffItem[]>([])
const files = ref<SharedInstancePublishFile[]>([])
const includedFilePaths = ref<string[]>([])
const excludedFilePaths = ref<string[]>([])
const activeTab = ref<TabsValue>('changes')
const state = ref<SharedInstancePublishState>('idle')
const tabs = computed<TabsTab[]>(() => [
	{ value: 'changes', label: formatMessage(messages.changesTab) },
	{ value: 'files', label: formatMessage(messages.filesTab) },
])
const fileItems = computed<FileTreeSelectItem[]>(() =>
	files.value.map(({ path, size, blocked }) => ({
		path,
		size,
		type: 'file',
		disabled: blocked,
		disabledTooltip: blocked ? formatMessage(messages.fileTypeNotAllowed) : undefined,
	})),
)
const selectedFiles = computed(() => {
	const includedPaths = new Set(includedFilePaths.value)
	const excludedPaths = new Set(excludedFilePaths.value)

	return files.value.filter(
		(file) => !file.blocked && isPathSelected(file.path, includedPaths, excludedPaths),
	)
})
const selectedFilePaths = computed(() => selectedFiles.value.map((file) => file.path))
const selectedSize = computed(() =>
	selectedFiles.value.reduce((total, file) => total + file.size, 0),
)
const overStorageLimit = computed(() => selectedSize.value > STORAGE_LIMIT_BYTES)
const confirmDisabled = computed(
	() =>
		overStorageLimit.value ||
		(publishDiffs.value.length === 0 && selectedFilePaths.value.length === 0),
)
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
		activeTab.value = 'changes'
		if (!modal.value) return

		modal.value.show(e)
		reviewOpened = true
	} catch (error) {
		handlePublishError(error)
	} finally {
		if (!reviewOpened) finishReview()
	}
}

function confirmPublish() {
	if (confirmDisabled.value) return

	void publishChanges()
	modal.value?.hide()
}

async function publishChanges() {
	if (state.value !== 'reviewing' || overStorageLimit.value) return

	setState('publishing')
	const task = trackSharedInstancePublish(props.instance)
	try {
		await publish_shared_instance(props.instance.id, selectedFilePaths.value, task.update)
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
		defaultMessage: 'Push update',
	},
	changesTab: {
		id: 'instance.shared-instance.publish-review.changes-tab',
		defaultMessage: 'Changes',
	},
	filesTab: {
		id: 'instance.shared-instance.publish-review.files-tab',
		defaultMessage: 'Files',
	},
	changesDescription: {
		id: 'instance.shared-instance.publish-review.changes-description',
		defaultMessage:
			"Review the changes we've detected since your last update. These will automatically be included when you push.",
	},
	filesDescription: {
		id: 'instance.shared-instance.publish-review.files-description',
		defaultMessage:
			"You can also include individual files alongside the detected changes. Select any additional files you'd like to publish.",
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
	fileTypeNotAllowed: {
		id: 'instance.shared-instance.publish-review.file-type-not-allowed',
		defaultMessage: "You're not allowed to push this file type.",
	},
})

defineExpose({ show })
</script>
