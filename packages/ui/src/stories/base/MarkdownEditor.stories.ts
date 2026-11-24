import { EditorView } from '@codemirror/view'
import { ImageIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, userEvent, waitFor, within } from 'storybook/test'
import { ref } from 'vue'

import { Button, IconButton } from '../../components/base/buttons'
import { MarkdownEditor } from '../../components/base/markdown-editor'
import { codeClipboardHtml } from '../../utils/markdown-code'

const meta = {
	title: 'Base/MarkdownEditor',
	component: MarkdownEditor,
} satisfies Meta<typeof MarkdownEditor>

export default meta

const CONTENT = `# Sample Markdown Document

This is a demonstration of the markdown editor component.

## Features

The editor supports various markdown formatting options:

- **Bold text** for emphasis
- *Italic text* for subtle emphasis
- \`Inline code\` for technical terms
- [Links](https://example.com) for references

## Code Blocks

\`\`\`javascript
function greet(name) {
	return \`Hello, \${name}!\`;
}
\`\`\`

## Lists

1. First item
2. Second item
3. Third item

### Nested Lists

- Parent item
	- Child item
	- Another child
- Another parent

## Blockquotes

> This is a blockquote that can be used for callouts or citations.

## Conclusion

This content demonstrates the editor's capabilities.`

export const Default: StoryObj = {
	render: () => ({
		components: { MarkdownEditor },
		setup() {
			const content = ref('# Hello World\n\nThis is some **markdown** content.')
			return { content }
		},
		template: /*html*/ `
			<div class="h-96">
				<MarkdownEditor v-model="content" />
			</div>
		`,
	}),
}

export const WithPlaceholder: StoryObj = {
	render: () => ({
		components: { MarkdownEditor },
		setup() {
			const content = ref('')
			return { content }
		},
		template: /*html*/ `
			<div class="h-96">
				<MarkdownEditor v-model="content" placeholder="Write your description here..." />
			</div>
		`,
	}),
}

export const WithContent: StoryObj = {
	render: () => ({
		components: { MarkdownEditor },
		setup() {
			const content = ref(CONTENT)
			return { content }
		},
		template: /*html*/ `
			<div>
				<MarkdownEditor v-model="content" />
			</div>
		`,
	}),
}

export const ImageOnlyToolbar: StoryObj = {
	render: () => ({
		components: { IconButton, ImageIcon, MarkdownEditor },
		setup() {
			const content = ref('')
			return { content }
		},
		template: /*html*/ `
			<MarkdownEditor v-model="content" hide-markdown-hint :min-height="80">
				<template #toolbar />
				<template #footer="{ disabled, openImageModal }">
					<IconButton class="mt-2" size="sm" label="Image" :disabled="disabled" @click="openImageModal">
						<ImageIcon aria-hidden="true" />
					</IconButton>
				</template>
			</MarkdownEditor>
		`,
	}),
}

export const ExternalImageAction: StoryObj = {
	render: () => ({
		components: { Button, MarkdownEditor },
		setup() {
			const content = ref('')
			const editor = ref<InstanceType<typeof MarkdownEditor>>()
			return { content, editor }
		},
		template: /*html*/ `
			<MarkdownEditor ref="editor" v-model="content" hide-markdown-hint :min-height="80">
				<template #toolbar />
			</MarkdownEditor>
			<Button class="mt-2" @click="editor?.openImageModal()">Attach image</Button>
		`,
	}),
}

