<template>
	<ReadyTransition :pending="loading">
		<div class="flex flex-col gap-4">
			<div class="flex items-center justify-between gap-2">
				<div class="flex flex-col gap-1">
					<h2 class="text-xl font-extrabold text-contrast">Screenshots</h2>
					<p class="text-sm text-secondary">
						{{
							screenshots.length > 0
								? `${screenshots.length} image${screenshots.length === 1 ? '' : 's'}`
								: 'Images saved by this profile appear here.'
						}}
					</p>
				</div>
				<ButtonStyled type="outlined">
					<button class="!h-10" @click="openScreenshotsFolder">
						<FolderOpenIcon class="size-5" />
						Open folder
					</button>
				</ButtonStyled>
			</div>

			<EmptyState
				v-if="error"
				type="error"
				heading="Could not load screenshots"
				:description="error"
			/>

			<EmptyState
				v-else-if="screenshots.length === 0"
				type="no-images"
				heading="No screenshots yet"
				description="Take a screenshot in-game and it will show up here."
			>
				<template #actions>
					<ButtonStyled type="outlined">
						<button class="!h-10" @click="openScreenshotsFolder">
							<FolderOpenIcon class="size-5" />
							Open folder
						</button>
					</ButtonStyled>
				</template>
			</EmptyState>

			<div v-else class="screenshots-grid">
				<Card
					v-for="(screenshot, index) in screenshots"
					:key="screenshot.path"
					class="screenshot-card"
				>
					<button class="screenshot-preview" @click="expandImage(index)">
						<img
							:src="screenshot.url"
							:alt="screenshot.name"
							class="screenshot-image"
							loading="lazy"
						/>
					</button>
					<div class="flex items-center justify-between gap-2 p-4">
						<div class="min-w-0 text-sm text-secondary">
							{{ formatDateTime(screenshot.modifiedAt) }}
						</div>
						<div class="flex items-center gap-1">
							<ButtonStyled circular type="transparent">
								<button
									:aria-label="formatMessage(messages.copyImage)"
									:title="formatMessage(messages.copyImage)"
									@click="copyScreenshotImage(screenshot)"
								>
									<CopyIcon class="size-5" />
								</button>
							</ButtonStyled>
							<ButtonStyled circular type="transparent">
								<TeleportOverflowMenu :options="getScreenshotMenuOptions(screenshot)">
									<MoreHorizontalIcon class="size-5" />
									<template #copy-filename>
										<ClipboardCopyIcon />
										{{ formatMessage(commonMessages.copyFilenameButton) }}
									</template>
									<template #copy-full-path>
										<ClipboardCopyIcon />
										{{ formatMessage(commonMessages.copyFullPathButton) }}
									</template>
									<template #open-in-folder>
										<FolderOpenIcon />
										{{ formatMessage(commonMessages.openInFolderButton) }}
									</template>
									<template #save-as>
										<DownloadIcon />
										{{ formatMessage(messages.saveAs) }}
									</template>
									<template #delete>
										<TrashIcon />
										{{ formatMessage(commonMessages.deleteLabel) }}
									</template>
								</TeleportOverflowMenu>
							</ButtonStyled>
						</div>
					</div>
				</Card>
			</div>
		</div>
	</ReadyTransition>

	<div v-if="activeScreenshot" class="expanded-image-modal" @click="hideImage">
		<div class="modal-content" @click.stop>
			<img
				class="modal-image"
				:src="activeScreenshot.url"
				:alt="activeScreenshot.name"
				@click.stop
			/>
			<div class="modal-toolbar">
				<div class="text-sm text-secondary">
					{{ formatDateTime(activeScreenshot.modifiedAt) }}
				</div>
				<div class="flex items-center gap-2">
					<ButtonStyled v-if="screenshots.length > 1" circular type="transparent">
						<button aria-label="Previous screenshot" @click="previousImage">
							<LeftArrowIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="screenshots.length > 1" circular type="transparent">
						<button aria-label="Next screenshot" @click="nextImage">
							<RightArrowIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled circular type="transparent">
						<button aria-label="Reveal screenshot in folder" @click="revealActiveScreenshot">
							<FolderOpenIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled circular type="transparent">
						<button aria-label="Close screenshot preview" @click="hideImage">
							<XIcon />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ClipboardCopyIcon,
	CopyIcon,
	DownloadIcon,
	FolderOpenIcon,
	LeftArrowIcon,
	MoreHorizontalIcon,
	RightArrowIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Card,
	commonMessages,
	defineMessages,
	EmptyState,
	injectNotificationManager,
	ReadyTransition,
	TeleportOverflowMenu,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import {
	delete_screenshot,
	get_screenshot_bytes,
	get_screenshots,
	open_screenshots_folder,
} from '@/helpers/profile'
import type { GameInstance } from '@/helpers/types'
import { highlightInFolder } from '@/helpers/utils'

