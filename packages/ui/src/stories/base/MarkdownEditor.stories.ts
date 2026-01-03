import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import MarkdownEditor from '../../components/base/MarkdownEditor.vue'

const meta = {
	title: 'Base/MarkdownEditor',
	component: MarkdownEditor,
} satisfies Meta<typeof MarkdownEditor>

export default meta


const CONTENT = `# Scrolling in Small Container

This story shows the markdown editor with a smaller fixed height to demonstrate scrolling.

## Multiple Paragraphs

Paragraph 1: Lorem ipsum dolor sit amet, consectetur adipiscing elit.

Paragraph 2: Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

Paragraph 3: Ut enim ad minim veniam, quis nostrud exercitation ullamco.

Paragraph 4: Duis aute irure dolor in reprehenderit in voluptate velit.

Paragraph 5: Excepteur sint occaecat cupidatat non proident.

## Lists

- Item A
- Item B
- Item C
- Item D
- Item E
- Item F
- Item G

## Conclusion

More content to ensure scrolling is triggered.`

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
