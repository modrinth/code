<template>
	<NewModal ref="linkModal" :header="formatMessage(messages.linkModalHeader)" class="!w-[40rem]">
		<div class="modal-insert">
			<label class="label" for="insert-link-label">
				<span class="label__title">{{ formatMessage(messages.linkModalLabelFieldTitle) }}</span>
			</label>
			<StyledInput
				id="insert-link-label"
				v-model="linkText"
				:icon="AlignLeftIcon"
				type="text"
				:placeholder="formatMessage(messages.linkModalLabelFieldPlaceholder)"
				clearable
				wrapper-class="w-full"
			/>
			<label class="label" for="insert-link-url">
				<span class="label__title">
					{{ formatMessage(messages.urlLabel) }}<span class="required">*</span>
				</span>
			</label>
			<StyledInput
				id="insert-link-url"
				v-model="linkUrl"
				:icon="LinkIcon"
				type="text"
				:placeholder="formatMessage(messages.linkModalUrlFieldPlaceholder)"
				clearable
				wrapper-class="w-full"
				@input="validateURL"
			/>
			<template v-if="linkValidationErrorMessage">
				<span class="label">
					<span class="label__title">{{ formatMessage(messages.errorLabel) }}</span>
					<span class="label__description">{{ linkValidationErrorMessage }}</span>
				</span>
			</template>
			<span class="label">
				<span class="label__title">{{ formatMessage(messages.previewLabel) }}</span>
				<span class="label__description"></span>
			</span>
			<div class="markdown-body-wrapper">
				<div
					style="width: 100%"
					class="markdown-body"
					v-html="renderHighlightedString(linkMarkdown)"
				/>
			</div>
			<div class="flex gap-2 justify-end mt-4">
				<ButtonStyled type="outlined">
					<button @click="() => linkModal?.hide()">
						<XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						:disabled="!!linkValidationErrorMessage || !linkUrl"
						@click="
							() => {
								if (editor) markdownCommands.replaceSelection(editor, linkMarkdown)
								linkModal?.hide()
							}
						"
					>
						<PlusIcon /> {{ formatMessage(messages.insertButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<NewModal ref="imageModal" :header="formatMessage(messages.imageModalHeader)" class="!w-[40rem]">
		<div class="modal-insert">
			<label class="label" for="insert-image-alt">
				<span class="label__title">
					{{ formatMessage(messages.imageModalDescriptionFieldTitle) }}
					<span class="required">*</span>
				</span>
				<span class="label__description">
					{{ formatMessage(messages.imageModalDescriptionFieldDescription) }}
				</span>
			</label>
			<StyledInput
				id="insert-image-alt"
				v-model="linkText"
				:icon="AlignLeftIcon"
				type="text"
				:placeholder="formatMessage(messages.imageModalDescriptionFieldPlaceholder)"
				clearable
				wrapper-class="w-full"
			/>
			<label class="label" for="insert-link-url">
				<span class="label__title">
					{{ formatMessage(messages.urlLabel) }}<span class="required">*</span>
				</span>
			</label>
			<div v-if="props.onImageUpload" class="image-strategy-chips">
				<Chips
					v-model="imageUploadOption"
					:items="['upload', 'link']"
					:format-label="formatImageUploadOption"
					:aria-label="formatMessage(messages.imageModalUploadModeLabel)"
				/>
			</div>
			<div
				v-if="props.onImageUpload && imageUploadOption === 'upload'"
				class="btn-input-alternative"
			>
				<FileInput
					accept="image/png,image/jpeg,image/gif,image/webp"
					:prompt="formatMessage(messages.imageModalUploadPrompt)"
					long-style
					should-always-reset
					class="file-input"
					@change="handleImageUpload"
				>
					<UploadIcon />
				</FileInput>
			</div>
			<StyledInput
				v-if="!props.onImageUpload || imageUploadOption === 'link'"
				id="insert-link-url"
				v-model="linkUrl"
				:icon="ImageIcon"
				type="text"
				:placeholder="formatMessage(messages.imageModalUrlFieldPlaceholder)"
				clearable
				wrapper-class="w-full"
				@input="validateURL"
			/>
			<template v-if="linkValidationErrorMessage">
				<span class="label">
					<span class="label__title">{{ formatMessage(messages.errorLabel) }}</span>
					<span class="label__description">{{ linkValidationErrorMessage }}</span>
				</span>
			</template>
			<span class="label">
				<span class="label__title">{{ formatMessage(messages.previewLabel) }}</span>
				<span class="label__description"></span>
			</span>
			<div class="markdown-body-wrapper">
				<div
					style="width: 100%"
					class="markdown-body"
					v-html="renderHighlightedString(imageMarkdown)"
				/>
			</div>
			<div class="flex gap-2 justify-end mt-4">
				<ButtonStyled type="outlined">
					<button @click="() => imageModal?.hide()">
						<XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						:disabled="!canInsertImage"
						@click="
							() => {
								if (editor) markdownCommands.replaceSelection(editor, imageMarkdown)
								imageModal?.hide()
							}
						"
					>
						<PlusIcon /> {{ formatMessage(messages.insertButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<NewModal ref="videoModal" :header="formatMessage(messages.videoModalHeader)" class="!w-[40rem]">
		<div class="modal-insert">
			<label class="label" for="insert-video-url">
				<span class="label__title">
					{{ formatMessage(messages.videoModalUrlFieldTitle) }}<span class="required">*</span>
				</span>
				<span class="label__description">
					{{ formatMessage(messages.videoModalUrlFieldDescription) }}
				</span>
			</label>
			<StyledInput
				id="insert-video-url"
				v-model="linkUrl"
				:icon="YouTubeIcon"
				type="text"
				:placeholder="formatMessage(messages.videoModalUrlFieldPlaceholder)"
				clearable
				wrapper-class="w-full"
				@input="validateURL"
			/>
			<template v-if="linkValidationErrorMessage">
				<span class="label">
					<span class="label__title">{{ formatMessage(messages.errorLabel) }}</span>
					<span class="label__description">{{ linkValidationErrorMessage }}</span>
				</span>
			</template>
			<span class="label">
				<span class="label__title">{{ formatMessage(messages.previewLabel) }}</span>
				<span class="label__description"></span>
			</span>

			<div class="markdown-body-wrapper">
				<div
					style="width: 100%"
					class="markdown-body"
					v-html="renderHighlightedString(videoMarkdown)"
				/>
			</div>
			<div class="flex gap-2 justify-end mt-4">
				<ButtonStyled type="outlined">
					<button @click="() => videoModal?.hide()">
						<XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						:disabled="!!linkValidationErrorMessage || !linkUrl"
						@click="
							() => {
								if (editor) markdownCommands.replaceSelection(editor, videoMarkdown)
								videoModal?.hide()
							}
						"
					>
						<PlusIcon /> {{ formatMessage(messages.insertButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<div class="block grow w-full">
		<div class="editor-action-row w-full">
			<div class="w-full flex justify-between items-center flex-wrap gap-2">
				<div class="editor-actions">
					<template
						v-for="(buttonGroup, _i) in Object.values(BUTTONS).filter((bg) => bg.display)"
						:key="_i"
					>
						<div class="divider"></div>
						<template v-for="button in buttonGroup.buttons" :key="button.label.id">
							<ButtonStyled circular>
								<button
									v-tooltip="formatMessage(button.label)"
									:aria-label="formatMessage(button.label)"
									:class="{ 'mobile-hidden-group': !!buttonGroup.hideOnMobile }"
									:disabled="previewMode || disabled"
									@click="() => button.action(editor)"
								>
									<component :is="button.icon" />
								</button>
							</ButtonStyled>
						</template>
					</template>
				</div>
				<div class="flex items-center gap-2">
					<Toggle id="preview" v-model="previewMode" small />
					<label class="label" for="preview">
						{{ formatMessage(messages.editorPreviewToggleLabel) }}
					</label>
				</div>
			</div>
		</div>
		<div ref="editorRef" :class="{ hide: previewMode }" />
		<div v-if="!previewMode" class="info-blurb mt-2">
			<div class="info-blurb">
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
				<span>{{ formatMessage(messages.editorMaxLengthLabel) }} </span>
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
		<div v-else>
			<div class="markdown-body-wrapper">
				<div
					style="width: 100%"
					:style="{
						maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
						overflowY: 'auto',
					}"
					class="markdown-body"
					v-html="renderHighlightedString(currentValue ?? '')"
				/>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { Compartment, EditorState } from '@codemirror/state'
import { EditorView, keymap, placeholder as cm_placeholder } from '@codemirror/view'
import {
	AlignLeftIcon,
	BoldIcon,
	CodeIcon,
	Heading1Icon,
	Heading2Icon,
	Heading3Icon,
	ImageIcon,
	InfoIcon,
	ItalicIcon,
	LinkIcon,
	ListBulletedIcon,
	ListOrderedIcon,
	PlusIcon,
	ScanEyeIcon,
	StrikethroughIcon,
	TextQuoteIcon,
	UploadIcon,
	XIcon,
	YouTubeIcon,
} from '@modrinth/assets'
import { markdownCommands, modrinthMarkdownEditorKeymap } from '@modrinth/utils/codemirror'
import { renderHighlightedString } from '@modrinth/utils/highlightjs'
import { type Component, computed, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages.ts'
import NewModal from '../modal/NewModal.vue'
import ButtonStyled from './ButtonStyled.vue'
import Chips from './Chips.vue'
import FileInput from './FileInput.vue'
import IntlFormatted from './IntlFormatted.vue'
import StyledInput from './StyledInput.vue'
import Toggle from './Toggle.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	insertButton: {
		id: 'markdown-editor.insert-button',
		defaultMessage: 'Insert',
	},
	urlLabel: {
		id: 'markdown-editor.url-label',
		defaultMessage: 'URL',
	},
	errorLabel: {
		id: 'markdown-editor.error-label',
		defaultMessage: 'Error',
	},
	previewLabel: {
		id: 'markdown-editor.preview-label',
		defaultMessage: 'Preview',
	},
	linkModalHeader: {
		id: 'markdown-editor.link-modal.header',
		defaultMessage: 'Insert link',
	},
	linkModalLabelFieldTitle: {
		id: 'markdown-editor.link-modal.label-field.title',
		defaultMessage: 'Label',
	},
	linkModalLabelFieldPlaceholder: {
		id: 'markdown-editor.link-modal.label-field.placeholder',
		defaultMessage: 'Enter label...',
	},
	linkModalUrlFieldPlaceholder: {
		id: 'markdown-editor.link-modal.url-field.placeholder',
		defaultMessage: "Enter the link's URL...",
	},
	imageModalHeader: {
		id: 'markdown-editor.image-modal.header',
		defaultMessage: 'Insert image',
	},
	imageModalDescriptionFieldTitle: {
		id: 'markdown-editor.image-modal.description-field.title',
		defaultMessage: 'Description (alt text)',
	},
	imageModalDescriptionFieldDescription: {
		id: 'markdown-editor.image-modal.description-field.description',
		defaultMessage:
			'Describe the image completely as you would to someone who could not see the image.',
	},
	imageModalDescriptionFieldPlaceholder: {
		id: 'markdown-editor.image-modal.description-field.placeholder',
		defaultMessage: 'Describe the image...',
	},
	imageModalUploadModeLabel: {
		id: 'markdown-editor.image-modal.upload-mode.label',
		defaultMessage: 'Image source',
	},
	imageModalUploadModeUpload: {
		id: 'markdown-editor.image-modal.upload-mode.upload',
		defaultMessage: 'Upload',
	},
	imageModalUploadModeLink: {
		id: 'markdown-editor.image-modal.upload-mode.link',
		defaultMessage: 'Link',
	},
	imageModalUploadPrompt: {
		id: 'markdown-editor.image-modal.upload.prompt',
		defaultMessage: 'Drag and drop to upload or click to select file',
	},
	imageModalUrlFieldPlaceholder: {
		id: 'markdown-editor.image-modal.url-field.placeholder',
		defaultMessage: 'Enter the image URL...',
	},
	videoModalHeader: {
		id: 'markdown-editor.video-modal.header',
		defaultMessage: 'Insert YouTube video',
	},
	videoModalUrlFieldTitle: {
		id: 'markdown-editor.video-modal.url-field.title',
		defaultMessage: 'YouTube video URL',
	},
	videoModalUrlFieldDescription: {
		id: 'markdown-editor.video-modal.url-field.description',
		defaultMessage: 'Enter a valid link to a YouTube video.',
	},
	videoModalUrlFieldPlaceholder: {
		id: 'markdown-editor.video-modal.url-field.placeholder',
		defaultMessage: 'Enter YouTube video URL',
	},
	editorPreviewToggleLabel: {
		id: 'markdown-editor.preview-toggle.label',
		defaultMessage: 'Preview',
	},
	editorMarkdownFormattingSupport: {
		id: 'markdown-editor.markdown-formatting-support',
		defaultMessage: 'This editor supports <markdown-link>Markdown formatting</markdown-link>.',
	},
	editorMaxLengthLabel: {
		id: 'markdown-editor.max-length.label',
		defaultMessage: 'Max length:',
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
	toolbarHeading1: {
		id: 'markdown-editor.toolbar.heading-1',
		defaultMessage: 'Heading 1',
	},
	toolbarHeading2: {
		id: 'markdown-editor.toolbar.heading-2',
		defaultMessage: 'Heading 2',
	},
	toolbarHeading3: {
		id: 'markdown-editor.toolbar.heading-3',
		defaultMessage: 'Heading 3',
	},
	toolbarBold: {
		id: 'markdown-editor.toolbar.bold',
		defaultMessage: 'Bold',
	},
	toolbarItalic: {
		id: 'markdown-editor.toolbar.italic',
		defaultMessage: 'Italic',
	},
	toolbarStrikethrough: {
		id: 'markdown-editor.toolbar.strikethrough',
		defaultMessage: 'Strikethrough',
	},
	toolbarCode: {
		id: 'markdown-editor.toolbar.code',
		defaultMessage: 'Code',
	},
	toolbarSpoiler: {
		id: 'markdown-editor.toolbar.spoiler',
		defaultMessage: 'Spoiler',
	},
	toolbarBulletedList: {
		id: 'markdown-editor.toolbar.bulleted-list',
		defaultMessage: 'Bulleted list',
	},
	toolbarOrderedList: {
		id: 'markdown-editor.toolbar.ordered-list',
		defaultMessage: 'Ordered list',
	},
	toolbarQuote: {
		id: 'markdown-editor.toolbar.quote',
		defaultMessage: 'Quote',
	},
	toolbarLink: {
		id: 'markdown-editor.toolbar.link',
		defaultMessage: 'Link',
	},
	toolbarImage: {
		id: 'markdown-editor.toolbar.image',
		defaultMessage: 'Image',
	},
	toolbarVideo: {
		id: 'markdown-editor.toolbar.video',
		defaultMessage: 'Video',
	},
	videoEmbedTitle: {
		id: 'markdown-editor.video-embed.title',
		defaultMessage: 'YouTube video player',
	},
	urlValidationErrorMalformed: {
		id: 'markdown-editor.url-validation-error.malformed',
		defaultMessage: 'Invalid URL. Make sure the URL is well-formed.',
	},
	urlValidationErrorUnsupportedProtocol: {
		id: 'markdown-editor.url-validation-error.unsupported-protocol',
		defaultMessage: 'Unsupported protocol. Use http or https.',
	},
	urlValidationErrorBlockedDomain: {
		id: 'markdown-editor.url-validation-error.blocked-domain',
		defaultMessage: 'Invalid URL. This domain is not allowed.',
	},
	uploadErrorNoHandler: {
		id: 'markdown-editor.upload-error.no-handler',
		defaultMessage: 'No image upload handler provided',
	},
	uploadErrorNoFile: {
		id: 'markdown-editor.upload-error.no-file',
		defaultMessage: 'No file provided',
	},
	defaultImageAltText: {
		id: 'markdown-editor.default-image-alt-text',
		defaultMessage: 'Replace this with a description',
	},
})

const props = withDefaults(
	defineProps<{
		modelValue: string
		disabled?: boolean
		headingButtons?: boolean
		/**
		 * @param file The file to upload
		 * @throws If the file is invalid or the upload fails
		 */
		onImageUpload?: (file: File) => Promise<string>
		placeholder?: string
		maxLength?: number
		maxHeight?: number
		minHeight?: number
	}>(),
	{
		modelValue: '',
		disabled: false,
		headingButtons: true,
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

const emit = defineEmits(['update:modelValue'])
const resolvedPlaceholder = computed(
	() => props.placeholder ?? formatMessage(messages.editorPlaceholder),
)

onMounted(() => {
	const updateListener = EditorView.updateListener.of((update) => {
		if (update.docChanged) {
			updateCurrentValue(update.state.doc.toString())
		}
	})

	editorThemeCompartment = new Compartment()

	const theme = EditorView.theme({
		// in defaults.scss there's references to .cm-content and such to inherit global styles
		'&': {
			borderRadius: 'var(--radius-md)',
			background: 'var(--color-button-bg)',
			border: '0.25rem solid transparent',
			transition: 'border-color 0.1s ease-in-out',
		},
		'&.cm-focused': {
			'box-shadow': 'inset 0 0 0 transparent, 0 0 0 0.25rem var(--color-brand-shadow)',
			color: 'var(--color-contrast)',
			outline: 'none',
		},
		'.cm-focused': {
			border: 'none',
		},
		'.cm-content': {
			minHeight: props.minHeight ? `${props.minHeight}px` : '200px',
			marginBlockEnd: '0.5rem',
			padding: '0.5rem',
			caretColor: 'var(--color-contrast)',
			width: '100%',
		},
		'.cm-scroller': {
			borderRadius: 'var(--radius-md)',
			height: '100%',
			maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
			overflow: 'auto',
		},
	})

	isDisabledCompartment = new Compartment()

	const disabledCompartment = EditorState.readOnly.of(props.disabled)

	const eventHandlers = EditorView.domEventHandlers({
		paste: (ev, view) => {
			const { clipboardData } = ev
			if (!clipboardData) return

			if (clipboardData.files && clipboardData.files.length > 0 && props.onImageUpload) {
				// If the user is pasting a file, upload it if there's an included handler and insert the link.
				uploadImagesFromList(clipboardData.files)
					.then(function (url) {
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
				try {
					cleanUrl(url)
				} catch (error: unknown) {
					if (error instanceof Error) {
						return
					}
				}

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
})

onBeforeUnmount(() => {
	editor?.destroy()
})

type ButtonAction = {
	label: MessageDescriptor
	icon: Component
	action: (editor: EditorView | null) => void
}
type ButtonGroup = {
	display: boolean
	hideOnMobile: boolean
	buttons: ButtonAction[]
}
type ButtonGroupMap = {
	[key: string]: ButtonGroup
}

function runEditorCommand(command: (view: EditorView) => boolean, editor: EditorView | null) {
	if (editor) {
		command(editor)
		editor.focus()
	}
}

const composeCommandButton = (
	label: MessageDescriptor,
	icon: Component,
	command: (view: EditorView) => boolean,
) => {
	return {
		label,
		icon,
		action: (e: EditorView | null) => runEditorCommand(command, e),
	}
}

const BUTTONS: ButtonGroupMap = {
	headings: {
		display: props.headingButtons,
		hideOnMobile: false,
		buttons: [
			composeCommandButton(messages.toolbarHeading1, Heading1Icon, markdownCommands.toggleHeader),
			composeCommandButton(messages.toolbarHeading2, Heading2Icon, markdownCommands.toggleHeader2),
			composeCommandButton(messages.toolbarHeading3, Heading3Icon, markdownCommands.toggleHeader3),
		],
	},
	stylizing: {
		display: true,
		hideOnMobile: false,
		buttons: [
			composeCommandButton(messages.toolbarBold, BoldIcon, markdownCommands.toggleBold),
			composeCommandButton(messages.toolbarItalic, ItalicIcon, markdownCommands.toggleItalic),
			composeCommandButton(
				messages.toolbarStrikethrough,
				StrikethroughIcon,
				markdownCommands.toggleStrikethrough,
			),
			composeCommandButton(messages.toolbarCode, CodeIcon, markdownCommands.toggleCodeBlock),
			composeCommandButton(messages.toolbarSpoiler, ScanEyeIcon, markdownCommands.toggleSpoiler),
		],
	},
	lists: {
		display: true,
		hideOnMobile: false,
		buttons: [
			composeCommandButton(
				messages.toolbarBulletedList,
				ListBulletedIcon,
				markdownCommands.toggleBulletList,
			),
			composeCommandButton(
				messages.toolbarOrderedList,
				ListOrderedIcon,
				markdownCommands.toggleOrderedList,
			),
			composeCommandButton(messages.toolbarQuote, TextQuoteIcon, markdownCommands.toggleQuote),
		],
	},
	components: {
		display: true,
		hideOnMobile: false,
		buttons: [
			{
				label: messages.toolbarLink,
				icon: LinkIcon,
				action: () => openLinkModal(),
			},
			{
				label: messages.toolbarImage,
				icon: ImageIcon,
				action: () => openImageModal(),
			},
			{
				label: messages.toolbarVideo,
				icon: YouTubeIcon,
				action: () => openVideoModal(),
			},
		],
	},
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
					effects: [
						editorThemeCompartment.reconfigure(
							EditorView.theme({
								// in defaults.scss there's references to .cm-content and such to inherit global styles
								'&': {
									borderRadius: 'var(--radius-md)',
									background: 'var(--color-button-bg)',
									border: '0.25rem solid transparent',
									transition: 'border-color 0.1s ease-in-out',
								},
								'&.cm-focused': {
									'box-shadow': 'inset 0 0 0 transparent, 0 0 0 0.25rem var(--color-brand-shadow)',
									color: 'var(--color-contrast)',
									outline: 'none',
								},
								'.cm-focused': {
									border: 'none',
								},
								'.cm-content': {
									minHeight: props.minHeight ? `${props.minHeight}px` : '200px',
									marginBlockEnd: '0.5rem',
									padding: '0.5rem',
									caretColor: 'var(--color-contrast)',
									width: '100%',
									opacity: newValue ? 0.6 : 1,
									pointerEvents: newValue ? 'none' : 'all',
									cursor: newValue ? 'not-allowed' : 'auto',
								},
								'.cm-scroller': {
									borderRadius: 'var(--radius-md)',
									height: '100%',
									maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
									overflow: 'auto',
								},
							}),
						),
					],
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

const previewMode = ref(false)

const linkText = ref('')
const linkUrl = ref('')
const linkValidationErrorMessage = ref<string | undefined>()

function validateURL() {
	if (!linkUrl.value || linkUrl.value === '') {
		linkValidationErrorMessage.value = undefined
		return
	}

	try {
		linkValidationErrorMessage.value = undefined
		cleanUrl(linkUrl.value)
	} catch (e: unknown) {
		if (e instanceof Error) {
			linkValidationErrorMessage.value = e.message
		}
	}
}

function cleanUrl(input: string): string {
	let url: URL

	// Attempt to validate and parse the URL
	try {
		url = new URL(input)
	} catch {
		throw new Error(formatMessage(messages.urlValidationErrorMalformed))
	}

	// Check for unsupported protocols
	if (url.protocol !== 'http:' && url.protocol !== 'https:') {
		throw new Error(formatMessage(messages.urlValidationErrorUnsupportedProtocol))
	}

	// If the scheme is "http", automatically upgrade it to "https"
	if (url.protocol === 'http:') {
		url.protocol = 'https:'
	}

	// Block certain domains for compliance
	const blockedDomains = ['forgecdn', 'cdn.discordapp', 'media.discordapp']
	if (blockedDomains.some((domain) => url.hostname.includes(domain))) {
		throw new Error(formatMessage(messages.urlValidationErrorBlockedDomain))
	}

	return url.toString()
}

const linkMarkdown = computed(() => {
	if (!linkUrl.value) {
		return ''
	}
	try {
		const url = cleanUrl(linkUrl.value)
		return url ? `[${linkText.value ? linkText.value : url}](${url})` : ''
	} catch (error: unknown) {
		if (error instanceof Error) {
			console.error(error.message)
		}
	}
	return ''
})

const uploadImagesFromList = async (files: FileList): Promise<string> => {
	const file = files[0]
	if (!props.onImageUpload) {
		throw new Error(formatMessage(messages.uploadErrorNoHandler))
	}
	if (file) {
		try {
			const url = await props.onImageUpload(file)
			return url
		} catch (error) {
			if (error instanceof Error) {
				console.error('Unable to upload image using handler.', error.message)
				throw new Error(error.message)
			}
		}
	}
	throw new Error(formatMessage(messages.uploadErrorNoFile))
}

const handleImageUpload = async (files: FileList) => {
	if (props.onImageUpload) {
		try {
			const uploadedURL = await uploadImagesFromList(files)
			linkUrl.value = uploadedURL
			validateURL()
		} catch (error) {
			if (error instanceof Error) {
				linkValidationErrorMessage.value = error.message
			}
			console.error(error)
		}
	}
}

const imageUploadOption = ref<string>('upload')
function formatImageUploadOption(option: string) {
	if (option === 'upload') {
		return formatMessage(messages.imageModalUploadModeUpload)
	}
	if (option === 'link') {
		return formatMessage(messages.imageModalUploadModeLink)
	}
	return option
}
const imageMarkdown = computed(() => (linkMarkdown.value.length ? `!${linkMarkdown.value}` : ''))

const canInsertImage = computed(() => {
	// Make sure the image url is valid, there is an image url, and there is alt text
	// They need to be valid, and not empty
	return (
		!linkValidationErrorMessage.value && linkUrl.value?.length > 0 && linkText.value?.length > 0
	)
})

const youtubeRegex =
	/^(?:https?:)?(?:\/\/)?(?:youtu\.be\/|(?:www\.|m\.)?youtube\.com\/(?:watch|v|embed)(?:\.php)?(?:\?.*v=|\/))(?<temp1>[a-zA-Z0-9_-]{7,15})(?:[?&][a-zA-Z0-9_-]+=[a-zA-Z0-9_-]+)*$/

const videoMarkdown = computed(() => {
	const match = youtubeRegex.exec(linkUrl.value)
	if (match) {
		return `<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/${match[1]}" title="${formatMessage(messages.videoEmbedTitle)}" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>`
	}
	return ''
})

const linkModal = ref<InstanceType<typeof NewModal> | null>(null)
const imageModal = ref<InstanceType<typeof NewModal> | null>(null)
const videoModal = ref<InstanceType<typeof NewModal> | null>(null)

function resetModalStates() {
	linkText.value = ''
	linkUrl.value = ''
	linkValidationErrorMessage.value = undefined
}

function openLinkModal() {
	if (editor) linkText.value = markdownCommands.yankSelection(editor)
	resetModalStates()
	linkModal.value?.show()
}

function openImageModal() {
	resetModalStates()
	imageModal.value?.show()
}

function openVideoModal() {
	resetModalStates()
	videoModal.value?.show()
}
</script>

<style lang="scss" scoped>
.file-input {
	width: 100%;
	padding: 1.5rem;
	padding-left: 2.5rem;
	background: var(--color-button-bg);
	border: 2px dashed var(--color-gray);
	border-radius: var(--radius-md);
	cursor: pointer;
	transition:
		opacity 0.5s ease-in-out,
		filter 0.2s ease-in-out,
		scale 0.05s ease-in-out,
		outline 0.2s ease-in-out;

	&:hover {
		filter: brightness(0.85);
	}
}

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

.editor-action-row {
	display: flex;
	align-items: center;
	flex-wrap: wrap;
	justify-content: space-between;
	margin-bottom: var(--gap-sm);
	gap: var(--gap-xs);

	@media (max-width: 768px) {
		flex-direction: column;
		align-items: start;
	}
}

.editor-actions {
	display: flex;
	flex-direction: row;
	flex-wrap: wrap;
	align-items: center;
	gap: var(--gap-xs);

	@media (max-width: 768px) {
		.divider {
			display: none;
		}

		.mobile-hidden-group {
			display: none;
		}
	}

	.divider {
		width: 0.125rem;
		height: 1.8rem;
		background-color: var(--color-button-bg);
		border-radius: var(--radius-max);
		margin-inline: var(--gap-xs);
	}

	.divider:first-child {
		display: none;
	}
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

.modal-insert {
	.label {
		margin-block: var(--gap-lg) var(--gap-sm);
		display: block;

		&:first-child {
			margin-top: 0;
		}
	}

	.label__title {
		color: var(--color-contrast);
		display: block;
		font-size: 1.17rem;
		font-weight: 700;

		.required {
			color: var(--color-red);
		}
	}

	.input-group {
		margin-top: var(--gap-lg);
	}
}

.image-strategy-chips {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--gap-xs);
	padding-bottom: var(--gap-md);
}

.btn-input-alternative {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: space-between;
	gap: var(--gap-xs);
	padding-bottom: var(--gap-xs);

	.btn {
		width: 100%;
		padding-left: 2.5rem;
		min-height: 4rem;
		display: flex;
		align-items: center;
		justify-content: start;
	}
}

.cm-disabled {
	opacity: 0.6;
	pointer-events: none;
	cursor: not-allowed;
}
</style>
