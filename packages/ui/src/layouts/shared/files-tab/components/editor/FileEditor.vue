<template>
	<div
		ref="editorContainer"
		class="relative flex flex-col overflow-hidden"
		:class="{ 'h-full': props.fillHeight }"
	>
		<EditorFindReplace
			ref="findReplaceRef"
			v-model:is-find-open="isFindOpen"
			v-model:find-query="inFileFindQuery"
			:is-editing-image="isEditingImage"
			:readonly="isEditorReadOnly"
			:find-match-count="findMatchCount"
			:current-find-match="currentFindMatch"
			@find-next="findNext"
			@find-previous="findPrevious"
			@close="closeFind"
			@replace="replaceOne"
			@replace-all="replaceAllOccurrences"
		/>
		<CodeEditor
			v-if="!isEditingImage && !mediaPreview && binarySize == null && !isLoading"
			ref="codeEditor"
			v-model="fileContent"
			:language="editorLanguage"
			:readonly="isEditorReadOnly"
			:extensions="editorKeys"
			:style="{ height: props.fillHeight ? undefined : editorHeight }"
			class="text-sm"
			:class="props.fillHeight ? 'min-h-0 flex-1' : 'rounded-[20px]'"
			@save="saveFileContent(false)"
		/>
		<FileImageViewer
			v-else-if="isEditingImage && imagePreview"
			:image-blob="imagePreview"
			:class="{ '!h-full !rounded-none': props.fillHeight }"
		/>
		<FileMediaViewer
			v-else-if="mediaPreview && props.file"
			:blob="mediaPreview.blob"
			:kind="mediaPreview.kind"
			:name="props.file.name"
			:class="{ '!h-full !rounded-none': props.fillHeight }"
		/>
		<FilePreviewFallback
			v-else-if="binarySize != null && props.file"
			:file="props.file"
			:size="binarySize"
			:class="{ '!h-full !rounded-none': props.fillHeight }"
		/>
		<FileViewControls
			v-if="props.tabId && props.file"
			class="absolute bottom-3 right-5 z-10"
			:tab-id="props.tabId"
			:file="props.file"
		/>
		<div
			v-else-if="isLoading"
			class="flex items-center justify-center bg-bg-raised"
			:class="props.fillHeight ? 'min-h-0 flex-1' : 'rounded-[20px]'"
			:style="{ height: props.fillHeight ? undefined : editorHeight }"
		>
			<SpinnerIcon class="h-8 w-8 animate-spin text-secondary" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { keymap } from '@codemirror/view'
import { SpinnerIcon } from '@modrinth/assets'
import {
	computed,
	nextTick,
	onMounted,
	onUnmounted,
	ref,
	shallowRef,
	useTemplateRef,
	watch,
} from 'vue'

import CodeEditor from '#ui/components/base/CodeEditor.vue'
import type CodeEditorImpl from '#ui/components/base/CodeEditorImpl.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import {
	clearFind,
	type FindMatches,
	findNextMatch,
	findPreviousMatch,
	findText,
	replaceAllMatches,
	replaceMatch,
} from '#ui/utils/codemirror/find'
import {
	getEditorLanguage,
	getFileExtension,
	isImageFile,
	mediaKindOf,
} from '#ui/utils/file-extensions'

import { injectFileBrowserUI } from '../../providers/file-browser-ui'
import {
	type FileInfo,
	type FileInterpretation,
	injectFileManager,
} from '../../providers/file-manager'
import EditorFindReplace from './EditorFindReplace.vue'
import FileImageViewer from './FileImageViewer.vue'
import FileMediaViewer from './FileMediaViewer.vue'
import FilePreviewFallback from './FilePreviewFallback.vue'
import FileViewControls from './FileViewControls.vue'

const props = defineProps<{
	file: FileInfo<'file'> | null
	/** Fill the parent's height (e.g. a dockview panel) instead of sizing to the window. */
	fillHeight?: boolean
	/** Which of the file's content variants to show (see `FileManagerContext.contentVariants`). */
	variant?: string | null
	/**
	 * How to show the file: `auto` picks by its type, while the others show its stored bytes as
	 * text, as a hex dump or as formatted JSON, read-only.
	 */
	interpretation?: FileInterpretation
	/** The tab showing the file, which offers its view controls in the view's corner. */
	tabId?: string
}>()

