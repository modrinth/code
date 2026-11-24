<template>
	<div class="editor-toolbar">
		<div v-if="!hideFormattingButtons" class="editor-actions">
			<template v-for="(group, index) in groups" :key="index">
				<template v-if="!group.headings || headingButtons">
					<div class="divider" />
					<IconButton
						v-for="button in group.buttons"
						:key="button.label.id"
						v-tooltip="formatMessage(button.label)"
						:label="formatMessage(button.label)"
						:type="buttonType"
						size="sm"
						:disabled="disabled"
						@click="button.action"
					>
						<component :is="button.icon" />
					</IconButton>
				</template>
			</template>
		</div>
		<slot />
	</div>
</template>

<script setup lang="ts">
import type { Command } from '@codemirror/view'
import {
	BoldIcon,
	CodeIcon,
	Heading1Icon,
	Heading2Icon,
	Heading3Icon,
	ImageIcon,
	ItalicIcon,
	LinkIcon,
	ListBulletedIcon,
	ListOrderedIcon,
	ScanEyeIcon,
	StrikethroughIcon,
	TextQuoteIcon,
	YouTubeIcon,
} from '@modrinth/assets'
import { markdownCommands } from '@modrinth/utils/codemirror'
import type { Component } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../../composables/i18n'
import IconButton from '../buttons/IconButton.vue'
import type { ButtonType } from '../buttons/types'

const props = withDefaults(
	defineProps<{
		disabled?: boolean
		headingButtons?: boolean
		hideFormattingButtons?: boolean
		buttonType?: ButtonType
		runCommand: (command: Command) => void
		openLinkModal: () => void
		openImageModal: () => void
		openVideoModal: () => void
	}>(),
	{ headingButtons: true },
)

const { formatMessage } = useVIntl()
const messages = defineMessages({
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
})

function composeCommandButton(label: MessageDescriptor, icon: Component, command: Command) {
	return { label, icon, action: () => props.runCommand(command) }
}

const groups = [
	{
		headings: true,
		buttons: [
			composeCommandButton(messages.toolbarHeading1, Heading1Icon, markdownCommands.toggleHeader),
			composeCommandButton(messages.toolbarHeading2, Heading2Icon, markdownCommands.toggleHeader2),
			composeCommandButton(messages.toolbarHeading3, Heading3Icon, markdownCommands.toggleHeader3),
		],
	},
	{
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
	{
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
	{
		buttons: [
			{
				label: messages.toolbarLink,
				icon: LinkIcon,
				action: () => props.openLinkModal(),
			},
			{
				label: messages.toolbarImage,
				icon: ImageIcon,
				action: () => props.openImageModal(),
			},
			{
				label: messages.toolbarVideo,
				icon: YouTubeIcon,
				action: () => props.openVideoModal(),
			},
		],
	},
]
</script>

<style lang="scss" scoped>
.editor-toolbar {
	display: flex;
	align-items: center;
	justify-content: space-between;
	flex-wrap: wrap;
	gap: 0.5rem;
}

.editor-actions {
	display: flex;
	flex-wrap: wrap;
	align-items: center;
	gap: var(--gap-xs);
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

@media (max-width: 768px) {
	.divider {
		display: none;
	}
}
</style>
