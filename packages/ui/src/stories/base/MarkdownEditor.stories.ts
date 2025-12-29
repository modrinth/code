import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import MarkdownEditor from '../../components/base/MarkdownEditor.vue'

const meta = {
	title: 'Base/MarkdownEditor',
	component: MarkdownEditor,
} satisfies Meta<typeof MarkdownEditor>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { MarkdownEditor },
		setup() {
			const content = ref('# Hello World\n\nThis is some **markdown** content.')
			return { content }
		},
		template: `
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
		template: `
			<div class="h-96">
				<MarkdownEditor v-model="content" placeholder="Write your description here..." />
			</div>
		`,
	}),
}