const emit = defineEmits<{
	close: []
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()
const ui = injectFileBrowserUI(null)
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
	hexTruncated: {
		id: 'files.editor.hex-truncated',
		defaultMessage: '… {bytes, plural, one {# more byte} other {# more bytes}} not shown',
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
const mediaPreview = shallowRef<{ blob: Blob; kind: 'video' | 'audio' } | null>(null)
/** The size of a file whose contents can't be shown, which then shows in their place. */
const binarySize = ref<number | null>(null)
const isLoading = ref(false)
const codeEditor = useTemplateRef<InstanceType<typeof CodeEditorImpl>>('codeEditor')
const editorView = computed(() => codeEditor.value?.view ?? null)
const editorContainer = ref<HTMLElement | null>(null)
const editorHeight = ref('300px')

const isFindOpen = ref(false)
const inFileFindQuery = ref('')
const findMatchCount = ref(0)
const currentFindMatch = ref(0)
const findReplaceRef = ref<{ focusFindInput: () => void; openReplace: () => void } | null>(null)

watch(inFileFindQuery, handleFindInput)

function updateEditorHeight() {
	if (props.fillHeight) return
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

/** Whether the file is shown other than as itself, which isn't something to edit. */
const isInterpreted = computed(() => (props.interpretation ?? 'auto') !== 'auto')

const editorLanguage = computed(() => {
	if (props.interpretation === 'json') return 'json'
	if (isInterpreted.value) return 'text'
	const hostLanguage = props.file ? ctx.editorLanguage?.(props.file) : undefined
	return hostLanguage ?? getEditorLanguage(props.file?.name ?? '')
})
const isEditorReadOnly = computed(
	() =>
		isInterpreted.value || (ctx.isBusy?.value ?? false) || (ctx.isReadOnly?.(props.file) ?? false),
)

watch(
	[() => props.file, () => props.variant, () => props.interpretation, () => ui?.jsonIndent.value],
	async ([newFile]) => {
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

async function loadFileContent(file: FileInfo<'file'>) {
	isLoading.value = true
	try {
		if (!props.fillHeight) window.scrollTo(0, 0)
		const extension = getFileExtension(file.name)
		const result = ctx.directoryTree.get(file, props.variant ?? undefined)

		const holder = Promise.withResolvers<ArrayBuffer>()

		const intervalId = setInterval(() => {
			if (!result.isLoading.value) {
				const data = result.data.value

				if (data != null) {
					clearInterval(intervalId)
					holder.resolve(data)
				}
			}

			if (result.loadError.value != null) {
				clearInterval(intervalId)
				holder.reject(result.loadError.value)
			}
		})

		const data = await holder.promise

		const mediaKind = mediaKindOf(file.name)
		resetState()
		if (isInterpreted.value) {
			const content = interpret(data, props.interpretation!)
			fileContent.value = content
			originalContent.value = content
		} else if (isImageFile(extension)) {
			isEditingImage.value = true
			imagePreview.value = new Blob([data])
		} else if (mediaKind) {
			mediaPreview.value = { blob: new Blob([data]), kind: mediaKind }
		} else if (looksBinary(data)) {
			binarySize.value = data.byteLength
		} else {
			const content = new TextDecoder().decode(data)
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
	mediaPreview.value = null
	binarySize.value = null
}

/** How much of a file a hex dump covers, so huge files don't freeze the editor. */
const HEX_DUMP_LIMIT = 2 * 1024 * 1024
const HEX_BYTES_PER_LINE = 16

/** Lines of offset, bytes in hex and the bytes as printable ASCII, like `xxd` prints them. */
function hexDump(data: ArrayBuffer) {
	const bytes = new Uint8Array(data, 0, Math.min(data.byteLength, HEX_DUMP_LIMIT))
	const lines: string[] = []
	for (let offset = 0; offset < bytes.length; offset += HEX_BYTES_PER_LINE) {
		const line = bytes.subarray(offset, offset + HEX_BYTES_PER_LINE)
		const hex = Array.from(line, (byte) => byte.toString(16).padStart(2, '0'))
			.join(' ')
			.padEnd(HEX_BYTES_PER_LINE * 3 - 1)
		const ascii = Array.from(line, (byte) =>
			byte >= 0x20 && byte < 0x7f ? String.fromCharCode(byte) : '.',
		).join('')
		lines.push(`${offset.toString(16).padStart(8, '0')}  ${hex}  ${ascii}`)
	}
	if (data.byteLength > HEX_DUMP_LIMIT) {
		lines.push(formatMessage(messages.hexTruncated, { bytes: data.byteLength - HEX_DUMP_LIMIT }))
	}
	return lines.join('\n')
}

/**
 * Line breaks typed straight into the data rather than escaped (as `\n`). Between values they're
 * only whitespace, but within strings they make JSON invalid, so they're dropped before parsing.
 */
const UNESCAPED_LINE_BREAKS = /[\r\n]+/g

/**
 * The file's stored bytes shown as text, a hex dump, or JSON laid out over lines and indented (as
 * the settings say) when it parses as JSON, e.g. compacted JSON.
 */
function interpret(data: ArrayBuffer, interpretation: FileInterpretation) {
	if (interpretation === 'hex') return hexDump(data)
	const text = new TextDecoder().decode(data)
	if (interpretation !== 'json') return text
	try {
		const parsed = JSON.parse(text.replace(UNESCAPED_LINE_BREAKS, ''))
		return JSON.stringify(parsed, null, ui?.jsonIndent.value ?? '\t')
	} catch {
		return text
	}
}

/** How much of a file is checked for being binary: enough to cover any text file's start. */
const BINARY_SNIFF_LENGTH = 8192

/** Text files don't hold NUL bytes, while nearly every binary format does near its start. */
function looksBinary(data: ArrayBuffer) {
	return new Uint8Array(data, 0, Math.min(data.byteLength, BINARY_SNIFF_LENGTH)).includes(0)
}

/** Find and replace open the page's own find bar, in place of the editor's. */
const editorKeys = [
	keymap.of([
		{
			key: 'Mod-f',
			preventDefault: true,
			run: () => {
				toggleFind()
				return true
			},
		},
		{
			key: 'Mod-h',
			mac: 'Mod-Alt-f',
			preventDefault: true,
			run: () => {
				if (isEditorReadOnly.value) return false
				isFindOpen.value = true
				nextTick(() => findReplaceRef.value?.openReplace())
				return true
			},
		},
	]),
]

async function saveFileContent(exit: boolean = false) {
	if (!props.file) return
	if (isEditorReadOnly.value) return

	try {
		await ctx.writeFile(props.file, new TextEncoder().encode(fileContent.value).buffer)

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

function showMatches(matches: FindMatches) {
	findMatchCount.value = matches.total
	currentFindMatch.value = matches.current
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
	if (editorView.value) {
		clearFind(editorView.value)
		editorView.value.focus()
	}
}

function replaceOne(replacement: string) {
	if (!editorView.value || isEditorReadOnly.value || findMatchCount.value === 0) return
	showMatches(replaceMatch(editorView.value, replacement))
}

function replaceAllOccurrences(replacement: string) {
	if (!editorView.value || isEditorReadOnly.value || findMatchCount.value === 0) return
	showMatches(replaceAllMatches(editorView.value, replacement))
}

function handleFindInput() {
	if (!editorView.value) return
	showMatches(findText(editorView.value, inFileFindQuery.value))
}

function findNext() {
	if (!editorView.value || findMatchCount.value === 0) return
	showMatches(findNextMatch(editorView.value))
}

function findPrevious() {
	if (!editorView.value || findMatchCount.value === 0) return
	showMatches(findPreviousMatch(editorView.value))
}

function close() {
	resetState()
	emit('close')
}

onUnmounted(() => {
	window.removeEventListener('resize', updateEditorHeight)
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