type ScreenshotItem = {
	name: string
	path: string
	relativePath: string
	url: string
	modifiedAt: Date
}

const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'])

const props = defineProps<{
	instance: GameInstance
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'medium',
})
const messages = defineMessages({
	saveAs: {
		id: 'instance.screenshots.save-as',
		defaultMessage: 'Save as...',
	},
	copyImage: {
		id: 'instance.screenshots.copy-image',
		defaultMessage: 'Copy image',
	},
	copiedImage: {
		id: 'instance.screenshots.copied-image',
		defaultMessage: 'Copied image',
	},
	copyImageFailed: {
		id: 'instance.screenshots.copy-image-failed',
		defaultMessage: 'Failed to copy image',
	},
})

const loading = ref(true)
const error = ref<string | null>(null)
const screenshots = ref<ScreenshotItem[]>([])
const activeIndex = ref<number | null>(null)

const activeScreenshot = computed(() =>
	activeIndex.value == null ? null : (screenshots.value[activeIndex.value] ?? null),
)

function getExtension(fileName: string): string {
	return fileName.split('.').pop()?.toLowerCase() ?? ''
}

function getImageMimeType(fileName: string): string {
	const extension = getExtension(fileName)
	if (extension === 'jpg' || extension === 'jpeg') {
		return 'image/jpeg'
	}
	if (extension === 'svg') {
		return 'image/svg+xml'
	}
	return `image/${extension || 'png'}`
}

function isMissingPathError(err: unknown): boolean {
	const message = err instanceof Error ? err.message : String(err)
	return /not found|cannot find|does not exist/i.test(message)
}

async function loadScreenshots() {
	loading.value = true
	error.value = null
	activeIndex.value = null

	try {
		const entries = await get_screenshots(props.instance.path)
		screenshots.value = entries
			.filter((entry) => IMAGE_EXTENSIONS.has(getExtension(entry.name)))
			.map((entry) => ({
				name: entry.name,
				path: entry.absolutePath,
				relativePath: entry.relativePath,
				url: entry.url,
				modifiedAt: new Date(entry.modifiedAtMs),
			}))
	} catch (err) {
		if (isMissingPathError(err)) {
			screenshots.value = []
			error.value = null
		} else {
			screenshots.value = []
			error.value = err instanceof Error ? err.message : String(err)
		}
	} finally {
		loading.value = false
	}
}

async function copyScreenshotFilename(screenshot: ScreenshotItem) {
	await navigator.clipboard.writeText(screenshot.name)
	addNotification({
		title: formatMessage(commonMessages.copiedFilenameLabel),
		type: 'success',
	})
}

async function copyScreenshotPath(screenshot: ScreenshotItem) {
	await navigator.clipboard.writeText(screenshot.path)
	addNotification({
		title: formatMessage(commonMessages.copiedPathLabel),
		type: 'success',
	})
}

async function copyScreenshotImage(screenshot: ScreenshotItem) {
	let objectUrl: string | null = null

	try {
		if (typeof ClipboardItem === 'undefined' || !navigator.clipboard?.write) {
			throw new Error('Image clipboard is not available in this app session.')
		}

		const screenshotBytes = await get_screenshot_bytes(props.instance.path, screenshot.name)
		const sourceBlob = new Blob([new Uint8Array(screenshotBytes)], {
			type: getImageMimeType(screenshot.name),
		})
		objectUrl = URL.createObjectURL(sourceBlob)
		const image = await new Promise<HTMLImageElement>((resolve, reject) => {
			const element = new Image()
			element.onload = () => resolve(element)
			element.onerror = () => reject(new Error('Failed to load screenshot for clipboard copy.'))
			element.src = objectUrl!
		})
		const canvas = document.createElement('canvas')
		canvas.width = image.naturalWidth
		canvas.height = image.naturalHeight
		const context = canvas.getContext('2d')
		if (!context) {
			throw new Error('Failed to initialize an image canvas.')
		}

		context.drawImage(image, 0, 0)
		const blob = await new Promise<Blob>((resolve, reject) => {
			canvas.toBlob((value) => {
				if (value) {
					resolve(value)
				} else {
					reject(new Error('Failed to convert screenshot for clipboard copy.'))
				}
			}, 'image/png')
		})
		const clipboardItem = new ClipboardItem({
			'image/png': blob,
		})
		await navigator.clipboard.write([clipboardItem])
		addNotification({
			title: formatMessage(messages.copiedImage),
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: formatMessage(messages.copyImageFailed),
			text: err instanceof Error ? err.message : String(err),
			type: 'error',
		})
	} finally {
		if (objectUrl) {
			URL.revokeObjectURL(objectUrl)
		}
	}
}

