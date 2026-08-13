<template>
	<ConfirmModal
		ref="deleteModal"
		:title="formatMessage(messages.deleteTitle)"
		:description="
			formatMessage(messages.deleteDescription, {
				name: screenshotToDelete?.file_name ?? '',
			})
		"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		:markdown="false"
		@proceed="confirmDelete"
	/>
	<ConfirmModal
		ref="bulkDeleteModal"
		:title="formatMessage(messages.bulkDeleteTitle)"
		:description="
			formatMessage(messages.bulkDeleteDescription, {
				count: selectedScreenshotNames.size,
			})
		"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		:markdown="false"
		@proceed="deleteSelectedScreenshots"
	/>

	<ImagePreviewModal ref="previewModal" :items="previewItems">
		<template #actions="{ item }">
			<IconButton
				v-tooltip="formatMessage(messages.copy)"
				:label="formatMessage(messages.copy)"
				@click="copyScreenshotByFileName(item.id)"
			>
				<ClipboardCopyIcon />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.showInFolder)"
				:label="formatMessage(messages.showInFolder)"
				@click="openScreenshotByFileName(item.id)"
			>
				<ExternalIcon />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(commonMessages.deleteLabel)"
				:label="formatMessage(commonMessages.deleteLabel)"
				@click="requestPreviewDelete(item.id)"
			>
				<TrashIcon />
			</IconButton>
		</template>
	</ImagePreviewModal>

	<ReadyTransition :pending="screenshotsReadyPending">
		<EmptyState
			v-if="screenshotsError"
			type="error"
			:heading="formatMessage(messages.errorHeading)"
			:description="screenshotsError.message"
		>
			<template #actions>
				<Button type="outlined" @click="screenshotsQuery.refetch()">
					{{ formatMessage(commonMessages.retryButton) }}
				</Button>
			</template>
		</EmptyState>

		<EmptyState
			v-else-if="screenshots.length === 0"
			type="no-images"
			:heading="formatMessage(messages.emptyHeading)"
			:description="formatMessage(messages.emptyDescription)"
		/>

		<div v-else class="flex flex-col gap-3">
			<div
				v-for="(group, groupIndex) in groupedScreenshots"
				:key="group.label"
				class="relative flex flex-col gap-3"
			>
				<div
					class="absolute left-2.5 top-5 w-px bg-surface-5"
					:class="groupIndex === groupedScreenshots.length - 1 ? 'bottom-0' : '-bottom-3'"
				/>
				<div class="relative flex items-center gap-2">
					<div class="flex w-5 shrink-0 items-center justify-center">
						<CalendarIcon class="size-5" />
					</div>
					<span class="text-lg font-semibold leading-5 text-contrast">{{ group.label }}</span>
				</div>

				<div class="grid grid-cols-1 gap-3 pl-7 sm:grid-cols-2 xl:grid-cols-3">
					<div
						v-for="screenshot in group.screenshots"
						:key="screenshot.file_name"
						role="button"
						tabindex="0"
						class="group relative aspect-video cursor-pointer overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-2 p-0 text-left shadow-sm transition hover:border-brand focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
						:class="{ '!border-contrast': selectedScreenshotNames.has(screenshot.file_name) }"
						:aria-label="
							selectionActive
								? formatMessage(
									selectedScreenshotNames.has(screenshot.file_name)
										? messages.deselectScreenshot
										: messages.selectScreenshot,
									{ name: screenshot.file_name },
								)
								: screenshot.file_name
						"
						:aria-pressed="
							selectionActive ? selectedScreenshotNames.has(screenshot.file_name) : undefined
						"
						@click="activateScreenshot(screenshot, $event)"
						@keydown="handleScreenshotKeydown($event, screenshot)"
					>
						<button
							type="button"
							class="selection-button group/selection absolute right-0.5 top-0 z-[2] flex size-[50px] cursor-pointer items-start justify-center border-0 bg-transparent p-0 pt-4"
							:aria-label="
								formatMessage(
									selectedScreenshotNames.has(screenshot.file_name)
										? messages.deselectScreenshot
										: messages.selectScreenshot,
									{ name: screenshot.file_name },
								)
							"
							:aria-pressed="selectedScreenshotNames.has(screenshot.file_name)"
							@click.stop="toggleScreenshotSelection(screenshot.file_name)"
						>
							<span
								class="relative flex size-6 items-center justify-center rounded-full opacity-0 transition-opacity duration-200 ease-out group-hover:opacity-100 group-hover/selection:brightness-125"
								:class="
									selectedScreenshotNames.has(screenshot.file_name)
										? 'border-0 !opacity-100'
										: 'border-2 border-solid border-primary bg-transparent'
								"
							>
								<span
									v-if="selectedScreenshotNames.has(screenshot.file_name)"
									class="absolute inset-0 rounded-full bg-contrast"
								/>
								<CheckIcon
									v-if="selectedScreenshotNames.has(screenshot.file_name)"
									class="relative size-4 invert [stroke-width:3]"
								/>
							</span>
						</button>
						<div
							v-if="!loadedScreenshots.has(screenshot.file_name)"
							class="absolute inset-0 animate-pulse bg-surface-3"
						/>
						<img
							:src="screenshot.url"
							:alt="screenshot.file_name"
							loading="lazy"
							class="h-full w-full object-cover transition duration-200 group-hover:scale-[1.02]"
							:class="loadedScreenshots.has(screenshot.file_name) ? 'opacity-100' : 'opacity-0'"
							@load="markLoaded(screenshot.file_name)"
						/>
						<div
							class="absolute inset-x-0 bottom-0 flex items-end justify-between gap-2 bg-gradient-to-t from-surface-1 to-transparent p-3 pt-[120px] text-contrast opacity-0 transition-opacity duration-200 group-hover:opacity-100 group-focus-within:opacity-100"
						>
							<div class="min-w-0">
								<div v-tooltip="screenshot.file_name" class="truncate text-sm font-semibold">
									{{ screenshot.file_name }}
								</div>
								<div class="text-xs text-secondary">
									{{ formatScreenshotTime(screenshot.created_at) }}
								</div>
							</div>
							<div
								v-if="!selectionActive"
								class="flex shrink-0 translate-y-1 gap-1 opacity-0 transition group-hover:translate-y-0 group-hover:opacity-100 group-focus-within:translate-y-0 group-focus-within:opacity-100"
								@click.stop
							>
								<IconButton
									v-tooltip="formatMessage(messages.copy)"
									:label="formatMessage(messages.copy)"
									type="quiet"
									class="bg-surface-2 text-contrast hover:bg-surface-3"
									@click="copyScreenshot(screenshot)"
								>
									<ClipboardCopyIcon />
								</IconButton>
								<IconButton
									v-tooltip="formatMessage(messages.showInFolder)"
									:label="formatMessage(messages.showInFolder)"
									type="quiet"
									class="bg-surface-2 text-contrast hover:bg-surface-3"
									@click="openScreenshot(screenshot)"
								>
									<ExternalIcon />
								</IconButton>
								<IconButton
									v-tooltip="formatMessage(commonMessages.deleteLabel)"
									:label="formatMessage(commonMessages.deleteLabel)"
									type="quiet"
									class="bg-surface-2 text-contrast hover:bg-surface-3"
									@click="requestDelete(screenshot)"
								>
									<TrashIcon />
								</IconButton>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<FloatingActionBar
			:shown="selectionActive"
			:aria-label="formatMessage(messages.selectionAriaLabel)"
			hide-when-modal-open
		>
			<div class="flex items-center gap-0.5">
				<span class="px-4 py-2.5 text-base font-semibold tabular-nums text-contrast">
					{{ formatMessage(messages.selectedCount, { count: selectedScreenshotNames.size }) }}
				</span>
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<Button type="quiet" :disabled="bulkBusy" @click="clearScreenshotSelection">
					{{ formatMessage(commonMessages.clearButton) }}
				</Button>
			</div>
			<div class="ml-auto flex items-center gap-0.5">
				<Button type="quiet" :disabled="bulkBusy" @click="exportSelectedScreenshots">
					<FileArchiveIcon />
					<span class="bar-label">{{ formatMessage(messages.exportZip) }}</span>
				</Button>
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<Button
					type="quiet"
					color="red"
					interaction="filled"
					:disabled="bulkBusy"
					@click="bulkDeleteModal?.show()"
				>
					<TrashIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</Button>
			</div>
		</FloatingActionBar>
	</ReadyTransition>
