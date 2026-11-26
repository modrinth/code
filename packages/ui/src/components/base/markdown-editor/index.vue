<template>
	<InsertLinkModal ref="linkModal" @insert="insertMarkdown" />
	<InsertImageModal
		ref="imageModal"
		:on-image-upload="props.onImageUpload"
		@insert="insertMarkdown"
	/>
	<InsertVideoModal ref="videoModal" @insert="insertMarkdown" />
	<div v-bind="$attrs" class="block box-border min-w-0 grow w-full p-1">
		<slot v-if="$slots.toolbar" name="toolbar" :actions="toolbarActions" />
		<MarkdownEditorToolbar v-else v-bind="toolbarActions" class="mb-2">
			<div class="flex items-center gap-2 flex-1">
				<Toggle :id="previewId" v-model="previewMode" small />
				<label class="label" :for="previewId">
					{{ formatMessage(messages.editorPreviewToggleLabel) }}
				</label>
				<slot name="after-preview" :preview-mode="previewMode" />
			</div>
		</MarkdownEditorToolbar>
		<div class="editor-panes" :class="{ 'is-split': editorMode === 'split' }">
			<InputFrame
				class="editor-input"
				:class="{ hide: previewMode }"
				:disabled="disabled"
				multiline
				@mousedown="focusEditorFrame"
			>
				<div ref="editorRef" class="min-w-0 w-full flex-1 self-stretch" />
			</InputFrame>
			<div v-if="!previewMode && (!hideMarkdownHint || maxLength)" class="info-blurb mt-2">
				<div v-if="!hideMarkdownHint" class="info-blurb">
					<InfoIcon />
					<IntlFormatted :message-id="messages.editorMarkdownFormattingSupport">
						<template #markdown-link="{ children }">
							<a
								class="markdown-resource-link"
								href="https://support.modrinth.com/en/articles/8801962-advanced-markdown-formatting"
								target="_blank"
							>
								<component :is="() => children" />
							</a>
						</template>
					</IntlFormatted>
				</div>
				<div :class="{ hide: !props.maxLength }" class="max-length-label">
					<span>
						{{
							props.maxLength
								? formatMessage(messages.editorMaxLengthValue, {
										currentLength: currentValue?.length || 0,
										maxLength: props.maxLength,
									})
								: formatMessage(messages.editorMaxLengthUnlimited)
						}}
					</span>
				</div>
			</div>
			<div v-if="previewMode || editorMode === 'split'" class="editor-preview">
				<div class="markdown-body-wrapper">
					<slot v-if="!currentValue?.trim()" name="empty-preview" />
					<div
						style="width: 100%"
						:style="{
							maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
							overflowY: 'auto',
						}"
						class="markdown-body"
					>
						<slot name="preview" :value="currentValue ?? ''">
							<div v-html="renderHighlightedString(currentValue ?? '')" />
						</slot>
					</div>
				</div>
			</div>
		</div>
	</div>
	<slot name="footer" :disabled="disabled" :open-image-modal="openImageModal" />
</template>