async function revealScreenshot(screenshot: ScreenshotItem) {
	await highlightInFolder(screenshot.path)
}

async function saveScreenshotAs(screenshot: ScreenshotItem) {
	try {
		await invoke('plugin:files|file_save_as', {
			instancePath: props.instance.path,
			filePath: screenshot.relativePath,
		})
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.downloadFailedLabel),
			text: err instanceof Error ? err.message : String(err),
			type: 'error',
		})
	}
}

async function deleteScreenshot(screenshot: ScreenshotItem) {
	try {
		if (activeScreenshot.value?.path === screenshot.path) {
			hideImage()
		}
		await delete_screenshot(props.instance.path, screenshot.name)
		await loadScreenshots()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.deleteFailedLabel),
			text: err instanceof Error ? err.message : String(err),
			type: 'error',
		})
	}
}

function getScreenshotMenuOptions(screenshot: ScreenshotItem) {
	return [
		{
			id: 'copy-filename',
			icon: ClipboardCopyIcon,
			action: () => copyScreenshotFilename(screenshot),
		},
		{
			id: 'copy-full-path',
			icon: ClipboardCopyIcon,
			action: () => copyScreenshotPath(screenshot),
		},
		{
			id: 'open-in-folder',
			icon: FolderOpenIcon,
			action: () => revealScreenshot(screenshot),
		},
		{ divider: true },
		{
			id: 'save-as',
			icon: DownloadIcon,
			action: () => saveScreenshotAs(screenshot),
		},
		{ divider: true },
		{
			id: 'delete',
			icon: TrashIcon,
			action: () => deleteScreenshot(screenshot),
			color: 'red' as const,
		},
	]
}

function expandImage(index: number) {
	activeIndex.value = index
}

function hideImage() {
	activeIndex.value = null
}

function nextImage() {
	if (activeIndex.value == null || screenshots.value.length === 0) return
	activeIndex.value = (activeIndex.value + 1) % screenshots.value.length
}

function previousImage() {
	if (activeIndex.value == null || screenshots.value.length === 0) return
	activeIndex.value = (activeIndex.value - 1 + screenshots.value.length) % screenshots.value.length
}

async function revealActiveScreenshot() {
	if (!activeScreenshot.value) return
	await highlightInFolder(activeScreenshot.value.path)
}

async function openScreenshotsFolder() {
	await open_screenshots_folder(props.instance.path)
}

function handleKeydown(event: KeyboardEvent) {
	if (!activeScreenshot.value) return
	if (event.key === 'Escape') {
		event.preventDefault()
		hideImage()
	} else if (event.key === 'ArrowLeft') {
		event.preventDefault()
		previousImage()
	} else if (event.key === 'ArrowRight') {
		event.preventDefault()
		nextImage()
	}
}

watch(
	() => props.instance.path,
	async () => {
		await loadScreenshots()
	},
)

onMounted(async () => {
	document.addEventListener('keydown', handleKeydown)
	await loadScreenshots()
})

onUnmounted(() => {
	document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped lang="scss">
.screenshots-grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
	gap: 1rem;
}

.screenshot-card {
	padding: 0;
	overflow: hidden;
}

.screenshot-preview {
	display: block;
	width: 100%;
	background: transparent;
	padding: 0;
	border: 0;
	cursor: pointer;
}

.screenshot-image {
	display: block;
	width: 100%;
	aspect-ratio: 16 / 9;
	object-fit: cover;
	background: var(--color-bg-secondary);
}

.expanded-image-modal {
	position: fixed;
	inset: 0;
	z-index: 30;
	display: flex;
	align-items: center;
	justify-content: center;
	padding: 1.5rem;
	background: rgb(0 0 0 / 80%);
}

.modal-content {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	width: min(90vw, 90rem);
	max-height: calc(100vh - 3rem);
}

.modal-image {
	max-width: 100%;
	max-height: calc(100vh - 8rem);
	object-fit: contain;
	border-radius: 1rem;
	background: rgb(0 0 0 / 30%);
}

.modal-toolbar {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
	padding: 0.75rem 1rem;
	border-radius: 999px;
	background: rgb(18 18 18 / 88%);
	backdrop-filter: blur(12px);
}

@media (max-width: 640px) {
	.modal-toolbar {
		flex-direction: column;
		align-items: stretch;
		border-radius: 1rem;
	}
}
</style>
