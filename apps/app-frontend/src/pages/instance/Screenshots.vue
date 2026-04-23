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
					<div class="flex flex-col gap-1 p-4">
						<div class="flex items-center justify-between gap-2">
							<div class="min-w-0 truncate font-semibold text-contrast" :title="screenshot.name">
								{{ screenshot.name }}
							</div>
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
						<div class="text-sm text-secondary">
							{{ formatDateTime(screenshot.modifiedAt) }}
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
				<div class="min-w-0">
					<div class="truncate font-semibold text-contrast" :title="activeScreenshot.name">
						{{ activeScreenshot.name }}
					</div>
					<div class="text-sm text-secondary">
						{{ formatDateTime(activeScreenshot.modifiedAt) }}
					</div>
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
import { readDir, readFile, remove, stat } from '@tauri-apps/plugin-fs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { get_full_path } from '@/helpers/profile'
import type { GameInstance } from '@/helpers/types'
import { highlightInFolder, openPath } from '@/helpers/utils'

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
})

const loading = ref(true)
const error = ref<string | null>(null)
const screenshots = ref<ScreenshotItem[]>([])
const screenshotsDir = ref('')
const activeIndex = ref<number | null>(null)

const activeScreenshot = computed(() =>
	activeIndex.value == null ? null : (screenshots.value[activeIndex.value] ?? null),
)

function getExtension(fileName: string): string {
	return fileName.split('.').pop()?.toLowerCase() ?? ''
}

function getImageMimeType(fileName: string): string {
	const extension = getExtension(fileName)
	switch (extension) {
		case 'jpg':
		case 'jpeg':
			return 'image/jpeg'
		case 'gif':
			return 'image/gif'
		case 'svg':
			return 'image/svg+xml'
		case 'webp':
			return 'image/webp'
		default:
			return 'image/png'
	}
}

function isMissingPathError(err: unknown): boolean {
	const message = err instanceof Error ? err.message : String(err)
	return /not found|cannot find|does not exist/i.test(message)
}

function revokeScreenshotUrls(items: ScreenshotItem[]) {
	for (const item of items) {
		URL.revokeObjectURL(item.url)
	}
}

async function loadScreenshots() {
	loading.value = true
	error.value = null
	activeIndex.value = null

	try {
		revokeScreenshotUrls(screenshots.value)

		const instanceRoot = await get_full_path(props.instance.path)
		screenshotsDir.value = `${instanceRoot}/screenshots`

		const entries = await readDir(screenshotsDir.value)
		const nextScreenshots = await Promise.all(
			entries
				.filter((entry) => !entry.isDirectory && IMAGE_EXTENSIONS.has(getExtension(entry.name)))
				.map(async (entry) => {
					const absolutePath = `${screenshotsDir.value}/${entry.name}`
					const bytes = await readFile(absolutePath)
					const metadata = await stat(absolutePath)
					const modifiedAt = metadata.mtime ?? metadata.birthtime ?? new Date(0)
					return {
						name: entry.name,
						path: absolutePath,
						relativePath: `screenshots/${entry.name}`,
						url: URL.createObjectURL(
							new Blob([bytes], {
								type: getImageMimeType(entry.name),
							}),
						),
						modifiedAt,
					} satisfies ScreenshotItem
				}),
		)

		screenshots.value = nextScreenshots.sort(
			(a, b) => b.modifiedAt.getTime() - a.modifiedAt.getTime(),
		)
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
		await remove(screenshot.path)
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
	if (!screenshotsDir.value) {
		const instanceRoot = await get_full_path(props.instance.path)
		screenshotsDir.value = `${instanceRoot}/screenshots`
	}
	await openPath(screenshotsDir.value)
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
	revokeScreenshotUrls(screenshots.value)
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