<script setup lang="ts">
import { history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { ensureSyntaxTree, syntaxTree } from '@codemirror/language'
import { Compartment, EditorState, type Extension } from '@codemirror/state'
import { type Command, EditorView, keymap, placeholder as cm_placeholder } from '@codemirror/view'
import { InfoIcon } from '@modrinth/assets'
import { markdownCommands, modrinthMarkdownEditorKeymap } from '@modrinth/utils/codemirror'
import { renderHighlightedString } from '@modrinth/utils/highlightjs/index'
import {
	computed,
	nextTick,
	onBeforeUnmount,
	onMounted,
	ref,
	toRef,
	useId,
	useTemplateRef,
	watch,
} from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { readCodeClipboard } from '../../../utils/markdown-code'
import InputFrame from '../inputs/InputFrame.vue'
import IntlFormatted from '../IntlFormatted.vue'
import Toggle from '../Toggle.vue'
import InsertImageModal from './insert-image-modal.vue'
import InsertLinkModal from './insert-link-modal.vue'
import InsertVideoModal from './insert-video-modal.vue'
import { parseMarkdownUrl } from './markdown-url'
import MarkdownEditorToolbar from './toolbar.vue'
import type { MarkdownEditorMode } from './types'

defineOptions({ inheritAttrs: false })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	editorPreviewToggleLabel: {
		id: 'markdown-editor.preview-toggle.label',
		defaultMessage: 'Preview',
	},
	editorMarkdownFormattingSupport: {
		id: 'markdown-editor.markdown-formatting-support',
		defaultMessage: 'This editor supports <markdown-link>Markdown formatting</markdown-link>.',
	},
	editorMaxLengthValue: {
		id: 'markdown-editor.max-length.value',
		defaultMessage: '{currentLength}/{maxLength}',
	},
	editorMaxLengthUnlimited: {
		id: 'markdown-editor.max-length.unlimited',
		defaultMessage: 'Unlimited',
	},
	editorPlaceholder: {
		id: 'markdown-editor.placeholder',
		defaultMessage: 'Write something...',
	},

	defaultImageAltText: {
		id: 'markdown-editor.default-image-alt-text',
		defaultMessage: 'Replace this with a description',
	},
})

const props = withDefaults(
	defineProps<{
		modelValue: string
		extensions?: Extension[]
		disabled?: boolean
		headingButtons?: boolean
		hideFormattingButtons?: boolean
		initialPreview?: boolean
		hideMarkdownHint?: boolean
		/**
		 * @param file The file to upload
		 * @throws If the file is invalid or the upload fails
		 */
		onImageUpload?: (file: File) => Promise<string | undefined>
		placeholder?: string
		maxLength?: number
		maxHeight?: number
		minHeight?: number
	}>(),
	{
		modelValue: '',
		disabled: false,
		headingButtons: true,
		hideFormattingButtons: false,
		initialPreview: false,
		hideMarkdownHint: false,
		onImageUpload: undefined,
		placeholder: undefined,
		maxLength: undefined,
		maxHeight: undefined,
		minHeight: undefined,
	},
)

const editorRef = ref<HTMLDivElement>()
let editor: EditorView | null = null
let isDisabledCompartment: Compartment | null = null
let editorThemeCompartment: Compartment | null = null
const extensionsCompartment = new Compartment()

const previewId = useId()

function focusEditorFrame(event: MouseEvent) {
	if (
		event.button !== 0 ||
		props.disabled ||
		previewMode.value ||
		!editor ||
		editor.contentDOM.contains(event.target as Node)
	)
		return
	const position =
		editor.posAtCoords({ x: event.clientX, y: event.clientY }) ?? editor.state.doc.length
	editor.dispatch({ selection: { anchor: position } })
	editor.focus()
	event.preventDefault()
}

const emit = defineEmits<{
	'update:modelValue': [value: string]
	ready: [view: EditorView]
}>()
const resolvedPlaceholder = computed(
	() => props.placeholder ?? formatMessage(messages.editorPlaceholder),
)

function createEditorTheme(disabled = props.disabled) {
	return EditorView.theme({
		'&': {
			background: 'transparent',
			width: '100%',
		},
		'&.cm-focused': {
			outline: 'none',
		},
		'.cm-content': {
			minHeight: `${props.minHeight ?? 200}px`,
			padding: '0 2px',
			caretColor: 'var(--color-contrast)',
			width: '100%',
			pointerEvents: disabled ? 'none' : 'auto',
		},
		'.cm-scroller': {
			height: '100%',
			maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
			overflow: 'auto',
		},
	})
}

