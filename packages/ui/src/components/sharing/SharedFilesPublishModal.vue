<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		:on-hide="handleHide"
		:fade="requiresRestart ? 'warning' : undefined"
		max-width="544px"
		width="544px"
		no-padding
		scrollable
	>
		<div class="flex flex-col gap-4 px-6 py-4">
			<Tabs v-model:value="activeTab" :tabs="tabs" size="large" />
			<p class="m-0 text-base text-primary">
				{{
					activeTab === 'changes'
						? (changesDescription ?? formatMessage(messages.changesDescription))
						: formatMessage(messages.filesDescription)
				}}
			</p>
			<template v-if="activeTab === 'changes'">
				<p v-if="changesLoading" class="m-0 flex items-center gap-2">
					<SpinnerIcon class="animate-spin" />{{ formatMessage(messages.refreshingChanges) }}
				</p>
				<Admonition
					v-else-if="changesError"
					type="critical"
					:header="formatMessage(messages.changesError)"
				>
					<Button @click="emit('retry-changes')">{{ formatMessage(messages.retry) }}</Button>
				</Admonition>
			</template>
			<template v-else>
				<p v-if="filesLoading" class="m-0 flex items-center gap-2">
					<SpinnerIcon class="animate-spin" />{{ formatMessage(messages.loadingFiles) }}
				</p>
				<Admonition
					v-else-if="filesError"
					type="critical"
					:header="formatMessage(messages.filesError)"
				>
					<Button @click="emit('retry-files')">{{ formatMessage(messages.retry) }}</Button>
				</Admonition>
			</template>
		</div>

		<ContentDiffList
			v-show="activeTab === 'changes'"
			:diffs="diffs"
			:added-label="formatMessage(messages.addedLabel)"
			:removed-label="formatMessage(messages.removedLabel)"
		/>
		<div v-show="activeTab === 'files'">
			<div
				ref="fileTreeContainer"
				class="max-h-[clamp(120px,calc(100vh-470px),336px)] overflow-y-auto border-0 border-y border-solid border-surface-5"
				:inert="disabled"
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
			<SharedFilesStorageMeter
				:used="selectedSize"
				:limit="STORAGE_LIMIT_BYTES"
				class="px-6 pt-4"
			/>
		</div>

		<template #actions>
			<div class="flex flex-col gap-4 p-2">
				<div v-if="requiresRestart" class="flex flex-row items-start gap-2 text-primary">
					<CircleAlertIcon class="size-6 shrink-0 text-orange" />
					<span class="leading-6">{{ formatMessage(messages.restartWarning) }}</span>
				</div>
				<div class="flex flex-row justify-end gap-2">
					<Button type="outlined" class="!border" @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<SplitButton
						v-if="requiresRestart"
						type="colored"
						color="orange"
						:disabled="confirmDisabled"
						:primary-disabled="restartDisabled"
						:options="restartOptions"
						:menu-label="formatMessage(messages.moreOptions)"
						@click="handleConfirm(true)"
					>
						<RotateCounterClockwiseIcon />
						{{ restartConfirmLabel ?? formatMessage(messages.pushAndRestartButton) }}
					</SplitButton>
					<Button
						v-else
						type="colored"
						color="brand"
						:disabled="confirmDisabled"
						@click="handleConfirm(false)"
					>
						<UploadIcon />
						{{ confirmLabel ?? formatMessage(messages.publishButton) }}
					</Button>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	CircleAlertIcon,
	RotateCounterClockwiseIcon,
	SpinnerIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button, type ButtonMenuOption, SplitButton } from '#ui/components/base/buttons'
import FileTreeSelect, { type FileTreeSelectItem } from '#ui/components/base/FileTreeSelect.vue'
import Tabs, { type TabsTab, type TabsValue } from '#ui/components/base/Tabs.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import ContentDiffList from '#ui/layouts/shared/installation-settings/components/ContentDiffList.vue'
import type { ContentDiffItem } from '#ui/layouts/shared/installation-settings/types'
import { commonMessages } from '#ui/utils/common-messages'

import SharedFilesStorageMeter from './SharedFilesStorageMeter.vue'

export type SharedFilesPublishFile = {
	path: string
	size: number
	blocked: boolean
}

const STORAGE_LIMIT_BYTES = 2 * 1024 ** 3

