import type { Meta, StoryObj } from '@storybook/vue3-vite'

import PreviewSelectButton from '../../components/base/PreviewSelectButton.vue'

const meta = {
	title: 'Base/PreviewSelectButton',
	component: PreviewSelectButton,
} satisfies Meta<typeof PreviewSelectButton>

export default meta
type Story = StoryObj<typeof meta> & { args?: Record<string, unknown> }

export const Default: Story = {
	args: { checked: false },
	render: () => ({
		components: { PreviewSelectButton },
		template: `
			<PreviewSelectButton :checked="false">
				<template #preview>
					<div class="w-16 h-16 bg-brand-highlight rounded-lg" />
				</template>
				Option Label
			</PreviewSelectButton>
		`,
	}),
}

export const AllStates: Story = {
	args: { checked: false },
	render: () => ({
		components: { PreviewSelectButton },
		template: `
			<div class="flex gap-4">
				<PreviewSelectButton :checked="false">
					<template #preview>
						<div class="w-16 h-16 bg-bg-raised rounded-lg flex items-center justify-center">
							<span class="text-secondary">A</span>
						</div>
					</template>
					Unchecked
				</PreviewSelectButton>
				<PreviewSelectButton :checked="true">
					<template #preview>
						<div class="w-16 h-16 bg-brand-highlight rounded-lg flex items-center justify-center">
							<span class="text-brand">B</span>
						</div>
					</template>
					Checked
				</PreviewSelectButton>
			</div>
		`,
	}),
}