onMounted(() => {
	const updateListener = EditorView.updateListener.of((update) => {
		if (update.docChanged) {
			updateCurrentValue(update.state.doc.toString())
		}
	})

	editorThemeCompartment = new Compartment()

	const theme = createEditorTheme()

	isDisabledCompartment = new Compartment()

	const disabledCompartment = EditorState.readOnly.of(props.disabled)

	const eventHandlers = EditorView.domEventHandlers({
		paste: (ev, view) => {
			const { clipboardData } = ev
			if (!clipboardData || props.disabled) return

			const code = readCodeClipboard(view.contentDOM.ownerDocument, clipboardData)
			if (code) {
				const { from, to } = view.state.selection.main
				let node = (ensureSyntaxTree(view.state, from, 50) ?? syntaxTree(view.state)).resolveInner(
					from,
					-1,
				)
				while (node) {
					if (node.name === 'CodeBlock') return false
					if (node.name === 'FencedCode') {
						const closingFence = node.lastChild
						if (
							closingFence?.name !== 'CodeMark' ||
							closingFence.from === node.firstChild?.from ||
							from < closingFence.to
						)
							return false
						break
					}
					if (!node.parent) break
					node = node.parent
				}
				const before = view.state.doc.sliceString(0, from)
				const after = view.state.doc.sliceString(to)
				const prefix = before.length && !before.endsWith('\n') ? '\n' : ''
				const suffix = after.length && !after.startsWith('\n') ? '\n' : ''
				const insert = prefix + code + suffix
				if (
					props.maxLength &&
					view.state.doc.length - (to - from) + insert.length > props.maxLength
				) {
					ev.preventDefault()
					return true
				}
				ev.preventDefault()
				view.dispatch({
					changes: { from, to, insert },
					selection: { anchor: from + insert.length },
					scrollIntoView: true,
					userEvent: 'input.paste',
				})
				return true
			}

			const image = clipboardData.files?.[0]
			if (image && props.onImageUpload) {
				// If the user is pasting a file, upload it if there's an included handler and insert the link.
				props
					.onImageUpload(image)
					.then(function (url) {
						if (!url) return
						const selection = markdownCommands.yankSelection(view)
						const altText = selection || formatMessage(messages.defaultImageAltText)
						const linkMarkdown = `![${altText}](${url})`
						return markdownCommands.replaceSelection(view, linkMarkdown)
					})
					.catch((error) => {
						if (error instanceof Error) {
							console.error('Problem with handling image.', error)
						}
					})

				return false
			}

			// If the user's pasting a url, automatically convert it to a link with the selection as the text or the url itself if no selection content.
			const url = ev.clipboardData?.getData('text/plain')

			if (url) {
				if (!parseMarkdownUrl(url).url) return

				const selection = view.state.selection.main
				const selectionText = view.state.doc.sliceString(selection.from, selection.to)
				if (selectionText) {
					const linkMarkdown = `[${selectionText}](${url})`
					return markdownCommands.replaceSelection(view, linkMarkdown)
				}
			}

			// Check if the length of the document is greater than the max length. If it is, prevent the paste.
			if (props.maxLength && view.state.doc.length > props.maxLength) {
				ev.preventDefault()
				return false
			}
		},
		blur: (_, view) => {
			if (props.maxLength && view.state.doc.length > props.maxLength) {
				// Calculate how many characters to remove from the end
				const excessLength = view.state.doc.length - props.maxLength
				// Dispatch transaction to remove excess characters
				view.dispatch({
					changes: {
						from: view.state.doc.length - excessLength,
						to: view.state.doc.length,
					},
					selection: { anchor: props.maxLength, head: props.maxLength }, // Place cursor at the end
				})
			}
		},
	})

	const inputFilter = EditorState.changeFilter.of((transaction) => {
		if (props.maxLength && transaction.newDoc.length > props.maxLength) {
			return false
		}
		return true
	})

	const editorState = EditorState.create({
		extensions: [
			EditorView.lineWrapping,
			extensionsCompartment.of(props.extensions ?? []),
			eventHandlers,
			updateListener,
			keymap.of([indentWithTab]),
			keymap.of(modrinthMarkdownEditorKeymap),
			history(),
			markdown({
				addKeymap: false,
			}),
			keymap.of(historyKeymap),
			cm_placeholder(resolvedPlaceholder.value),
			inputFilter,
			isDisabledCompartment.of(disabledCompartment),
			editorThemeCompartment.of(theme),
		],
	})

	editor = new EditorView({
		state: editorState,
		parent: editorRef.value,
		doc: props.modelValue ?? '', // This doesn't work for some reason
	})

	// set editor content to props.modelValue
	editor?.dispatch({
		changes: {
			from: 0,
			to: editor.state.doc.length,
			insert: props.modelValue,
		},
	})
	emit('ready', editor)
})