const props = defineProps<{
	diffs: ContentDiffItem[]
	files: SharedFilesPublishFile[]
	changesDescription?: string
	confirmLabel?: string
	restartConfirmLabel?: string
	/** Pushing loads server-side changes, so the primary action also restarts the server. */
	requiresRestart?: boolean
	restartDisabled?: boolean
	/** Allows pushing when nothing changed and no files are selected. */
	allowEmpty?: boolean
	disabled?: boolean
	changesLoading?: boolean
	changesError?: boolean
	filesLoading?: boolean
	filesError?: boolean
}>()

const emit = defineEmits<{
	confirm: [paths: string[], restart: boolean]
	cancel: []
	'retry-changes': []
	'retry-files': []
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const fileTreeContainer = ref<HTMLElement>()
const includedFilePaths = ref<string[]>([])
const excludedFilePaths = ref<string[]>([])
const activeTab = ref<TabsValue>('changes')
const closingFromAction = ref(false)
const tabs = computed<TabsTab[]>(() => [
	{ value: 'changes', label: formatMessage(messages.changesTab) },
	{ value: 'files', label: formatMessage(messages.filesTab) },
])
const fileItems = computed<FileTreeSelectItem[]>(() =>
	props.files.map(({ path, size, blocked }) => ({
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

	return props.files.filter(
		(file) => !file.blocked && isPathSelected(file.path, includedPaths, excludedPaths),
	)
})
const selectedSize = computed(() =>
	selectedFiles.value.reduce((total, file) => total + file.size, 0),
)
const confirmDisabled = computed(
	() =>
		props.disabled ||
		selectedSize.value > STORAGE_LIMIT_BYTES ||
		(!props.allowEmpty && props.diffs.length === 0 && selectedFiles.value.length === 0),
)
const restartOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'push-only',
		label: formatMessage(messages.pushOnlyButton),
		icon: UploadIcon,
		action: () => handleConfirm(false),
	},
])

/** Opens the review. Pass `keepSelection` to reopen it without losing the picked files. */
function show(options?: { event?: MouseEvent; keepSelection?: boolean }) {
	if (!options?.keepSelection) {
		includedFilePaths.value = []
		excludedFilePaths.value = []
		activeTab.value = 'changes'
	}
	modal.value?.show(options?.event)
}

function hide() {
	modal.value?.hide()
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

function handleConfirm(restart: boolean) {
	if (confirmDisabled.value || (restart && props.restartDisabled)) return
	const paths = selectedFiles.value.map((file) => file.path)
	closingFromAction.value = true
	hide()
	emit('confirm', paths, restart)
	closingFromAction.value = false
}

function handleCancel() {
	closingFromAction.value = true
	hide()
	emit('cancel')
	closingFromAction.value = false
}

function handleHide() {
	if (closingFromAction.value) {
		closingFromAction.value = false
		return
	}
	emit('cancel')
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
	pushAndRestartButton: {
		id: 'instance.shared-instance.publish-review.push-and-restart-button',
		defaultMessage: 'Push and restart',
	},
	pushOnlyButton: {
		id: 'instance.shared-instance.publish-review.push-only-button',
		defaultMessage: 'Push update only',
	},
	restartWarning: {
		id: 'instance.shared-instance.publish-review.restart-warning',
		defaultMessage: 'These changes require a server restart to take effect.',
	},
	moreOptions: {
		id: 'instance.shared-instance.publish-review.more-options',
		defaultMessage: 'More push options',
	},
	addedLabel: {
		id: 'instance.shared-instance.publish-review.added-label',
		defaultMessage: 'Added',
	},
	removedLabel: {
		id: 'instance.shared-instance.publish-review.removed-label',
		defaultMessage: 'Removed',
	},
	fileTypeNotAllowed: {
		id: 'instance.shared-instance.publish-review.file-type-not-allowed',
		defaultMessage: "You're not allowed to push this file type.",
	},
	refreshingChanges: {
		id: 'servers.play.refreshing-preview',
		defaultMessage: 'Refreshing changes…',
	},
	changesError: {
		id: 'servers.play.preview-error',
		defaultMessage: 'Could not refresh the changes. Retry before publishing.',
	},
	loadingFiles: {
		id: 'instance.shared-instance.publish-review.loading-files',
		defaultMessage: 'Loading files…',
	},
	filesError: {
		id: 'instance.shared-instance.publish-review.files-error',
		defaultMessage: 'Could not load the files you can share.',
	},
	retry: { id: 'servers.play.retry', defaultMessage: 'Retry' },
})

defineExpose({ show, hide })
</script>
