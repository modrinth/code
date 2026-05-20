<template>
	<div
		ref="editorContainer"
		class="relative flex flex-col overflow-hidden rounded-[20px] border border-solid border-surface-4 shadow-sm"
	>
		<EditorFindReplace
			ref="findReplaceRef"
			v-model:is-find-open="isFindOpen"
			v-model:find-query="inFileFindQuery"
			:is-editing-image="isEditingImage"
			:find-match-count="findMatchCount"
			:current-find-match="currentFindMatch"
			@find-next="findNext"
			@find-previous="findPrevious"
			@close="closeFind"
			@replace="replaceOne"
			@replace-all="replaceAllOccurrences"
		/>
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
import { SpinnerIcon } from '@modrinth/assets'
import type { Ace } from 'ace-builds'
import { type Component, computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { getEditorLanguage, getFileExtension, isImageFile } from '#ui/utils/file-extensions'

import { injectFileManager } from '../../providers/file-manager'
import type { EditingFile } from '../../types'
import EditorFindReplace from './EditorFindReplace.vue'
import FileImageViewer from './FileImageViewer.vue'

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
const client = injectModrinthClient()

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
})

const fileContent = ref('')
const originalContent = ref('')
const isEditingImage = ref(false)
const imagePreview = ref<Blob | null>(null)
const isLoading = ref(false)
const editorInstance = ref<Ace.Editor | null>(null)
const editorContainer = ref<HTMLElement | null>(null)
const editorHeight = ref('300px')

const isFindOpen = ref(false)
const inFileFindQuery = ref('')
const findMatchCount = ref(0)
const currentFindMatch = ref(0)
const findReplaceRef = ref<{ focusFindInput: () => void; openReplace: () => void } | null>(null)

watch(inFileFindQuery, handleFindInput)

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
			closeFind()
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

function onEditorInit(editor: Ace.Editor) {
	editorInstance.value = editor

	editor.commands.addCommand({
		name: 'save',
		bindKey: { win: 'Ctrl-S', mac: 'Command-S' },
		exec: () => saveFileContent(false),
	})

	editor.commands.addCommand({
		name: 'find',
		bindKey: { win: 'Ctrl-F', mac: 'Command-F' },
		exec: () => toggleFind(),
	})

	editor.commands.addCommand({
		name: 'replace',
		bindKey: { win: 'Ctrl-H', mac: 'Command-Option-F' },
		exec: () => {
			isFindOpen.value = true
			nextTick(() => findReplaceRef.value?.openReplace())
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
		const data = await client.mclogs.logs_v1.create(fileContent.value)

		if (data.success && data.url) {
			await navigator.clipboard.writeText(data.url)
			addNotification({
				title: formatMessage(messages.logUrlCopiedTitle),
				text: formatMessage(messages.logUrlCopiedText),
				type: 'success',
			})
		} else {
			throw new Error('mclo.gs upload failed')
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

function toggleFind() {
	if (isFindOpen.value) {
		closeFind()
	} else {
		isFindOpen.value = true
		nextTick(() => findReplaceRef.value?.focusFindInput())
	}
}

function closeFind() {
	isFindOpen.value = false
	inFileFindQuery.value = ''
	findMatchCount.value = 0
	currentFindMatch.value = 0
	editorInstance.value?.find('', { wrap: true })
	editorInstance.value?.focus()
}

function replaceOne(query: string) {
	const editor = editorInstance.value
	if (!editor || findMatchCount.value === 0) return
	editor.replace(query)
	nextTick(() => {
		const count = countOccurrences(fileContent.value, inFileFindQuery.value)
		findMatchCount.value = count
		currentFindMatch.value = count > 0 ? Math.min(currentFindMatch.value, count) : 0
	})
}

function replaceAllOccurrences(query: string) {
	const editor = editorInstance.value
	if (!editor || findMatchCount.value === 0) return
	editor.replaceAll(query)
	nextTick(() => {
		const count = countOccurrences(fileContent.value, inFileFindQuery.value)
		findMatchCount.value = count
		currentFindMatch.value = count > 0 ? 1 : 0
		if (count > 0) {
			editor.find(inFileFindQuery.value, { wrap: true, caseSensitive: false })
		}
	})
}

function handleFindInput() {
	const editor = editorInstance.value
	if (!editor) return

	const query = inFileFindQuery.value
	if (!query) {
		findMatchCount.value = 0
		currentFindMatch.value = 0
		editor.find('', { wrap: true })
		return
	}

	const count = countOccurrences(fileContent.value, query)
	findMatchCount.value = count

	if (count > 0) {
		editor.find(query, { wrap: true, caseSensitive: false })
		currentFindMatch.value = 1
	} else {
		currentFindMatch.value = 0
	}
}

function findNext() {
	const editor = editorInstance.value
	if (!editor || findMatchCount.value === 0) return
	editor.findNext()
	currentFindMatch.value = (currentFindMatch.value % findMatchCount.value) + 1
}

function findPrevious() {
	const editor = editorInstance.value
	if (!editor || findMatchCount.value === 0) return
	editor.findPrevious()
	currentFindMatch.value =
		((currentFindMatch.value - 2 + findMatchCount.value) % findMatchCount.value) + 1
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
	isFindOpen,
	fileContent,
	hasUnsavedChanges,
	revertChanges,
	toggleFind,
})
</script>
