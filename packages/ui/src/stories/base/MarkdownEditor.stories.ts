import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import MarkdownEditor from '../../components/base/MarkdownEditor.vue'

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
		components: { MarkdownEditor },
		setup() {
			const content = ref('')
			return { content }
		},
		template: /*html*/ `
			<MarkdownEditor v-model="content" image-only-toolbar hide-markdown-hint :min-height="80" />
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
			<MarkdownEditor ref="editor" v-model="content" image-only-toolbar="external" hide-markdown-hint :min-height="80" />
			<Button class="mt-2" @click="editor?.openImageModal()">Attach image</Button>
		`,
	}),
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