</template>

<script setup lang="ts">
import {
	CalendarIcon,
	CheckIcon,
	ClipboardCopyIcon,
	ExternalIcon,
	FileArchiveIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	ConfirmModal,
	defineMessages,
	EmptyState,
	FloatingActionBar,
	IconButton,
	injectNotificationManager,
	ReadyTransition,
	useFormatDateTime,
	useReadyState,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { save } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import dayjs from 'dayjs'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import ImagePreviewModal from '@/components/ui/ImagePreviewModal.vue'
import { instance_listener } from '@/helpers/events.js'
import {
	delete_screenshot,
	export_screenshots,
	open_screenshot,
	type InstanceScreenshot,
} from '@/helpers/instance'

import { injectInstancePage } from '../instance-context'
import { instanceKeys, instanceScreenshotsQueryOptions } from '../query-options'

const messages = defineMessages({
	emptyHeading: {
		id: 'app.instance.screenshots.empty-heading',
		defaultMessage: 'No screenshots yet',
	},
	emptyDescription: {
		id: 'app.instance.screenshots.empty-description',
		defaultMessage: 'Screenshots you take in-game will appear here.',
	},
	errorHeading: {
		id: 'app.instance.screenshots.error-heading',
		defaultMessage: 'Failed to load screenshots',
	},
	copy: {
		id: 'app.instance.screenshots.copy',
		defaultMessage: 'Copy image',
	},
	showInFolder: {
		id: 'app.instance.screenshots.show-in-folder',
		defaultMessage: 'Show in folder',
	},
	deleteTitle: {
		id: 'app.instance.screenshots.delete-title',
		defaultMessage: 'Delete screenshot',
	},
	deleteDescription: {
		id: 'app.instance.screenshots.delete-description',
		defaultMessage: 'Permanently delete {name}? This action cannot be undone.',
	},
	deleteSuccess: {
		id: 'app.instance.screenshots.delete-success',
		defaultMessage: 'Screenshot deleted',
	},
	copySuccess: {
		id: 'app.instance.screenshots.copy-success',
		defaultMessage: 'Screenshot copied to clipboard',
	},
	selectionAriaLabel: {
		id: 'app.instance.screenshots.selection.aria-label',
		defaultMessage: 'Selected screenshots',
	},
	selectedCount: {
		id: 'app.instance.screenshots.selection.selected-count',
		defaultMessage: '{count} selected',
	},
	selectScreenshot: {
		id: 'app.instance.screenshots.selection.select',
		defaultMessage: 'Select {name}',
	},
	deselectScreenshot: {
		id: 'app.instance.screenshots.selection.deselect',
		defaultMessage: 'Deselect {name}',
	},
	exportZip: {
		id: 'app.instance.screenshots.selection.export-zip',
		defaultMessage: 'Export ZIP',
	},
	zipArchive: {
		id: 'app.instance.screenshots.selection.zip-archive',
		defaultMessage: 'ZIP archive',
	},
	exportSuccess: {
		id: 'app.instance.screenshots.selection.export-success',
		defaultMessage: 'Screenshots exported',
	},
	bulkDeleteTitle: {
		id: 'app.instance.screenshots.selection.delete-title',
		defaultMessage: 'Delete selected screenshots',
	},
	bulkDeleteDescription: {
		id: 'app.instance.screenshots.selection.delete-description',
		defaultMessage:
			'Delete {count, plural, one {# screenshot} other {# screenshots}}? This action cannot be undone.',
	},
	bulkDeleteSuccess: {
		id: 'app.instance.screenshots.selection.delete-success',
		defaultMessage: '{count, plural, one {# screenshot deleted} other {# screenshots deleted}}',
	},
	today: {
		id: 'app.instance.screenshots.group.today',
		defaultMessage: 'Today',
	},
	yesterday: {
		id: 'app.instance.screenshots.group.yesterday',
		defaultMessage: 'Yesterday',
	},
	thisWeek: {
		id: 'app.instance.screenshots.group.this-week',
		defaultMessage: 'This week',
	},
	thisMonth: {
		id: 'app.instance.screenshots.group.this-month',
		defaultMessage: 'This month',
	},
})

type ScreenshotGroup = {
	label: string
	screenshots: InstanceScreenshot[]
}

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
const instancePage = injectInstancePage()
const queryClient = useQueryClient()
const instanceId = instancePage.instanceId
const previewModal = ref<InstanceType<typeof ImagePreviewModal>>()
const deleteModal = ref<InstanceType<typeof ConfirmModal>>()
const bulkDeleteModal = ref<InstanceType<typeof ConfirmModal>>()
const screenshotToDelete = ref<InstanceScreenshot | null>(null)
const deleteFromPreview = ref(false)
const loadedScreenshots = ref(new Set<string>())
const selectedScreenshotNames = ref(new Set<string>())
const bulkDeleting = ref(false)
const bulkExporting = ref(false)

const screenshotsQuery = useQuery(computed(() => instanceScreenshotsQueryOptions(instanceId.value)))
const screenshotsReadyPending = useReadyState(screenshotsQuery)
const screenshots = computed(() => screenshotsQuery.data.value ?? [])
const selectionActive = computed(() => selectedScreenshotNames.value.size > 0)
const bulkBusy = computed(() => bulkDeleting.value || bulkExporting.value)
const selectedScreenshots = computed(() =>
	screenshots.value.filter((screenshot) =>
		selectedScreenshotNames.value.has(screenshot.file_name),
	),
)
const screenshotsError = computed(() => {
	const error = screenshotsQuery.error.value
	return error instanceof Error ? error : error ? new Error(String(error)) : null
})
const formatScreenshotDate = useFormatDateTime({ dateStyle: 'long', timeStyle: 'short' })
const formatScreenshotTime = useFormatDateTime({ hour: 'numeric', minute: '2-digit' })
const formatScreenshotMonth = useFormatDateTime({ month: 'long', year: 'numeric' })
const previewItems = computed(() =>
	screenshots.value.map((screenshot) => ({
		id: screenshot.file_name,
		src: screenshot.url,
		alt: screenshot.file_name,
		title: screenshot.file_name,
		description: formatScreenshotDate(screenshot.created_at),
	})),
)

const groupedScreenshots = computed((): ScreenshotGroup[] => {
	const now = dayjs()
	const groups: ScreenshotGroup[] = []

	function addToGroup(label: string, screenshot: InstanceScreenshot) {
		let group = groups.find((candidate) => candidate.label === label)
		if (!group) {
			group = { label, screenshots: [] }
			groups.push(group)
		}
		group.screenshots.push(screenshot)
	}

	for (const screenshot of screenshots.value) {
		const created = dayjs(screenshot.created_at)
		const isToday = created.isSame(now, 'day')
		const isYesterday = created.isSame(now.subtract(1, 'day'), 'day')

		if (isToday) {
			addToGroup(formatMessage(messages.today), screenshot)
		} else if (isYesterday) {
			addToGroup(formatMessage(messages.yesterday), screenshot)
		} else if (created.isSame(now, 'week')) {
			addToGroup(formatMessage(messages.thisWeek), screenshot)
		} else if (created.isSame(now, 'month')) {
			addToGroup(formatMessage(messages.thisMonth), screenshot)
		} else {
			addToGroup(formatScreenshotMonth(created.toDate()), screenshot)
		}
	}

	return groups
})

watch(screenshots, (currentScreenshots) => {
	const currentNames = new Set(currentScreenshots.map((screenshot) => screenshot.file_name))
	selectedScreenshotNames.value = new Set(
		[...selectedScreenshotNames.value].filter((fileName) => currentNames.has(fileName)),
	)
})

function toggleScreenshotSelection(fileName: string) {
	if (bulkBusy.value) return
	const nextSelection = new Set(selectedScreenshotNames.value)
	if (nextSelection.has(fileName)) {
		nextSelection.delete(fileName)
	} else {
		nextSelection.add(fileName)
	}
	selectedScreenshotNames.value = nextSelection
}

function activateScreenshot(screenshot: InstanceScreenshot, event?: MouseEvent) {
	if (selectionActive.value || event?.shiftKey) {
		toggleScreenshotSelection(screenshot.file_name)
	} else {
		showPreview(screenshot)
	}
}

function handleScreenshotKeydown(event: KeyboardEvent, screenshot: InstanceScreenshot) {
	if (event.target !== event.currentTarget) return
	if (event.key !== 'Enter' && event.key !== ' ') return

	event.preventDefault()
	activateScreenshot(screenshot)
}

function clearScreenshotSelection() {
	if (bulkBusy.value) return
	selectedScreenshotNames.value = new Set()
}

function markLoaded(fileName: string) {
	loadedScreenshots.value.add(fileName)
}

function showPreview(screenshot: InstanceScreenshot) {
	const index = screenshots.value.findIndex((item) => item.file_name === screenshot.file_name)
	if (index >= 0) previewModal.value?.show(index)
}

function requestDelete(screenshot: InstanceScreenshot) {
	deleteFromPreview.value = false
	screenshotToDelete.value = screenshot
	deleteModal.value?.show()
}

function screenshotByFileName(fileName: string) {
	return screenshots.value.find((screenshot) => screenshot.file_name === fileName)
}

function requestPreviewDelete(fileName: string) {
	const screenshot = screenshotByFileName(fileName)
	if (!screenshot) return
	deleteFromPreview.value = true
	screenshotToDelete.value = screenshot
	deleteModal.value?.show()
}

async function confirmDelete() {
	const screenshot = screenshotToDelete.value
	if (!screenshot) return

	try {
		await delete_screenshot(instanceId.value, screenshot.file_name)
		if (deleteFromPreview.value) previewModal.value?.hide()
		await queryClient.invalidateQueries({ queryKey: instanceKeys.screenshots(instanceId.value) })
		addNotification({ type: 'success', title: formatMessage(messages.deleteSuccess) })
	} catch (error) {
		handleError(error)
	} finally {
		screenshotToDelete.value = null
		deleteFromPreview.value = false
	}
}

async function deleteSelectedScreenshots() {
	const selected = selectedScreenshots.value
	if (bulkBusy.value || selected.length === 0) return

	bulkDeleting.value = true
	try {
		const results = await Promise.allSettled(
			selected.map((screenshot) => delete_screenshot(instanceId.value, screenshot.file_name)),
		)
		const deletedNames = new Set<string>()

		for (const [index, result] of results.entries()) {
			if (result.status === 'fulfilled') {
				deletedNames.add(selected[index].file_name)
			} else {
				handleError(result.reason)
			}
		}

		if (deletedNames.size > 0) {
			selectedScreenshotNames.value = new Set(
				[...selectedScreenshotNames.value].filter((fileName) => !deletedNames.has(fileName)),
			)
			await queryClient.invalidateQueries({
				queryKey: instanceKeys.screenshots(instanceId.value),
			})
			addNotification({
				type: 'success',
				title: formatMessage(messages.bulkDeleteSuccess, { count: deletedNames.size }),
			})
		}
	} finally {
		bulkDeleting.value = false
	}
}

async function exportSelectedScreenshots() {
	const selected = selectedScreenshots.value
	if (bulkBusy.value || selected.length === 0) return

	try {
		const instanceName = instancePage.instance.value.name.replace(/[\\/:*?"<>|]/g, '-')
		const outputPath = await save({
			defaultPath: `${instanceName} screenshots.zip`,
			filters: [
				{
					name: formatMessage(messages.zipArchive),
					extensions: ['zip'],
				},
			],
		})
		if (!outputPath) return

		bulkExporting.value = true
		await export_screenshots(
			instanceId.value,
			selected.map((screenshot) => screenshot.file_name),
			outputPath,
		)
		addNotification({ type: 'success', title: formatMessage(messages.exportSuccess) })
	} catch (error) {
		handleError(error)
	} finally {
		bulkExporting.value = false
	}
}

function copyScreenshotByFileName(fileName: string) {
	const screenshot = screenshotByFileName(fileName)
	if (screenshot) void copyScreenshot(screenshot)
}

async function copyScreenshot(screenshot: InstanceScreenshot) {
	try {
		const png = readFile(screenshot.path).then((bytes) => new Blob([bytes], { type: 'image/png' }))
		await navigator.clipboard.write([new ClipboardItem({ 'image/png': png })])
		addNotification({ type: 'success', title: formatMessage(messages.copySuccess) })
	} catch (error) {
		handleError(error)
	}
}

async function openScreenshot(screenshot: InstanceScreenshot) {
	try {
		await open_screenshot(instanceId.value, screenshot.file_name)
	} catch (error) {
		handleError(error)
	}
}

function openScreenshotByFileName(fileName: string) {
	const screenshot = screenshotByFileName(fileName)
	if (screenshot) void openScreenshot(screenshot)
}

const unlistenInstance = await instance_listener(
	(event: { instance_id: string; event: string }) => {
		if (event.instance_id !== instanceId.value || event.event !== 'screenshots_updated') return
		void queryClient.invalidateQueries({ queryKey: instanceKeys.screenshots(instanceId.value) })
	},
)

onBeforeUnmount(() => {
	unlistenInstance()
})
</script>
