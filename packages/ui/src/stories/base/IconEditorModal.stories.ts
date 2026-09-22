import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Button from '../../components/base/buttons/Button.vue'
import IconEditorModal from '../../components/base/icon-editor-modal/index.vue'
import type { IconConfig } from '../../components/base/icon-editor-modal/types'

const meta = {
	title: 'Base/IconEditorModal',
	component: IconEditorModal,
	parameters: { layout: 'padded' },
} satisfies Meta<typeof IconEditorModal>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		save: async () => {},
	},
	render: (args) => ({
		components: { Button, IconEditorModal },
		setup() {
			const editor = ref<InstanceType<typeof IconEditorModal> | null>(null)
			const saved = ref<IconConfig | null>(null)
			return { args, editor, saved }
		},
		template: `
			<div class="flex flex-col items-start gap-4">
				<Button @click="editor?.show()">Open icon editor</Button>
				<pre v-if="saved">{{ saved }}</pre>
				<IconEditorModal ref="editor" :save="args.save" @saved="(_result, config) => saved = config" />
			</div>
		`,
	}),
}
