<template>
	<div
		ref="editorContainer"
		class="relative flex flex-col overflow-hidden rounded-[20px] border border-solid border-surface-4 shadow-sm"
	>
		<Transition name="search-bar">
			<div
				v-if="isSearchOpen && !isEditingImage"
				class="absolute right-3 top-3 z-10 flex flex-col gap-1 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-1.5 shadow-lg"
				@keydown.escape.stop="closeSearch"
			>
				<!-- Find row -->
				<div class="flex items-center gap-1">
					<Button
						v-tooltip="formatMessage(messages.toggleReplace)"
						icon-only
						transparent
						:aria-label="formatMessage(messages.toggleReplace)"
						@click="toggleReplace"
					>
						<ChevronRightIcon
							class="transition-transform duration-150"
							:class="{ 'rotate-90': isReplaceOpen }"
						/>
					</Button>
					<div
						@keydown.enter.prevent.stop="findNext"
						@keydown.shift.enter.prevent.stop="findPrevious"
					>
						<StyledInput
							ref="searchInputRef"
							v-model="inFileSearchQuery"
							:icon="SearchIcon"
							type="search"
							size="small"
							autocomplete="off"
							:placeholder="formatMessage(messages.searchInFile)"
							wrapper-class="w-44"
						/>
					</div>
					<span class="min-w-[6rem] px-1 text-right text-sm text-secondary tabular-nums">
						{{
							searchMatchCount > 0
								? formatMessage(messages.matchCount, {
										current: currentSearchMatch,
										total: searchMatchCount,
									})
								: inFileSearchQuery
									? formatMessage(messages.noResults)
									: ''
						}}
					</span>
					<Button
						v-tooltip="formatMessage(messages.previousMatch)"
						icon-only
						transparent
						:disabled="searchMatchCount === 0"
						:aria-label="formatMessage(messages.previousMatch)"
						@click="findPrevious"
					>
						<ChevronUpIcon />
					</Button>
					<Button
						v-tooltip="formatMessage(messages.nextMatch)"
						icon-only
						transparent
						:disabled="searchMatchCount === 0"
						:aria-label="formatMessage(messages.nextMatch)"
						@click="findNext"
					>
						<ChevronDownIcon />
					</Button>
					<div class="mx-0.5 h-4 w-px bg-surface-5" />
					<Button
						v-tooltip="formatMessage(messages.closeSearch)"
						icon-only
						transparent
						:aria-label="formatMessage(messages.closeSearch)"
						@click="closeSearch"
					>
						<XIcon />
					</Button>
				</div>

				<!-- Replace row -->
				<div v-if="isReplaceOpen" class="flex items-center gap-1">
					<div class="w-8 flex-shrink-0" />
					<div @keydown.enter.prevent.stop="replaceOne">
						<StyledInput
							ref="replaceInputRef"
							v-model="replaceQuery"
							type="text"
							size="small"
							autocomplete="off"
							:placeholder="formatMessage(messages.replaceInFile)"
							wrapper-class="w-44"
						/>
					</div>
					<ButtonStyled type="outlined">
						<button
							class="!h-8 whitespace-nowrap !border !border-surface-5 px-2 text-sm disabled:opacity-50"
							:disabled="searchMatchCount === 0"
							@click="replaceOne"
						>
							{{ formatMessage(messages.replace) }}
						</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button
							class="!h-8 whitespace-nowrap !border !border-surface-5 px-2 text-sm disabled:opacity-50"
							:disabled="searchMatchCount === 0"
							@click="replaceAllOccurrences"
						>
							{{ formatMessage(messages.replaceAll) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</Transition>
		<component
			:is="props.editorComponent"
			v-if="!isEditingImage && !isLoading && props.editorComponent"
			v-model:value="fileContent"
			:lang="editorLanguage"
			theme="modrinth"
			:print-margin="false"
			:style="{ height: editorHeight, fontSize: '0.875rem' }"
			class="ace-modrinth rounded-[20px]"
			@init="onEditorInit"
		/>
		<FileImageViewer v-else-if="isEditingImage && imagePreview" :image-blob="imagePreview" />
		<div
			v-else-if="isLoading || !props.editorComponent"
			class="flex items-center justify-center rounded-[20px] bg-bg-raised"
			:style="{ height: editorHeight }"
		>
			<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronDownIcon,
	ChevronRightIcon,
	ChevronUpIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { type Component, computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import Button from '#ui/components/base/Button.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { getEditorLanguage, getFileExtension, isImageFile } from '#ui/utils/file-extensions'

import { injectFileManager } from '../../providers/file-manager'
import type { EditingFile } from '../../types'
import FileImageViewer from './FileImageViewer.vue'

interface MclogsResponse {
	success: boolean
	url?: string
	error?: string
}

interface AceEditorInstance {
	commands: {
		addCommand: (cmd: {
			name: string
			bindKey: { win: string; mac: string }
			exec: () => void
		}) => void
	}
	find: (
		needle: string,
		options?: {
			backwards?: boolean
			wrap?: boolean
			caseSensitive?: boolean
			wholeWord?: boolean
			regExp?: boolean
		},
		animate?: boolean,
	) => unknown
	findNext: (options?: object, animate?: boolean) => void
	findPrevious: (options?: object, animate?: boolean) => void
	replace: (replacement: string) => void
	replaceAll: (replacement: string) => void
	focus: () => void
}

const props = defineProps<{
	file: EditingFile | null
	editorComponent: Component | null
}>()

const emit = defineEmits<{
	close: []
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()

const messages = defineMessages({
	failedToOpenTitle: {
		id: 'files.editor.failed-to-open-title',
		defaultMessage: 'Failed to open file',
	},
	failedToOpenText: {
		id: 'files.editor.failed-to-open-text',
		defaultMessage: 'Could not load file contents.',
	},
	fileSavedTitle: {
		id: 'files.editor.file-saved-title',
		defaultMessage: 'File saved',
	},
	fileSavedText: {
		id: 'files.editor.file-saved-text',
		defaultMessage: 'Your file has been saved.',
	},
	saveFailedTitle: {
		id: 'files.editor.save-failed-title',
		defaultMessage: 'Save failed',
	},
	saveFailedText: {
		id: 'files.editor.save-failed-text',
		defaultMessage: 'Could not save the file.',
	},
	logUrlCopiedTitle: {
		id: 'files.editor.log-url-copied-title',
		defaultMessage: 'Log URL copied',
	},
	logUrlCopiedText: {
		id: 'files.editor.log-url-copied-text',
		defaultMessage: 'Your log file URL has been copied to your clipboard.',
	},
	failedToShareTitle: {
		id: 'files.editor.failed-to-share-title',
		defaultMessage: 'Failed to share file',
	},
	failedToShareText: {
		id: 'files.editor.failed-to-share-text',
		defaultMessage: 'Could not upload to mclo.gs.',
	},
	searchInFile: {
		id: 'files.editor.search-in-file',
		defaultMessage: 'Search...',
	},
	matchCount: {
		id: 'files.editor.search-match-count',
		defaultMessage: '{current} of {total}',
	},
	noResults: {
		id: 'files.editor.search-no-results',
		defaultMessage: 'No results',
	},
	previousMatch: {
		id: 'files.editor.search-previous-match',
		defaultMessage: 'Previous match',
	},
	nextMatch: {
		id: 'files.editor.search-next-match',
		defaultMessage: 'Next match',
	},
	closeSearch: {
		id: 'files.editor.search-close',
		defaultMessage: 'Close search',
	},
	toggleReplace: {
		id: 'files.editor.search-toggle-replace',
		defaultMessage: 'Toggle replace',
	},
	replaceInFile: {
		id: 'files.editor.replace-in-file',
		defaultMessage: 'Replace...',
	},
	replace: {
		id: 'files.editor.replace',
		defaultMessage: 'Replace',
	},
	replaceAll: {
		id: 'files.editor.replace-all',
		defaultMessage: 'Replace All',
	},
})

const fileContent = ref('')
const originalContent = ref('')
const isEditingImage = ref(false)
const imagePreview = ref<Blob | null>(null)
const isLoading = ref(false)
const editorInstance = ref<AceEditorInstance | null>(null)
const editorContainer = ref<HTMLElement | null>(null)
const editorHeight = ref('300px')

const isSearchOpen = ref(false)
const isReplaceOpen = ref(false)
const inFileSearchQuery = ref('')
const replaceQuery = ref('')
const searchMatchCount = ref(0)
const currentSearchMatch = ref(0)
const searchInputRef = ref<{ focus: () => void } | null>(null)
const replaceInputRef = ref<{ focus: () => void } | null>(null)

watch(inFileSearchQuery, handleSearchInput)

function updateEditorHeight() {
	if (editorContainer.value) {
		const top = editorContainer.value.getBoundingClientRect().top
		const padding = 24
		editorHeight.value = `${Math.max(300, window.innerHeight - top - padding)}px`
	}
}

onMounted(() => {
	nextTick(updateEditorHeight)
	window.addEventListener('resize', updateEditorHeight)
})

const editorLanguage = computed(() => {
	const ext = getFileExtension(props.file?.name ?? '')
	return getEditorLanguage(ext)
})

watch(
	() => props.file,
	async (newFile) => {
		if (newFile) {
			closeSearch()
			await loadFileContent(newFile)
			nextTick(updateEditorHeight)
		} else {
			resetState()
		}
	},
	{ immediate: true },
)

async function loadFileContent(file: { name: string; path: string }) {
	isLoading.value = true
	try {
		window.scrollTo(0, 0)
		const extension = getFileExtension(file.name)
		const normalizedPath = file.path.startsWith('/') ? file.path : `/${file.path}`

		if (isImageFile(extension)) {
			const content = await ctx.readFileAsBlob(normalizedPath)
			isEditingImage.value = true
			imagePreview.value = content
		} else {
			isEditingImage.value = false
			const content = await ctx.readFile(normalizedPath)
			fileContent.value = content
			originalContent.value = content
		}
	} catch (error) {
		console.error('Error fetching file content:', error)
		addNotification({
			title: formatMessage(messages.failedToOpenTitle),
			text: formatMessage(messages.failedToOpenText),
			type: 'error',
		})
		emit('close')
	} finally {
		isLoading.value = false
	}
}

const hasUnsavedChanges = computed(
	() => !isEditingImage.value && !isLoading.value && fileContent.value !== originalContent.value,
)

function revertChanges() {
	fileContent.value = originalContent.value
}

function resetState() {
	fileContent.value = ''
	originalContent.value = ''
	isEditingImage.value = false
	imagePreview.value = null
}

function onEditorInit(editor: AceEditorInstance) {
	editorInstance.value = editor

	editor.commands.addCommand({
		name: 'save',
		bindKey: { win: 'Ctrl-S', mac: 'Command-S' },
		exec: () => saveFileContent(false),
	})

	editor.commands.addCommand({
		name: 'find',
		bindKey: { win: 'Ctrl-F', mac: 'Command-F' },
		exec: () => toggleSearch(),
	})

	editor.commands.addCommand({
		name: 'replace',
		bindKey: { win: 'Ctrl-H', mac: 'Command-Option-F' },
		exec: () => {
			isSearchOpen.value = true
			isReplaceOpen.value = true
			nextTick(() => searchInputRef.value?.focus())
		},
	})
}

async function saveFileContent(exit: boolean = false) {
	if (!props.file) return

	try {
		const normalizedPath = props.file.path.startsWith('/') ? props.file.path : `/${props.file.path}`
		await ctx.writeFile(normalizedPath, fileContent.value)

		originalContent.value = fileContent.value

		if (exit) {
			emit('close')
		}

		addNotification({
			title: formatMessage(messages.fileSavedTitle),
			text: formatMessage(messages.fileSavedText),
			type: 'success',
		})
	} catch (error) {
		console.error('Error saving file content:', error)
		addNotification({
			title: formatMessage(messages.saveFailedTitle),
			text: formatMessage(messages.saveFailedText),
			type: 'error',
		})
	}
}

async function shareToMclogs() {
	if (ctx.shareToMclogs) {
		await ctx.shareToMclogs(fileContent.value)
		return
	}

	try {
		const response = await fetch('https://api.mclo.gs/1/log', {
			method: 'POST',
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			body: new URLSearchParams({ content: fileContent.value }),
		})

		const data = (await response.json()) as MclogsResponse

		if (data.success && data.url) {
			await navigator.clipboard.writeText(data.url)
			addNotification({
				title: formatMessage(messages.logUrlCopiedTitle),
				text: formatMessage(messages.logUrlCopiedText),
				type: 'success',
			})
		} else {
			throw new Error(data.error)
		}
	} catch (error) {
		console.error('Error sharing file:', error)
		addNotification({
			title: formatMessage(messages.failedToShareTitle),
			text: formatMessage(messages.failedToShareText),
			type: 'error',
		})
	}
}

function countOccurrences(content: string, query: string): number {
	if (!query) return 0
	const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
	return (content.match(new RegExp(escaped, 'gi')) ?? []).length
}

function toggleSearch() {
	if (isSearchOpen.value) {
		closeSearch()
	} else {
		isSearchOpen.value = true
		isReplaceOpen.value = false
		nextTick(() => searchInputRef.value?.focus())
	}
}

function toggleReplace() {
	isReplaceOpen.value = !isReplaceOpen.value
	if (isReplaceOpen.value) {
		nextTick(() => replaceInputRef.value?.focus())
	}
}

function closeSearch() {
	isSearchOpen.value = false
	isReplaceOpen.value = false
	inFileSearchQuery.value = ''
	replaceQuery.value = ''
	searchMatchCount.value = 0
	currentSearchMatch.value = 0
	editorInstance.value?.find('', { wrap: true })
	editorInstance.value?.focus()
}

function replaceOne() {
	const editor = editorInstance.value
	if (!editor || searchMatchCount.value === 0) return
	editor.replace(replaceQuery.value)
	nextTick(() => {
		const count = countOccurrences(fileContent.value, inFileSearchQuery.value)
		searchMatchCount.value = count
		currentSearchMatch.value = count > 0 ? Math.min(currentSearchMatch.value, count) : 0
	})
}

function replaceAllOccurrences() {
	const editor = editorInstance.value
	if (!editor || searchMatchCount.value === 0) return
	editor.replaceAll(replaceQuery.value)
	nextTick(() => {
		const count = countOccurrences(fileContent.value, inFileSearchQuery.value)
		searchMatchCount.value = count
		currentSearchMatch.value = count > 0 ? 1 : 0
		if (count > 0) {
			editor.find(inFileSearchQuery.value, { wrap: true, caseSensitive: false })
		}
	})
}

function handleSearchInput() {
	const editor = editorInstance.value
	if (!editor) return

	const query = inFileSearchQuery.value
	if (!query) {
		searchMatchCount.value = 0
		currentSearchMatch.value = 0
		editor.find('', { wrap: true })
		return
	}

	const count = countOccurrences(fileContent.value, query)
	searchMatchCount.value = count

	if (count > 0) {
		editor.find(query, { wrap: true, caseSensitive: false })
		currentSearchMatch.value = 1
	} else {
		currentSearchMatch.value = 0
	}
}

function findNext() {
	const editor = editorInstance.value
	if (!editor || searchMatchCount.value === 0) return
	editor.findNext()
	currentSearchMatch.value = (currentSearchMatch.value % searchMatchCount.value) + 1
}

function findPrevious() {
	const editor = editorInstance.value
	if (!editor || searchMatchCount.value === 0) return
	editor.findPrevious()
	currentSearchMatch.value =
		((currentSearchMatch.value - 2 + searchMatchCount.value) % searchMatchCount.value) + 1
}

function close() {
	resetState()
	emit('close')
}

onUnmounted(() => {
	window.removeEventListener('resize', updateEditorHeight)
	editorInstance.value = null
	resetState()
})

defineExpose({
	saveFileContent,
	shareToMclogs,
	close,
	isEditingImage,
	isSearchOpen,
	fileContent,
	hasUnsavedChanges,
	revertChanges,
	toggleSearch,
})
</script>

<style scoped>
.search-bar-enter-active,
.search-bar-leave-active {
	transition:
		opacity 0.15s ease,
		transform 0.15s ease;
}

.search-bar-enter-from,
.search-bar-leave-to {
	opacity: 0;
	transform: translateY(-4px) scale(0.97);
}
</style>