watch(
	() => props.extensions,
	(extensions) => {
		editor?.dispatch({ effects: extensionsCompartment.reconfigure(extensions ?? []) })
	},
)

onBeforeUnmount(() => {
	editor?.destroy()
})

function runCommand(command: Command) {
	if (!editor) return
	command(editor)
	editor.focus()
}

watch(
	() => props.disabled,
	(newValue) => {
		if (editor) {
			if (isDisabledCompartment) {
				editor.dispatch({
					effects: [isDisabledCompartment.reconfigure(EditorState.readOnly.of(newValue))],
				})
			}

			if (editorThemeCompartment) {
				editor.dispatch({
					effects: [editorThemeCompartment.reconfigure(createEditorTheme(newValue))],
				})
			}
		}
	},
	{
		immediate: true,
	},
)

const currentValue = toRef(props, 'modelValue')
watch(
	currentValue,
	(newValue) => {
		if (editor && newValue !== editor.state.doc.toString()) {
			editor.dispatch({
				changes: {
					from: 0,
					to: editor.state.doc.length,
					insert: newValue,
				},
			})
		}
	},
	{
		immediate: true,
	},
)

const updateCurrentValue = (newValue: string) => {
	emit('update:modelValue', newValue)
}

const mode = defineModel<MarkdownEditorMode>('mode')
const initialMode = props.initialPreview ? 'preview' : 'write'
const editorMode = computed(() => mode.value ?? initialMode)
const previewMode = computed({
	get: () => editorMode.value === 'preview',
	set: (value: boolean) => {
		mode.value = value ? 'preview' : 'write'
	},
})
const toolbarActions = computed(() => ({
	disabled: previewMode.value || props.disabled,
	headingButtons: props.headingButtons,
	hideFormattingButtons: props.hideFormattingButtons,
	runCommand,
	openLinkModal,
	openImageModal,
	openVideoModal,
}))

async function focus() {
	if (props.disabled) return
	previewMode.value = false
	await nextTick()
	editor?.focus()
}

defineExpose({ focus, openImageModal })

const linkModal = useTemplateRef('linkModal')
const imageModal = useTemplateRef('imageModal')
const videoModal = useTemplateRef('videoModal')

function openLinkModal() {
	linkModal.value?.show()
}

function openImageModal() {
	imageModal.value?.show()
}

function openVideoModal() {
	videoModal.value?.show()
}

function insertMarkdown(markdown: string) {
	if (editor) markdownCommands.replaceSelection(editor, markdown)
}
</script>

<style lang="scss" scoped>
.markdown-resource-link {
	cursor: pointer;
	color: var(--color-blue);

	&:focus-visible,
	&:hover {
		filter: brightness(1.2);
		text-decoration: none;
	}

	&:active {
		filter: brightness(1.1);
		text-decoration: none;
	}
}

.display-options {
	margin-bottom: var(--gap-sm);
}

.info-blurb {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--gap-xs);
}

.hide {
	display: none;
}

.preview {
	display: flex;
	align-items: center;
	justify-items: end;
	gap: var(--gap-xs);
}

.markdown-body-wrapper {
	border: 1px solid var(--color-divider);
	border-radius: var(--radius-md);
	width: 100%;
	padding: var(--radius-md);
	min-height: 6rem;
}

.cm-disabled {
	opacity: 0.6;
	pointer-events: none;
	cursor: not-allowed;
}

:deep(.cm-line) {
	padding: 0 !important;
}

.editor-panes.is-split {
	display: grid;
	grid-template-columns: minmax(0, 1fr);

	@media (min-width: 1024px) {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}

.editor-preview {
	min-width: 0;
}
</style>
