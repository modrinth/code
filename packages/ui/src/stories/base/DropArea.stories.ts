import type { Meta, StoryObj } from '@storybook/vue3-vite'

import DropArea from '../../components/base/DropArea.vue'

const meta = {
	title: 'Base/DropArea',
	component: DropArea,
} satisfies Meta<typeof DropArea>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { DropArea },
		template: `
			<DropArea accept="*" @change="(files) => console.log('Files dropped:', files)">
				<div class="p-8 border-2 border-dashed border-divider rounded-lg text-center">
					<p class="text-secondary">Drag and drop files anywhere on the page</p>
					<p class="text-sm text-secondary mt-2">The drop overlay will appear when you drag files over</p>
				</div>
			</DropArea>
		`,
	}),
}

export const ImagesOnly: StoryObj = {
	render: () => ({
		components: { DropArea },
		template: `
			<DropArea accept="image/*" @change="(files) => console.log('Images dropped:', files)">
				<div class="p-8 border-2 border-dashed border-divider rounded-lg text-center">
					<p class="text-secondary">Drop images here</p>
					<p class="text-sm text-secondary mt-2">Only accepts image files</p>
				</div>
			</DropArea>
		`,
	}),
}