export const InsertModals: StoryObj = {
	render: () => ({
		components: { MarkdownEditor },
		setup() {
			return { content: ref('') }
		},
		template: /*html*/ `<MarkdownEditor v-model="content" />`,
	}),
	play: async ({ canvasElement }) => {
		const canvas = within(canvasElement)
		const page = within(canvasElement.ownerDocument.body)

		await userEvent.click(await canvas.findByRole('button', { name: 'Link' }))
		const link = within(await page.findByRole('dialog', { name: 'Insert link' }))
		await userEvent.type(link.getByPlaceholderText('Enter label...'), 'Example')
		await userEvent.type(
			link.getByPlaceholderText("Enter the link's URL..."),
			'https://example.com',
		)
		await userEvent.click(link.getByRole('button', { name: 'Insert' }))
		await waitFor(() => expect(page.queryByRole('dialog', { name: 'Insert link' })).toBeNull())

		await userEvent.click(canvas.getByRole('button', { name: 'Image' }))
		const image = within(await page.findByRole('dialog', { name: 'Insert image' }))
		await userEvent.type(image.getByPlaceholderText('Describe the image...'), 'Example image')
		await userEvent.type(
			image.getByPlaceholderText('Enter the image URL...'),
			'https://example.com/image.png',
		)
		await userEvent.click(image.getByRole('button', { name: 'Insert' }))
		await waitFor(() => expect(page.queryByRole('dialog', { name: 'Insert image' })).toBeNull())

		await userEvent.click(canvas.getByRole('button', { name: 'Video' }))
		const video = within(await page.findByRole('dialog', { name: 'Insert YouTube video' }))
		await userEvent.type(
			video.getByPlaceholderText('Enter YouTube video URL'),
			'https://youtu.be/abcdefghijk',
		)
		await userEvent.click(video.getByRole('button', { name: 'Insert' }))
		await waitFor(() =>
			expect(page.queryByRole('dialog', { name: 'Insert YouTube video' })).toBeNull(),
		)

		const editor = EditorView.findFromDOM(canvasElement.querySelector<HTMLElement>('.cm-editor')!)!
		const source = editor.state.doc.toString()
		expect(source).toContain('[Example](https://example.com/)')
		expect(source).toContain('![Example image](https://example.com/image.png)')
		expect(source).toContain('https://www.youtube-nocookie.com/embed/abcdefghijk')
	},
}

export const MaxHeightWithScroll: StoryObj = {
	render: () => ({
		components: { MarkdownEditor },
		setup() {
			const content = ref(CONTENT)
			return { content }
		},
		template: /*html*/ `
			<div>
				<MarkdownEditor v-model="content" :max-height="300" />
			</div>
		`,
	}),
}

const PASTED_CODE = 'const answer = 42;\nconsole.log(answer);'
const EXISTING_CODE_BLOCK = '```javascript\n// Paste below\n\n```\n'

function renderCodePaste(initialContent: string) {
	return () => ({
		components: { MarkdownEditor },
		setup() {
			const content = ref(initialContent)
			return { content }
		},
		template: /*html*/ `
			<div class="max-w-3xl">
				<p>Code-aware clipboard content should create a fenced block in prose and remain plain code inside an existing block.</p>
				<MarkdownEditor v-model="content" />
				<pre data-testid="markdown-source" class="whitespace-pre-wrap">{{ content }}</pre>
			</div>
		`,
	})
}

async function pasteCode(canvasElement: HTMLElement, position: number, expected: string) {
	await waitFor(() => expect(canvasElement.querySelector('.cm-editor')).not.toBeNull())
	const editor = EditorView.findFromDOM(canvasElement.querySelector<HTMLElement>('.cm-editor')!)!
	editor.dispatch({ selection: { anchor: position } })
	editor.focus()
	const clipboardData = new DataTransfer()
	clipboardData.setData('text/plain', PASTED_CODE)
	clipboardData.setData(
		'text/html',
		codeClipboardHtml(canvasElement.ownerDocument, PASTED_CODE, 'javascript'),
	)
	editor.contentDOM.dispatchEvent(
		new ClipboardEvent('paste', { clipboardData, bubbles: true, cancelable: true }),
	)
	await waitFor(() => {
		expect(editor.state.doc.toString()).toBe(expected)
		expect(canvasElement.querySelector('[data-testid="markdown-source"]')?.textContent).toBe(
			expected,
		)
	})
}

export const CodePasteInProse: StoryObj = {
	render: renderCodePaste('Before after'),
	play: async ({ canvasElement }) => {
		await pasteCode(canvasElement, 7, `Before \n\`\`\`javascript\n${PASTED_CODE}\n\`\`\`\nafter`)
	},
}

export const CodePasteInsideCodeBlock: StoryObj = {
	render: renderCodePaste(EXISTING_CODE_BLOCK),
	play: async ({ canvasElement }) => {
		const position = EXISTING_CODE_BLOCK.indexOf('\n\n') + 1
		await pasteCode(
			canvasElement,
			position,
			EXISTING_CODE_BLOCK.slice(0, position) + PASTED_CODE + EXISTING_CODE_BLOCK.slice(position),
		)
	},
}
