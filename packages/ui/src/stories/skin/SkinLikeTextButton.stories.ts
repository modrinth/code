import { PlusIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import SkinLikeTextButton from '../../components/skin/SkinLikeTextButton.vue'

const meta = {
	title: 'Skin/SkinLikeTextButton',
	component: SkinLikeTextButton,
	args: {
		selected: false,
		tooltip: 'Add a skin',
		dragActive: false,
		dropzone: false,
	},
	render: (args) => ({
		components: { PlusIcon, SkinLikeTextButton },
		setup() {
			return { args }
		},
		template: /* html */ `
			<div class="w-[156px]">
				<SkinLikeTextButton v-bind="args">
					<template #icon>
						<PlusIcon class="size-8" />
					</template>
					Add skin
					<template #subtitle>Drag and drop</template>
				</SkinLikeTextButton>
			</div>
		`,
	}),
} satisfies Meta<typeof SkinLikeTextButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Selected: Story = {
	args: {
		selected: true,
	},
}

export const DragActive: Story = {
	args: {
		dragActive: true,
		dropzone: true,
	},
}
